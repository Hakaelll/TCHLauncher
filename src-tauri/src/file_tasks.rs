use crate::{backup,download::DownloadTask,model::{Backup,Progress}};
use std::{collections::HashMap,path::Path,sync::{Arc,Mutex}};

pub type Registry=Arc<Mutex<HashMap<String,Arc<DownloadTask>>>>;

struct Registration{jobs:Registry,server_id:String,task_id:String}
impl Drop for Registration{
    fn drop(&mut self){
        if let Ok(mut jobs)=self.jobs.lock(){
            if jobs.get(&self.server_id).is_some_and(|task|task.id==self.task_id){jobs.remove(&self.server_id);}
        }
    }
}

// The caller holds the server operation reservation through this whole call.
fn registered<T>(id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static,operation:impl FnOnce(&DownloadTask)->Result<T,String>)->Result<T,String>{
    registered_kind(id,"backup",jobs,emit,operation)
}

fn registered_kind<T>(id:&str,kind:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static,operation:impl FnOnce(&DownloadTask)->Result<T,String>)->Result<T,String>{
    let task=Arc::new(DownloadTask::new(id,kind,emit)?);
    {
        let mut jobs=jobs.lock().map_err(|_|"No se pudieron registrar las tareas")?;
        if jobs.contains_key(id){return Err("Ya hay una tarea activa para este servidor".into());}
        jobs.insert(id.into(),task.clone());
    }
    let _registration=Registration{jobs:jobs.clone(),server_id:id.into(),task_id:task.id.clone()};
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||operation(&task)))
        .unwrap_or_else(|_|Err(if kind=="backup"{"La copia se interrumpió inesperadamente"}else{"La operación se interrumpió inesperadamente"}.into()));
    task.finish(&result.as_ref().map(|_|()).map_err(Clone::clone));
    result
}

pub fn create_backup(root:&Path,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered(id,jobs,emit,|task|backup::create_tracked(root,id,task))
}

pub fn import_world(server:&crate::model::Server,source:&Path,name:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<crate::world::WorldInfo,String>{
    registered_kind(&server.id,"world_import",jobs,emit,|task|crate::world::import_tracked(server,source,name,task))
}

pub fn import_server(root:&Path,input:crate::model::ImportServer,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<crate::model::Server,String>{
    registered_kind(id,"server_import",jobs,emit,|task|crate::storage::import_tracked(root,input,id,task))
}

pub fn remove_content(root:&Path,server:&crate::model::Server,filename:&str,sha512:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<(),String>{
    registered_kind(&server.id,"content_remove",jobs,emit,|task|crate::content_storage::remove_tracked(root,server,filename,sha512,task))
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::{scheduler,storage};
    use std::fs;
    use uuid::Uuid;

    struct Library(std::path::PathBuf);
    impl Drop for Library{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    #[test]
    fn scheduled_backup_is_visible_until_terminal_event_and_records_real_files(){
        let root=Library(std::env::temp_dir().join(format!("tch-scheduled-progress-{}",Uuid::new_v4())));
        fs::create_dir_all(&root.0).unwrap();
        let server=storage::create(&root.0,crate::model::CreateServer{name:"Scheduled".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        fs::write(Path::new(&server.directory).join("keep.bin"),vec![8u8;300_000]).unwrap();
        scheduler::add(&root.0,&server.id,scheduler::Input{action:"backup".into(),next_at:1000,interval_minutes:Some(1)},0).unwrap();
        let due=scheduler::claim(&root.0,1000).unwrap().unwrap();
        let jobs:Registry=Arc::new(Mutex::new(HashMap::new()));let observed=jobs.clone();
        let events=Arc::new(Mutex::new(Vec::<Progress>::new()));let capture=events.clone();
        let copy=create_backup(&root.0,&due.server_id,&jobs,move|event|{
            assert_eq!(observed.lock().unwrap().get(&event.server_id).unwrap().id,event.task_id);
            capture.lock().unwrap().push(event);
        }).unwrap();
        scheduler::finish(&root.0,&due.id,"Copia guardada").unwrap();
        assert!(jobs.lock().unwrap().is_empty());assert_eq!(fs::read(Path::new(&copy.path).join("keep.bin")).unwrap(),vec![8u8;300_000]);
        assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),1);
        let events=events.lock().unwrap();assert!(events.iter().any(|event|event.stage=="copying"&&event.received>0));assert_eq!(events.last().unwrap().stage,"completed");
        let scheduled=scheduler::add(&root.0,&server.id,scheduler::Input{action:"backup".into(),next_at:2000,interval_minutes:None},0).unwrap();
        let cancelled=scheduler::claim(&root.0,2000).unwrap().unwrap();assert_eq!(cancelled.id,scheduled.id);
        let observed=jobs.clone();let events=Arc::new(Mutex::new(Vec::<Progress>::new()));let capture=events.clone();
        let result=create_backup(&root.0,&cancelled.server_id,&jobs,move|event|{
            if event.stage=="copying"&&event.received>0{assert!(observed.lock().unwrap()[&event.server_id].request_cancel());}
            capture.lock().unwrap().push(event);
        });
        assert_eq!(result.err().unwrap(),crate::download::CANCELLED);scheduler::finish(&root.0,&cancelled.id,"Copia cancelada; servidor detenido").unwrap();
        assert!(jobs.lock().unwrap().is_empty());assert_eq!(events.lock().unwrap().last().unwrap().stage,"cancelled");
        assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),1);assert_eq!(fs::read(Path::new(&server.directory).join("keep.bin")).unwrap(),vec![8u8;300_000]);
    }
    #[test]
    fn server_import_tracks_copy_cancellation_database_failure_and_external_registration(){
        let root=Library(std::env::temp_dir().join(format!("tch-server-import-progress-{}",Uuid::new_v4())));
        let source=root.0.join("original");fs::create_dir_all(&source).unwrap();
        fs::write(source.join("server.properties"),"server-port=25566\nmax-players=12\n").unwrap();
        fs::write(source.join("server.jar"),vec![5u8;300_000]).unwrap();
        let expected=backup::size(&source);
        storage::connect(&root.0).unwrap();
        for mode in ["cancel","database_failure","copy","external"]{
            let id=Uuid::new_v4().to_string();let jobs:Registry=Arc::new(Mutex::new(HashMap::new()));let observed=jobs.clone();
            let events=Arc::new(Mutex::new(Vec::<Progress>::new()));let capture=events.clone();let library=root.0.clone();
            if mode=="database_failure"{storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER reject_import BEFORE INSERT ON servers BEGIN SELECT RAISE(ABORT,'simulated database failure'); END;").unwrap();}
            let before=storage::list(&root.0).unwrap().len();
            let input=crate::model::ImportServer{path:source.to_string_lossy().into(),name:mode.into(),version:"1.21.4".into(),platform:"vanilla".into(),jar_file:"server.jar".into(),copy:mode!="external",storage_target_gb:20,image_path:None};
            let result=import_server(&root.0,input,&id,&jobs,move|event|{
                let task=observed.lock().unwrap()[&event.server_id].clone();assert_eq!(task.id,event.task_id);assert_eq!(event.kind,"server_import");
                if event.stage=="copying"&&event.received>0&&mode=="cancel"{assert!(task.request_cancel());}
                if event.stage=="publishing"{assert!(!task.request_cancel());assert_eq!(storage::list(&library).unwrap().len(),before);}
                capture.lock().unwrap().push(event);
            });
            let events=events.lock().unwrap();assert!(jobs.lock().unwrap().is_empty());
            if mode=="cancel"{assert_eq!(result.unwrap_err(),crate::download::CANCELLED);assert_eq!(events.last().unwrap().stage,"cancelled");}
            else if mode=="database_failure"{assert!(result.is_err());assert_eq!(events.last().unwrap().stage,"failed");storage::connect(&root.0).unwrap().execute_batch("DROP TRIGGER reject_import;").unwrap();}
            else {let server=result.unwrap();assert_eq!(server.id,id);assert_eq!(events.last().unwrap().stage,"completed");assert_eq!(fs::read(Path::new(&server.directory).join("server.jar")).unwrap(),vec![5u8;300_000]);if mode=="external"{assert_eq!(Path::new(&server.directory).canonicalize().unwrap(),source.canonicalize().unwrap());assert!(!events.iter().any(|event|event.stage=="copying"));}else{let measured=events.iter().filter(|event|event.stage=="copying").last().unwrap();assert_eq!(measured.received,expected);assert_eq!(measured.total,Some(expected));}}
            assert_eq!(storage::list(&root.0).unwrap().len(),before+usize::from(mode=="copy"||mode=="external"));
            assert!(!root.0.join("servers").join(format!(".{id}.part")).exists());
            assert_eq!(root.0.join("servers").join(&id).exists(),mode=="copy");
            assert_eq!(fs::read(source.join("server.jar")).unwrap(),vec![5u8;300_000]);
        }
    }

    #[test]
    fn occupied_registry_is_preserved_and_panics_finish_and_release_registration(){
        let jobs:Registry=Arc::new(Mutex::new(HashMap::new()));let existing=Arc::new(DownloadTask::new("server","install",|_|{}).unwrap());
        jobs.lock().unwrap().insert("server".into(),existing.clone());
        assert!(registered("server",&jobs,|_|{},|_|Ok(())).is_err());assert_eq!(jobs.lock().unwrap()["server"].id,existing.id);
        jobs.lock().unwrap().clear();let events=Arc::new(Mutex::new(Vec::<Progress>::new()));let capture=events.clone();
        let result:Result<(),String>=registered("server",&jobs,move|event|capture.lock().unwrap().push(event),|_|panic!("simulated worker failure"));
        assert_eq!(result.unwrap_err(),"La copia se interrumpió inesperadamente");assert!(jobs.lock().unwrap().is_empty());assert_eq!(events.lock().unwrap().last().unwrap().stage,"failed");
    }
}
