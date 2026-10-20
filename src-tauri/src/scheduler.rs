use crate::storage;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

// A worker owns its capacity until all execution/result reporting has finished.
// Drop also releases it when the worker unwinds unexpectedly.
pub struct WorkerPermit(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl WorkerPermit {
    pub fn new(workers:std::sync::Arc<std::sync::atomic::AtomicUsize>)->Self {
        workers.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Self(workers)
    }
}
impl Drop for WorkerPermit {
    fn drop(&mut self){self.0.fetch_sub(1,std::sync::atomic::Ordering::SeqCst);}
}
pub fn guarded<T>(work:impl FnOnce()->Result<T,String>)->Result<T,String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
        .unwrap_or_else(|_|Err("La tarea se interrumpió por un fallo interno. Revisa su resultado antes de volver a ejecutarla.".into()))
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub action: String,
    pub next_at: i64,
    pub interval_minutes: Option<u32>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub server_id: String,
    pub action: String,
    pub next_at: i64,
    pub interval_minutes: Option<u32>,
    pub enabled: bool,
    pub running: bool,
    pub last_at: Option<i64>,
    pub last_result: Option<String>,
    pub invalid: bool,
}

fn connect(root: &Path) -> Result<Connection, String> {
    let db = storage::connect(root)?;
    db.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS scheduled_tasks (
        id TEXT PRIMARY KEY,
        server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
        action TEXT NOT NULL CHECK(action IN ('backup','backup_restart','start','restart','stop')),
        next_at INTEGER NOT NULL, interval_minutes INTEGER,
        enabled INTEGER NOT NULL DEFAULT 1, running INTEGER NOT NULL DEFAULT 0,
        last_at INTEGER, last_result TEXT
    ); CREATE INDEX IF NOT EXISTS scheduled_due ON scheduled_tasks(enabled,running,next_at);")
        .map_err(|e| e.to_string())?;
    Ok(db)
}

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {id:row.get(0)?,server_id:row.get(1)?,action:row.get(2)?,next_at:row.get(3)?,interval_minutes:row.get(4)?,enabled:row.get(5)?,running:row.get(6)?,last_at:row.get(7)?,last_result:row.get(8)?,invalid:false})
}
const SELECT: &str = "SELECT id,server_id,action,next_at,interval_minutes,enabled,running,last_at,last_result FROM scheduled_tasks";

pub fn exclusive_owner(root:&Path)->Result<std::fs::File,String> {
    let path=root.join(".tch-instance.lock");
    if std::fs::symlink_metadata(&path).is_ok_and(|meta|meta.file_type().is_symlink()){return Err("El bloqueo de la biblioteca no puede ser un enlace".into());}
    let file=std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path).map_err(|e|e.to_string())?;
    file.try_lock().map_err(|_|"Esta biblioteca ya está abierta en otra instancia de TCHLauncher. Usa su ventana o ciérrala antes de abrirla de nuevo.".to_string())?;
    Ok(file)
}

pub fn list(root: &Path, server_id: &str) -> Result<Vec<Task>, String> {
    storage::get(root, server_id)?;
    let db=connect(root)?;
    let mut statement=db.prepare(&format!("{SELECT} WHERE server_id=?1 ORDER BY next_at,id")).map_err(|e|e.to_string())?;
    let rows=statement.query_map([server_id],|raw|{
        if let Ok(task)=row(raw){if valid_saved(&task){return Ok(task);}}
        Ok(Task{id:raw.get(0)?,server_id:server_id.into(),action:"invalid".into(),next_at:0,interval_minutes:None,enabled:raw.get(5)?,running:raw.get(6)?,last_at:None,last_result:Some("Error: La tarea guardada contiene datos no válidos. Quítala y programa otra.".into()),invalid:true})
    }).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

pub fn add(root: &Path, server_id: &str, input: Input, now: i64) -> Result<Task, String> {
    storage::get(root,server_id)?;
    if !["backup","backup_restart","start","restart","stop"].contains(&input.action.as_str()) || input.next_at<=now || input.next_at>now+5*366*24*60*60*1000 || input.interval_minutes.is_some_and(|v|!(1..=525600).contains(&v)) {
        return Err("Elige una acción, una fecha futura y un intervalo de 1 minuto a 1 año.".into());
    }
    let db=connect(root)?;
    let count:u32=db.query_row("SELECT count(*) FROM scheduled_tasks WHERE server_id=?1",[server_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if count>=100 {return Err("Este servidor ya tiene 100 tareas. Elimina alguna antes de añadir otra.".into());}
    let task=Task{id:Uuid::new_v4().to_string(),server_id:server_id.into(),action:input.action,next_at:input.next_at,interval_minutes:input.interval_minutes,enabled:true,running:false,last_at:None,last_result:None,invalid:false};
    db.execute("INSERT INTO scheduled_tasks(id,server_id,action,next_at,interval_minutes) VALUES(?1,?2,?3,?4,?5)",params![task.id,server_id,task.action,task.next_at,task.interval_minutes]).map_err(|e|e.to_string())?;
    Ok(task)
}

fn valid_saved(task:&Task)->bool {
    Uuid::parse_str(&task.id).is_ok()&&Uuid::parse_str(&task.server_id).is_ok()
        &&["backup","backup_restart","start","restart","stop"].contains(&task.action.as_str())
        &&(0..=8_640_000_000_000_000).contains(&task.next_at)
        &&task.interval_minutes.is_none_or(|minutes|(1..=525600).contains(&minutes))
}
fn following_occurrence(task:&Task,now:i64)->Option<i64>{
    if !valid_saved(task)||now<task.next_at{return None;}
    match task.interval_minutes {
        None=>Some(task.next_at),
        Some(minutes)=>{
            let interval=i64::from(minutes)*60000;
            let elapsed=now.checked_sub(task.next_at)?;
            let next=now.checked_add(interval-elapsed%interval)?;
            (next<=8_640_000_000_000_000).then_some(next)
        }
    }
}
pub fn set_enabled(root:&Path,server_id:&str,id:&str,enabled:bool,now:i64)->Result<(),String> {
    storage::get(root,server_id)?;
    let mut db=connect(root)?;let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    if enabled {
        let task=transaction.query_row(&format!("{SELECT} WHERE id=?1 AND server_id=?2"),params![id,server_id],row).optional().map_err(|_|"La tarea guardada no es válida. Quítala y programa otra.".to_string())?.ok_or("La tarea ya no existe")?;
        if !valid_saved(&task){return Err("La tarea guardada no es válida. Quítala y programa otra.".into());}
        if task.next_at<=now&&task.interval_minutes.is_some(){
            let next=now.checked_add(i64::from(task.interval_minutes.unwrap())*60000).filter(|next|*next<=8_640_000_000_000_000).ok_or("Fecha de tarea fuera de rango")?;
            transaction.execute("UPDATE scheduled_tasks SET next_at=?2 WHERE id=?1 AND running=0",params![id,next]).map_err(|e|e.to_string())?;
        }
    }
    let changed=transaction.execute("UPDATE scheduled_tasks SET enabled=?3 WHERE id=?1 AND server_id=?2 AND running=0 AND (?3=0 OR next_at>?4 OR interval_minutes IS NOT NULL)",params![id,server_id,enabled,now]).map_err(|e|e.to_string())?;
    if changed!=1 {return Err("La tarea está ejecutándose o su fecha ya pasó. Programa una fecha nueva.".into());}
    transaction.commit().map_err(|e|e.to_string())?;Ok(())
}
pub fn remove(root:&Path,server_id:&str,id:&str)->Result<(),String> {
    storage::get(root,server_id)?;
    if connect(root)?.execute("DELETE FROM scheduled_tasks WHERE id=?1 AND server_id=?2 AND running=0",params![id,server_id]).map_err(|e|e.to_string())?!=1 {return Err("La tarea ya no existe o está ejecutándose.".into());}Ok(())
}

// Claim one occurrence durably before acting. Missed interval ticks are skipped;
// a delayed task runs once instead of replaying every action after app downtime.
pub fn claim(root:&Path,now:i64)->Result<Option<Task>,String> {
    let mut db=connect(root)?;
    let transaction=db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    // A malformed occurrence must not stop the runner or starve valid tasks.
    let candidates={
        let mut statement=transaction.prepare(&format!("{SELECT} WHERE enabled=1 AND running=0 AND next_at<=?1 AND NOT EXISTS(SELECT 1 FROM scheduled_tasks active WHERE active.server_id=scheduled_tasks.server_id AND active.running=1) ORDER BY next_at,id LIMIT 100")).map_err(|e|e.to_string())?;
        let rows=statement.query_map([now],|raw|Ok((raw.get::<_,String>(0)?,row(raw)))).map_err(|e|e.to_string())?;
        rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?
    };
    let mut claimed=None;
    for (id,loaded) in candidates {
        let candidate=loaded.ok().and_then(|task|following_occurrence(&task,now).map(|next|(task,next)));
        if let Some((task,next))=candidate {
            transaction.execute("UPDATE scheduled_tasks SET running=1,last_at=?2,last_result='En ejecución',next_at=?3,enabled=?4 WHERE id=?1",params![task.id,now,next,task.interval_minutes.is_some()]).map_err(|e|e.to_string())?;
            claimed=Some(task);break;
        }
        transaction.execute("UPDATE scheduled_tasks SET enabled=0,last_result='Error: La tarea guardada contiene datos no válidos. Quítala y programa otra.' WHERE id=?1 AND running=0",[id]).map_err(|e|e.to_string())?;
    }
    transaction.commit().map_err(|e|e.to_string())?;
    Ok(claimed)
}

pub fn finish(root:&Path,id:&str,result:&str)->Result<(),String> {
    if connect(root)?.execute("UPDATE scheduled_tasks SET running=0,last_result=?2 WHERE id=?1 AND running=1",params![id,result]).map_err(|e|e.to_string())?!=1 {return Err("La tarea no está en ejecución.".into());}Ok(())
}
pub fn recover(root:&Path)->Result<(),String> {
    connect(root)?.execute("UPDATE scheduled_tasks SET running=0,last_result='Interrumpida al cerrar la aplicación; no se repite esta ejecución' WHERE running=1",[]).map_err(|e|e.to_string())?;Ok(())
}
