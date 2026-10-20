use crate::storage;
use rusqlite::{params,OptionalExtension,TransactionBehavior};
use serde::{Deserialize,Serialize};
use std::path::Path;

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Policy {pub enabled:bool,pub delay_seconds:u32,pub max_attempts:u32}
impl Default for Policy {fn default()->Self{Self{enabled:false,delay_seconds:30,max_attempts:3}}}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status {pub policy:Policy,pub attempts:u32,pub next_at:Option<i64>,pub last_result:Option<String>}
pub struct Claim {pub server_id:String,pub token:String,pub attempt:u32}
fn connect(root:&Path)->Result<rusqlite::Connection,String> {
    let db=storage::connect(root)?;db.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e|e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS crash_restart (
        server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,
        enabled INTEGER NOT NULL DEFAULT 0, delay_seconds INTEGER NOT NULL DEFAULT 30,
        max_attempts INTEGER NOT NULL DEFAULT 3, attempts INTEGER NOT NULL DEFAULT 0,
        last_token TEXT, next_at INTEGER, last_result TEXT
    );").map_err(|e|e.to_string())?;Ok(db)
}
pub fn status(root:&Path,id:&str)->Result<Status,String> {
    storage::get(root,id)?;
    connect(root)?.query_row("SELECT enabled,delay_seconds,max_attempts,attempts,next_at,last_result FROM crash_restart WHERE server_id=?1",[id],|row|Ok(Status{policy:Policy{enabled:row.get(0)?,delay_seconds:row.get(1)?,max_attempts:row.get(2)?},attempts:row.get(3)?,next_at:row.get(4)?,last_result:row.get(5)?})).optional().map_err(|e|e.to_string()).map(|value|value.unwrap_or(Status{policy:Policy::default(),attempts:0,next_at:None,last_result:None}))
}
pub fn save(root:&Path,id:&str,policy:&Policy,current_exit:Option<&str>)->Result<(),String> {
    storage::get(root,id)?;
    if !(5..=3600).contains(&policy.delay_seconds)||!(1..=10).contains(&policy.max_attempts){return Err("Elige entre 5 y 3600 segundos de espera y entre 1 y 10 intentos.".into());}
    connect(root)?.execute("INSERT INTO crash_restart(server_id,enabled,delay_seconds,max_attempts,last_token) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(server_id) DO UPDATE SET enabled=excluded.enabled,delay_seconds=excluded.delay_seconds,max_attempts=excluded.max_attempts,attempts=0,last_token=excluded.last_token,next_at=NULL,last_result=NULL",params![id,policy.enabled,policy.delay_seconds,policy.max_attempts,current_exit]).map_err(|e|e.to_string())?;Ok(())
}
// A process exit token identifies one actual exit, not a polling observation.
pub fn observe(root:&Path,id:&str,token:&str,now:i64)->Result<bool,String> {
    let changed=connect(root)?.execute("UPDATE crash_restart SET last_token=?2,next_at=CASE WHEN attempts<max_attempts THEN ?3+delay_seconds*1000 ELSE NULL END,last_result=CASE WHEN attempts<max_attempts THEN 'Fallo detectado; esperando para iniciar' ELSE 'Límite de intentos alcanzado; revisa la consola' END WHERE server_id=?1 AND enabled=1 AND (last_token IS NULL OR last_token<>?2)",params![id,token,now]).map_err(|e|e.to_string())?;Ok(changed==1)
}
pub fn claim(root:&Path,now:i64)->Result<Option<Claim>,String> {
    let mut db=connect(root)?;let tx=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    let claim=tx.query_row("SELECT server_id,last_token,attempts+1 FROM crash_restart WHERE enabled=1 AND next_at<=?1 AND attempts<max_attempts ORDER BY next_at,server_id LIMIT 1",[now],|row|Ok(Claim{server_id:row.get(0)?,token:row.get(1)?,attempt:row.get(2)?})).optional().map_err(|e|e.to_string())?;
    if let Some(claim)=&claim {tx.execute("UPDATE crash_restart SET attempts=attempts+1,next_at=NULL,last_result='Comprobando inicio automático' WHERE server_id=?1",[&claim.server_id]).map_err(|e|e.to_string())?;}
    tx.commit().map_err(|e|e.to_string())?;Ok(claim)
}
pub fn allowed(root:&Path,claim:&Claim)->Result<bool,String> {
    connect(root)?.query_row("SELECT EXISTS(SELECT 1 FROM crash_restart WHERE server_id=?1 AND enabled=1 AND last_token=?2 AND attempts=?3)",params![claim.server_id,claim.token,claim.attempt],|r|r.get(0)).map_err(|e|e.to_string())
}
pub fn cancel(root:&Path,claim:&Claim)->Result<(),String> {
    connect(root)?.execute("UPDATE crash_restart SET next_at=NULL,last_result='Reintento cancelado: el proceso o su configuración cambiaron' WHERE server_id=?1 AND last_token=?2 AND attempts=?3",params![claim.server_id,claim.token,claim.attempt]).map_err(|e|e.to_string())?;Ok(())
}
pub fn defer(root:&Path,claim:&Claim,now:i64)->Result<(),String> {
    connect(root)?.execute("UPDATE crash_restart SET attempts=attempts-1,next_at=?4+delay_seconds*1000,last_result='Esperando a que termine la operación actual' WHERE server_id=?1 AND enabled=1 AND last_token=?2 AND attempts=?3",params![claim.server_id,claim.token,claim.attempt,now]).map_err(|e|e.to_string())?;Ok(())
}
pub fn finish(root:&Path,claim:&Claim,result:Result<(),String>,now:i64)->Result<(),String> {
    let failed=result.is_err();let message=match result {Ok(())=>format!("Inicio automático solicitado (intento {})",claim.attempt),Err(error)=>format!("Error en intento {}: {error}",claim.attempt)};
    connect(root)?.execute("UPDATE crash_restart SET last_result=?4,next_at=CASE WHEN ?5=1 AND attempts<max_attempts AND enabled=1 THEN ?6+delay_seconds*1000 ELSE NULL END WHERE server_id=?1 AND last_token=?2 AND attempts=?3",params![claim.server_id,claim.token,claim.attempt,message,failed,now]).map_err(|e|e.to_string())?;Ok(())
}
pub fn active(root:&Path,id:&str,healthy:bool)->Result<bool,String> {
    let changed=connect(root)?.execute("UPDATE crash_restart SET next_at=NULL,attempts=CASE WHEN ?2 THEN 0 ELSE attempts END,last_result=CASE WHEN ?2 THEN 'Servidor estable durante 10 minutos; intentos restablecidos' ELSE 'Reintento cancelado: el servidor ya está activo' END WHERE server_id=?1 AND (next_at IS NOT NULL OR (?2 AND attempts>0))",params![id,healthy]).map_err(|e|e.to_string())?;Ok(changed==1)
}
pub fn recover(root:&Path)->Result<(),String> {
    connect(root)?.execute("UPDATE crash_restart SET next_at=NULL,last_result='Reintento cancelado al cerrar la aplicación' WHERE next_at IS NOT NULL OR last_result='Comprobando inicio automático'",[]).map_err(|e|e.to_string())?;Ok(())
}
