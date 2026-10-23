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

pub fn create_stop_backup(root:&Path,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered(id,jobs,emit,|task|backup::create_tracked_for(root,id,task,backup::Reason::ServerStop))
}

pub fn create_scheduled_backup(root:&Path,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered(id,jobs,emit,|task|backup::create_tracked_for(root,id,task,backup::Reason::Scheduled))
}

pub fn create_backup(root:&Path,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered(id,jobs,emit,|task|backup::create_tracked(root,id,task))
}

pub fn import_world(root:&Path,server:&crate::model::Server,source:&Path,name:&str,dimensions:&[String],jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<crate::world::WorldInfo,String>{
    registered_kind(&server.id,"world_import",jobs,emit,|task|crate::world_import::import(root,server,source,name,dimensions,task))
}

pub fn import_server(root:&Path,input:crate::model::ImportServer,id:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<crate::model::Server,String>{
    registered_kind(id,"server_import",jobs,emit,|task|crate::storage::import_tracked(root,input,id,task))
}

pub fn duplicate_world(root:&Path,server:&crate::model::Server,name:&str,new_name:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<crate::world::WorldInfo,String>{
    registered_kind(&server.id,"world_duplicate",jobs,emit,|task|crate::world::duplicate_tracked(root,server,name,new_name,task))
}

pub fn export_world(root:&Path,server:&crate::model::Server,name:&str,parent:&Path,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<std::path::PathBuf,String>{
    registered_kind(&server.id,"world_export",jobs,emit,|task|crate::world::export_tracked(root,server,name,parent,task))
}

pub fn export_world_bundle(server:&crate::model::Server,name:&str,destination:&Path,include_config:bool,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<std::path::PathBuf,String>{
    registered_kind(&server.id,"world_export",jobs,emit,|task|crate::world_bundle::export(server,name,destination,include_config,task))
}

pub fn set_active_world(root:&Path,server:&crate::model::Server,name:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Option<Backup>,String>{
    registered_kind(&server.id,"world_activate",jobs,emit,|task|crate::world::set_active_tracked(root,server,name,task))
}

pub fn delete_world(root:&Path,server:&crate::model::Server,name:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<Backup,String>{
    registered_kind(&server.id,"world_delete",jobs,emit,|task|crate::world::delete_tracked(root,server,name,task))
}

pub fn remove_content(root:&Path,server:&crate::model::Server,filename:&str,sha512:&str,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<(),String>{
    registered_kind(&server.id,"content_remove",jobs,emit,|task|crate::content_storage::remove_tracked(root,server,filename,sha512,task))
}

pub fn set_content_enabled(root:&Path,server:&crate::model::Server,filename:&str,sha512:&str,enabled:bool,jobs:&Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<(),String>{
    registered_kind(&server.id,"content_toggle",jobs,emit,|task|crate::content_storage::set_enabled_tracked(root,server,filename,sha512,enabled,task))
}
