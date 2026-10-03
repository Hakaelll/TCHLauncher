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
    let task=Arc::new(DownloadTask::new(id,"backup",emit)?);
    {
        let mut jobs=jobs.lock().map_err(|_|"No se pudieron registrar las tareas")?;
        if jobs.contains_key(id){return Err("Ya hay una tarea activa para este servidor".into());}
        jobs.insert(id.into(),task.clone());
    }
    let _registration=Registration{jobs:jobs.clone(),server_id:id.into(),task_id:task.id.clone()};
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||operation(&task)))
        .unwrap_or_else(|_|Err("La copia se interrumpió inesperadamente".into()));
    task.finish(&result.as_ref().map(|_|()).map_err(Clone::clone));
    result
}

pub fn create_backup(root:&Path,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered(id,jobs,emit,|task|backup::create_tracked(root,id,task))
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
    fn occupied_registry_is_preserved_and_panics_finish_and_release_registration(){
        let jobs:Registry=Arc::new(Mutex::new(HashMap::new()));let existing=Arc::new(DownloadTask::new("server","install",|_|{}).unwrap());
        jobs.lock().unwrap().insert("server".into(),existing.clone());
        assert!(registered("server",&jobs,|_|{},|_|Ok(())).is_err());assert_eq!(jobs.lock().unwrap()["server"].id,existing.id);
        jobs.lock().unwrap().clear();let events=Arc::new(Mutex::new(Vec::<Progress>::new()));let capture=events.clone();
        let result:Result<(),String>=registered("server",&jobs,move|event|capture.lock().unwrap().push(event),|_|panic!("simulated worker failure"));
        assert_eq!(result.unwrap_err(),"La copia se interrumpió inesperadamente");assert!(jobs.lock().unwrap().is_empty());assert_eq!(events.lock().unwrap().last().unwrap().stage,"failed");
    }
}
