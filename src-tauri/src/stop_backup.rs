//! Explicit opt-in backup after a clean exit of a server that reached Running.
use crate::{storage,file_tasks,model::Progress,process::Exit};
use rusqlite::{params,OptionalExtension};
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status{pub enabled:bool,pub phase:String,pub last_at:Option<i64>,pub last_result:Option<String>}
fn db(root:&Path)->Result<rusqlite::Connection,String>{
    let db=storage::connect(root)?;db.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e|e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS stop_backup(server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,enabled INTEGER NOT NULL DEFAULT 0,last_token TEXT,phase TEXT NOT NULL DEFAULT 'idle',last_at INTEGER,last_result TEXT);").map_err(|e|e.to_string())?;Ok(db)
}
pub fn status(root:&Path,id:&str)->Result<Status,String>{
    storage::get(root,id)?;
    db(root)?.query_row("SELECT enabled,phase,last_at,last_result FROM stop_backup WHERE server_id=?1",[id],|row|Ok(Status{enabled:row.get(0)?,phase:row.get(1)?,last_at:row.get(2)?,last_result:row.get(3)?})).optional().map_err(|e|e.to_string()).map(|value|value.unwrap_or(Status{enabled:false,phase:"idle".into(),last_at:None,last_result:None}))
}
pub fn configure(root:&Path,id:&str,enabled:bool,current_token:Option<&str>)->Result<(),String>{
    storage::get(root,id)?;
    if let Some(token)=current_token{uuid::Uuid::parse_str(token).map_err(|_|"La sesión del servidor no es válida")?;}
    let changed=db(root)?.execute("INSERT INTO stop_backup(server_id,enabled,last_token) VALUES(?1,?2,?3) ON CONFLICT(server_id) DO UPDATE SET enabled=?2,last_token=?3,phase='idle',last_at=NULL,last_result=NULL WHERE phase!='copying'",params![id,enabled,current_token]).map_err(|e|e.to_string())?;
    if changed!=1{return Err("Espera a que termine la copia automática".into());}Ok(())
}
pub fn pending(root:&Path,id:&str,exit:&Exit)->Result<bool,String>{
    if !exit.successful||!exit.was_running{return Ok(false);}
    db(root)?.query_row("SELECT EXISTS(SELECT 1 FROM stop_backup WHERE server_id=?1 AND enabled=1 AND phase!='copying' AND (last_token IS NULL OR last_token!=?2))",params![id,exit.token],|row|row.get(0)).map_err(|e|e.to_string())
}
pub fn run(root:&Path,id:&str,exit:&Exit,jobs:&file_tasks::Registry,emit:impl Fn(Progress)+Send+Sync+'static)->Result<bool,String>{
    if !exit.successful||!exit.was_running{return Ok(false);}
    storage::get(root,id)?;uuid::Uuid::parse_str(&exit.token).map_err(|_|"La sesión del servidor no es válida")?;
    // The caller holds the stopped-server reservation. Claim before copying so a
    // failed/interrupted/cancelled attempt cannot replay this process session.
    let changed=db(root)?.execute("UPDATE stop_backup SET last_token=?2,phase='copying',last_at=?3,last_result='Creando copia tras la parada' WHERE server_id=?1 AND enabled=1 AND phase!='copying' AND (last_token IS NULL OR last_token!=?2)",params![id,exit.token,chrono::Utc::now().timestamp_millis()]).map_err(|e|e.to_string())?;
    if changed==0{return Ok(false);}
    let result=file_tasks::create_stop_backup(root,id,jobs,emit);
    let (phase,message)=match &result{
        Ok(copy)=>("completed",copy.retention_warning.clone().unwrap_or_else(||"Copia guardada tras detener el servidor".into())),
        Err(error) if error==crate::download::CANCELLED=>("cancelled","Copia automática cancelada; no se repetirá esta parada".into()),
        Err(error)=>("error",format!("Error: {error}")),
    };
    db(root)?.execute("UPDATE stop_backup SET phase=?3,last_result=?4 WHERE server_id=?1 AND last_token=?2 AND phase='copying'",params![id,exit.token,phase,message]).map_err(|e|e.to_string())?;
    result.map(|_|true)
}
pub fn recover(root:&Path)->Result<(),String>{
    db(root)?.execute("UPDATE stop_backup SET phase='error',last_result='Error: Copia interrumpida al cerrar la aplicación; no se repite esta parada' WHERE phase='copying'",[]).map_err(|e|e.to_string())?;Ok(())
}
#[cfg(test)]mod tests{
    use super::*;use crate::backup;use std::{fs,sync::{Arc,Mutex},collections::HashMap};
    #[test]fn opt_in_clean_exit_creates_one_real_copy_and_never_replays(){
        let root=std::env::temp_dir().join(format!("tch-stop-copy-{}",uuid::Uuid::new_v4()));
        let server=storage::create(&root,crate::model::CreateServer{name:"Stop".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        fs::write(Path::new(&server.directory).join("keep.bin"),vec![7u8;300000]).unwrap();let jobs=Arc::new(Mutex::new(HashMap::new()));
        let mut exit=Exit{token:uuid::Uuid::new_v4().to_string(),unexpected:false,successful:true,was_running:true};
        assert!(!status(&root,&server.id).unwrap().enabled);assert!(!run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());
        configure(&root,&server.id,true,Some(&exit.token)).unwrap();assert!(!pending(&root,&server.id,&exit).unwrap());exit.token=uuid::Uuid::new_v4().to_string();
        exit.was_running=false;assert!(!run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());exit.was_running=true;exit.successful=false;assert!(!run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());exit.successful=true;
        assert!(run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());let copies=backup::list_details(&root,&server.id).unwrap();assert_eq!(copies.len(),1);assert_eq!(copies[0].reason,Some(backup::Reason::ServerStop));assert_eq!(fs::read(Path::new(&copies[0].backup.path).join("keep.bin")).unwrap(),vec![7u8;300000]);assert_eq!(status(&root,&server.id).unwrap().phase,"completed");assert!(!run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());recover(&root).unwrap();assert!(!pending(&root,&server.id,&exit).unwrap());
        exit.token=uuid::Uuid::new_v4().to_string();let cancel_jobs=jobs.clone();assert_eq!(run(&root,&server.id,&exit,&jobs,move|event|{if event.stage=="copying"&&event.received>0{cancel_jobs.lock().unwrap()[&event.server_id].request_cancel();}}).unwrap_err(),crate::download::CANCELLED);assert_eq!(status(&root,&server.id).unwrap().phase,"cancelled");assert!(!pending(&root,&server.id,&exit).unwrap());assert_eq!(storage::list_backups(&root,&server.id).unwrap().len(),1);assert!(jobs.lock().unwrap().is_empty());
        exit.token=uuid::Uuid::new_v4().to_string();db(&root).unwrap().execute("UPDATE stop_backup SET phase='copying',last_token=?2 WHERE server_id=?1",params![server.id,exit.token]).unwrap();assert!(configure(&root,&server.id,false,None).is_err());recover(&root).unwrap();assert_eq!(status(&root,&server.id).unwrap().phase,"error");assert!(!pending(&root,&server.id,&exit).unwrap());
        configure(&root,&server.id,false,Some(&exit.token)).unwrap();assert!(!status(&root,&server.id).unwrap().enabled);fs::remove_dir_all(root).unwrap();
    }
    #[test]fn database_failure_keeps_original_files_and_records_a_non_replaying_failure(){
        let root=std::env::temp_dir().join(format!("tch-stop-copy-failure-{}",uuid::Uuid::new_v4()));
        let server=storage::create(&root,crate::model::CreateServer{name:"Stop".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        let file=Path::new(&server.directory).join("keep.bin");fs::write(&file,b"original world bytes").unwrap();let jobs=Arc::new(Mutex::new(HashMap::new()));
        configure(&root,&server.id,true,None).unwrap();let mut exit=Exit{token:uuid::Uuid::new_v4().to_string(),unexpected:false,successful:true,was_running:true};
        db(&root).unwrap().execute_batch("CREATE TRIGGER reject_stop_backup BEFORE INSERT ON backups BEGIN SELECT RAISE(ABORT,'test database failure'); END;").unwrap();
        assert!(run(&root,&server.id,&exit,&jobs,|_|{}).is_err());let saved=status(&root,&server.id).unwrap();assert_eq!(saved.phase,"error");assert!(saved.last_result.unwrap().starts_with("Error:"));assert_eq!(fs::read(&file).unwrap(),b"original world bytes");assert!(storage::list_backups(&root,&server.id).unwrap().is_empty());assert!(jobs.lock().unwrap().is_empty());recover(&root).unwrap();assert!(!pending(&root,&server.id,&exit).unwrap());
        db(&root).unwrap().execute_batch("DROP TRIGGER reject_stop_backup;").unwrap();exit.token=uuid::Uuid::new_v4().to_string();assert!(run(&root,&server.id,&exit,&jobs,|_|{}).unwrap());assert_eq!(storage::list_backups(&root,&server.id).unwrap().len(),1);fs::remove_dir_all(root).unwrap();
    }

}
