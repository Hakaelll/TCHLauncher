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
