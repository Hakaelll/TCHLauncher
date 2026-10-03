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

#[cfg(test)]
mod tests {
    use super::*;
    struct Library(std::path::PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,String) {
        let root=std::env::temp_dir().join(format!("tch-scheduler-{}",Uuid::new_v4()));
        let server=storage::create(&root,crate::model::CreateServer{name:"Scheduled".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();(Library(root),server.id)
    }
    #[test]
    fn invalid_occurrences_are_disabled_without_blocking_valid_tasks(){
        let (root,id)=fixture();let zero=add(&root.0,&id,Input{action:"backup".into(),next_at:100,interval_minutes:Some(1)},0).unwrap();let negative=add(&root.0,&id,Input{action:"restart".into(),next_at:200,interval_minutes:Some(1)},0).unwrap();let bad_date=add(&root.0,&id,Input{action:"stop".into(),next_at:300,interval_minutes:None},0).unwrap();let healthy=add(&root.0,&id,Input{action:"start".into(),next_at:1000,interval_minutes:None},0).unwrap();
        let db=connect(&root.0).unwrap();db.execute("UPDATE scheduled_tasks SET interval_minutes=0 WHERE id=?1",[&zero.id]).unwrap();db.execute("UPDATE scheduled_tasks SET interval_minutes=-1 WHERE id=?1",[&negative.id]).unwrap();db.execute("UPDATE scheduled_tasks SET next_at=-1 WHERE id=?1",[&bad_date.id]).unwrap();
        assert_eq!(list(&root.0,&id).unwrap().iter().filter(|task|task.invalid).count(),3);
        assert_eq!(claim(&root.0,1000).unwrap().unwrap().id,healthy.id);
        for task in [&zero,&negative,&bad_date]{
            let saved=list(&root.0,&id).unwrap().into_iter().find(|saved|saved.id==task.id).unwrap();assert!(saved.invalid);assert!(!saved.running);assert!(!saved.enabled);assert!(saved.last_result.unwrap().contains("datos no válidos"));
            assert!(set_enabled(&root.0,&id,&task.id,true,2000).is_err());
            remove(&root.0,&id,&task.id).unwrap();
        }
        finish(&root.0,&healthy.id,"Inicio solicitado").unwrap();assert!(claim(&root.0,999999).unwrap().is_none());assert_eq!(list(&root.0,&id).unwrap().len(),1);
    }
    #[test]
    fn recurrence_is_checked_at_date_limits_and_skips_missed_ticks(){
        let (root,id)=fixture();let mut task=add(&root.0,&id,Input{action:"backup".into(),next_at:1000,interval_minutes:Some(1)},0).unwrap();
        assert_eq!(following_occurrence(&task,181000),Some(241000));assert_eq!(following_occurrence(&task,180999),Some(181000));assert!(following_occurrence(&task,i64::MAX).is_none());
        task.interval_minutes=Some(0);assert!(following_occurrence(&task,1000).is_none());task.interval_minutes=Some(u32::MAX);assert!(following_occurrence(&task,1000).is_none());task.interval_minutes=Some(1);task.next_at=i64::MIN;assert!(following_occurrence(&task,1000).is_none());
    }
    #[test]
    fn worker_failure_is_persisted_releases_capacity_and_does_not_replay(){
        let (root,id)=fixture();let task=add(&root.0,&id,Input{action:"restart".into(),next_at:1000,interval_minutes:None},0).unwrap();
        let claimed=claim(&root.0,1000).unwrap().unwrap();let workers=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let path=root.0.clone();let count=workers.clone();let worker=WorkerPermit::new(count.clone());
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst),1);
        std::thread::spawn(move||{
            let _permit=worker;
            let result=guarded(||->Result<String,String>{panic!("test worker failure")}).unwrap_or_else(|error|format!("Error: {error}"));
            finish(&path,&claimed.id,&result).unwrap();
        }).join().unwrap();
        assert_eq!(workers.load(std::sync::atomic::Ordering::SeqCst),0);
        let saved=list(&root.0,&id).unwrap().remove(0);assert_eq!(saved.id,task.id);assert!(!saved.running);assert!(!saved.enabled);assert!(saved.last_result.unwrap().contains("fallo interno"));assert!(claim(&root.0,999999).unwrap().is_none());
        let count=workers.clone();assert!(std::thread::spawn(move||{let _permit=WorkerPermit::new(count);panic!("reporting failure")}).join().is_err());assert_eq!(workers.load(std::sync::atomic::Ordering::SeqCst),0);
        assert_eq!(guarded(||Ok::<_,String>("completed")).unwrap(),"completed");assert_eq!(guarded(||Err::<(),_>("expected error".into())).unwrap_err(),"expected error");
    }
    #[test]
    fn periodic_claim_is_durable_skips_missed_ticks_and_records_failure() {
        let (root,id)=fixture();let task=add(&root.0,&id,Input{action:"backup".into(),next_at:1000,interval_minutes:Some(1)},0).unwrap();
        assert!(claim(&root.0,999).unwrap().is_none());assert_eq!(claim(&root.0,181000).unwrap().unwrap().id,task.id);assert!(claim(&root.0,181000).unwrap().is_none());
        let saved=list(&root.0,&id).unwrap().remove(0);assert_eq!(saved.next_at,241000);assert!(saved.running);
        finish(&root.0,&task.id,"Error: ocupado").unwrap();assert!(claim(&root.0,181000).unwrap().is_none());assert_eq!(list(&root.0,&id).unwrap()[0].last_result.as_deref(),Some("Error: ocupado"));
        assert!(claim(&root.0,241000).unwrap().is_some());
    }
    #[test]
    fn interrupted_once_task_never_replays_and_can_be_removed() {
        let (root,id)=fixture();let task=add(&root.0,&id,Input{action:"restart".into(),next_at:1000,interval_minutes:None},0).unwrap();claim(&root.0,1000).unwrap();
        assert!(remove(&root.0,&id,&task.id).is_err());recover(&root.0).unwrap();assert!(claim(&root.0,999999).unwrap().is_none());let saved=list(&root.0,&id).unwrap();assert!(!saved[0].enabled);assert!(!saved[0].running);assert!(saved[0].last_result.as_ref().unwrap().contains("Interrumpida"));remove(&root.0,&id,&task.id).unwrap();assert!(list(&root.0,&id).unwrap().is_empty());
    }
    #[test]
    fn paused_tasks_validation_and_server_ownership() {
        let (root,id)=fixture();for input in [Input{action:"kill".into(),next_at:1000,interval_minutes:None},Input{action:"stop".into(),next_at:0,interval_minutes:None},Input{action:"start".into(),next_at:1000,interval_minutes:Some(0)}] {assert!(add(&root.0,&id,input,0).is_err());}
        let task=add(&root.0,&id,Input{action:"stop".into(),next_at:1000,interval_minutes:None},0).unwrap();set_enabled(&root.0,&id,&task.id,false,0).unwrap();assert!(claim(&root.0,2000).unwrap().is_none());assert!(set_enabled(&root.0,&id,&task.id,true,2000).is_err());assert!(remove(&root.0,"foreign",&task.id).is_err());
        storage::connect(&root.0).unwrap().execute("DELETE FROM servers WHERE id=?1",[&id]).unwrap();assert!(claim(&root.0,2000).unwrap().is_none());
    }
    #[test]
    fn concurrent_claims_serialize_and_periodic_tasks_resume_in_the_future() {
        let (root,id)=fixture();let task=add(&root.0,&id,Input{action:"backup".into(),next_at:1000,interval_minutes:Some(1)},0).unwrap();
        set_enabled(&root.0,&id,&task.id,false,0).unwrap();set_enabled(&root.0,&id,&task.id,true,2000).unwrap();assert_eq!(list(&root.0,&id).unwrap()[0].next_at,62000);
        let workers=(0..4).map(|_|{let path=root.0.clone();std::thread::spawn(move||claim(&path,62000).unwrap())}).collect::<Vec<_>>();
        let claimed=workers.into_iter().filter_map(|worker|worker.join().unwrap()).collect::<Vec<_>>();assert_eq!(claimed.len(),1);assert_eq!(claimed[0].id,task.id);
    }
    #[test]
    fn ownership_lock_prevents_second_runner_and_releases_on_close() {
        let (root,_)=fixture();let owner=exclusive_owner(&root.0).unwrap();assert!(exclusive_owner(&root.0).is_err());drop(owner);assert!(exclusive_owner(&root.0).is_ok());
    }
    #[test]
    fn due_backup_persists_files_and_applies_saved_retention() {
        let (root,id)=fixture();let server=storage::get(&root.0,&id).unwrap();let world=std::path::Path::new(&server.directory).join("world");std::fs::create_dir(&world).unwrap();std::fs::write(world.join("level.dat"),b"saved world").unwrap();
        crate::backup::create(&root.0,&id).unwrap();crate::backup_retention::save(&root.0,&id,&crate::backup_retention::Policy{enabled:true,max_count:Some(1),max_bytes:None}).unwrap();
        add(&root.0,&id,Input{action:"backup".into(),next_at:1000,interval_minutes:Some(1)},0).unwrap();let due=claim(&root.0,1000).unwrap().unwrap();let copy=crate::backup::create(&root.0,&due.server_id).unwrap();finish(&root.0,&due.id,"Copia guardada").unwrap();
        assert_eq!(std::fs::read(std::path::Path::new(&copy.path).join("world/level.dat")).unwrap(),b"saved world");assert_eq!(storage::list_backups(&root.0,&id).unwrap().len(),1);assert_eq!(list(&root.0,&id).unwrap()[0].last_result.as_deref(),Some("Copia guardada"));
    }
}
