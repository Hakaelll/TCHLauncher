use crate::{backup,model::Backup,storage};
use serde::{Serialize,Deserialize};
use sha2::{Digest,Sha512};
use std::{collections::HashSet,fs,io::Write,path::Path};
use rusqlite::OptionalExtension;
use uuid::Uuid;

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Policy {pub enabled:bool,pub max_count:Option<u32>,pub max_bytes:Option<u64>}
impl Default for Policy {fn default()->Self {Self{enabled:false,max_count:Some(10),max_bytes:Some(20*1024*1024*1024)}}}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Plan {pub fingerprint:String,pub remove:Vec<Backup>,pub before_bytes:u64,pub after_bytes:u64,pub warnings:Vec<String>}

fn validate(policy:&Policy)->Result<(),String> {
    if policy.max_count.is_some_and(|v|!(1..=1000).contains(&v)) || policy.max_bytes.is_some_and(|v|!(1024*1024..=2048*1024*1024*1024).contains(&v)) || (policy.enabled&&policy.max_count.is_none()&&policy.max_bytes.is_none()) {return Err("Revisa los límites de retención: 1–1000 copias y 1 MB–2048 GB.".into());}Ok(())
}
pub fn get(root:&Path,id:&str)->Result<Policy,String> {
    storage::get(root,id)?;
    let value:Option<String>=storage::connect(root)?.query_row("SELECT value FROM application_settings WHERE key=?1",[format!("backup-policy:{id}")],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    let policy=match value {Some(value)=>serde_json::from_str(&value).map_err(|_|"La política de copias no es válida")?,None=>Policy::default()};validate(&policy)?;Ok(policy)
}
pub fn save(root:&Path,id:&str,policy:&Policy)->Result<(),String> {
    storage::get(root,id)?;validate(policy)?;
    storage::connect(root)?.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",rusqlite::params![format!("backup-policy:{id}"),serde_json::to_string(policy).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;Ok(())
}

fn check_child(base:&Path,path:&Path)->Result<(),String> {
    let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    if !meta.is_dir() || meta.file_type().is_symlink() || path.canonicalize().map_err(|e|e.to_string())?.parent()!=Some(base.canonicalize().map_err(|e|e.to_string())?.as_path()) {return Err("La copia sale de la biblioteca o no es una carpeta normal".into());}Ok(())
}

pub fn preview(root:&Path,id:&str,protected:&[String])->Result<Plan,String> {
    let policy=get(root,id)?;let base=backup::backup_base(root,id)?;
    let mut copies=storage::list_backups(root,id)?;copies.sort_by(|a,b|b.created_at.cmp(&a.created_at).then_with(||b.id.cmp(&a.id)));
    let mut digest=Sha512::new();digest.update(serde_json::to_vec(&policy).map_err(|e|e.to_string())?);let mut total=0u64;
    for copy in &mut copies {
        Uuid::parse_str(&copy.id).map_err(|_|"Identificador de copia no válido")?;
        let expected=base.join(&copy.id);if Path::new(&copy.path)!=expected {return Err("La ruta de la copia no corresponde a la biblioteca".into());}check_child(&base,&expected)?;
        digest.update(copy.id.as_bytes());digest.update(copy.created_at.as_bytes());let mut size=0u64;
        let mut files=walkdir::WalkDir::new(&expected).into_iter().collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;files.sort_by_key(|entry|entry.path().to_owned());
        for entry in files {
            if entry.file_type().is_symlink(){return Err("La copia contiene enlaces simbólicos".into());}
            let meta=entry.metadata().map_err(|e|e.to_string())?;
            digest.update(entry.path().strip_prefix(&base).map_err(|e|e.to_string())?.to_string_lossy().as_bytes());digest.update(meta.len().to_le_bytes());
            let modified=meta.modified().map_err(|e|e.to_string())?.duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?;digest.update(modified.as_nanos().to_le_bytes());
            if meta.is_file(){size=size.checked_add(meta.len()).ok_or("Tamaño de copia demasiado grande")?;}
        }
        copy.size_bytes=size;total=total.checked_add(size).ok_or("Tamaño de copias demasiado grande")?;
    }
    let mut keep:HashSet<_>=protected.iter().cloned().collect();if let Some(newest)=copies.first(){keep.insert(newest.id.clone());}
    let before_bytes=total;let mut count=copies.len();let mut remove=vec![];
    let exceeds=|count:usize,size:u64|policy.max_count.is_some_and(|max|count>max as usize)||policy.max_bytes.is_some_and(|max|size>max);
    if policy.enabled {
        for copy in copies.into_iter().rev() {
            if !exceeds(count,total){break;}
            if keep.contains(&copy.id){continue;}
            count-=1;total-=copy.size_bytes;remove.push(copy);
        }
    }
    let warnings=if policy.enabled&&exceeds(count,total){vec!["Las copias protegidas superan el límite. Se conservarán la más reciente y las usadas por la operación actual.".into()]}else{vec![]};
    Ok(Plan {fingerprint:format!("{:x}",digest.finalize()),remove,before_bytes,after_bytes:total,warnings})
}

pub fn apply(root:&Path,id:&str,expected:Option<&str>,protected:&[String])->Result<Plan,String> {
    if expected.is_none()&&!get(root,id)?.enabled {return Ok(Plan{fingerprint:String::new(),remove:vec![],before_bytes:0,after_bytes:0,warnings:vec![]});}
    recover(root,id)?;let plan=preview(root,id,protected)?;
    if expected.is_some_and(|value|value!=plan.fingerprint){return Err("Las copias o la política cambiaron. Vuelve a revisar la limpieza.".into());}
    if plan.remove.is_empty(){return Ok(plan);}
    if plan.remove.len()>100000 {return Err("La limpieza supera el límite de copias por operación".into());}
    let base=backup::backup_base(root,id)?;let journal=base.join(".tch-retention.json");let pending=base.join(".tch-retention.pending");
    let ids=plan.remove.iter().map(|copy|copy.id.clone()).collect::<Vec<_>>();
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&pending).map_err(|e|e.to_string())?;
    file.write_all(&serde_json::to_vec(&ids).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);fs::rename(&pending,&journal).map_err(|e|e.to_string())?;
    let result=(||->Result<(),String>{
        for copy in &plan.remove {
            let source=base.join(&copy.id);check_child(&base,&source)?;let tomb=base.join(format!(".tch-prune-{}",copy.id));if tomb.exists(){return Err("Existe una limpieza pendiente para esta copia".into());}fs::rename(source,tomb).map_err(|e|e.to_string())?;
        }
        let mut db=storage::connect(root)?;let transaction=db.transaction().map_err(|e|e.to_string())?;
        for copy in &plan.remove {if transaction.execute("DELETE FROM backups WHERE id=?1 AND server_id=?2",rusqlite::params![copy.id,id]).map_err(|e|e.to_string())?!=1{return Err("La copia cambió durante la limpieza".into());}}
        transaction.commit().map_err(|e|e.to_string())?;Ok(())
    })();
    recover(root,id)?;result?;Ok(plan)
}

pub fn recover(root:&Path,id:&str)->Result<(),String> {
    let base=backup::backup_base(root,id)?;let journal=base.join(".tch-retention.json");let pending=base.join(".tch-retention.pending");
    for path in [&journal,&pending] {if fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()){return Err("El registro de limpieza no puede ser un enlace".into());}}
    if pending.exists(){fs::remove_file(pending).map_err(|e|e.to_string())?;}
    if !journal.exists(){return Ok(());}
    if fs::metadata(&journal).map_err(|e|e.to_string())?.len()>5_000_000{return Err("Registro de retención demasiado grande".into());}
    let ids:Vec<String>=serde_json::from_slice(&fs::read(&journal).map_err(|e|e.to_string())?).map_err(|_|"Registro de retención no válido")?;
    let mut unique=HashSet::new();for id in &ids {Uuid::parse_str(id).map_err(|_|"Registro de copia no válido")?;if !unique.insert(id){return Err("Registro de limpieza repetido".into());}}
    let db=storage::connect(root)?;
    for copy in ids {
        let source=base.join(&copy);let tomb=base.join(format!(".tch-prune-{copy}"));if !tomb.exists(){continue;}check_child(&base,&tomb)?;
        let retained:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM backups WHERE server_id=?1 AND id=?2)",rusqlite::params![id,copy],|r|r.get(0)).map_err(|e|e.to_string())?;
        if retained {if source.exists(){return Err("La copia original cambió durante la recuperación de limpieza".into());}fs::rename(tomb,source).map_err(|e|e.to_string())?;}else{fs::remove_dir_all(tomb).map_err(|e|e.to_string())?;}
    }
    fs::remove_file(journal).map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Library(std::path::PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,crate::model::Server) {
        let root=std::env::temp_dir().join(format!("tch-retention-{}",Uuid::new_v4()));
        let server=storage::create(&root,crate::model::CreateServer{name:"Retention".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        (Library(root),server)
    }
    fn count_policy(root:&Path,id:&str,count:u32){save(root,id,&Policy{enabled:true,max_count:Some(count),max_bytes:None}).unwrap();}
    #[test]
    fn default_keeps_copies_and_enabled_policy_prunes_on_creation() {
        let (root,server)=fixture();let mut ids=vec![];
        for _ in 0..3 {ids.push(backup::create(&root.0,&server.id).unwrap().id);}
        assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),3);
        count_policy(&root.0,&server.id,2);
        let latest=backup::create(&root.0,&server.id).unwrap();assert!(latest.retention_warning.is_none());
        let retained=storage::list_backups(&root.0,&server.id).unwrap();assert_eq!(retained.len(),2);assert!(retained.iter().any(|copy|copy.id==latest.id));
        for id in ids {assert_eq!(backup::backup_base(&root.0,&server.id).unwrap().join(&id).exists(),retained.iter().any(|copy|copy.id==id));}
    }
    #[test]
    fn space_limit_uses_actual_size_and_keeps_oversized_newest() {
        let (root,server)=fixture();let first=backup::create(&root.0,&server.id).unwrap();let second=backup::create(&root.0,&server.id).unwrap();
        storage::connect(&root.0).unwrap().execute("UPDATE backups SET created_at='2000-01-01' WHERE id=?1",[&first.id]).unwrap();
        fs::write(Path::new(&first.path).join("payload"),vec![1u8;600_000]).unwrap();fs::write(Path::new(&second.path).join("payload"),vec![2u8;1_100_000]).unwrap();
        save(&root.0,&server.id,&Policy{enabled:true,max_count:None,max_bytes:Some(1024*1024)}).unwrap();
        let plan=preview(&root.0,&server.id,&[]).unwrap();assert_eq!(plan.remove.len(),1);assert_eq!(plan.remove[0].id,first.id);assert!(plan.before_bytes>1_700_000);assert!(plan.after_bytes>1_100_000);assert_eq!(plan.warnings.len(),1);
        apply(&root.0,&server.id,Some(&plan.fingerprint),&[]).unwrap();assert!(Path::new(&second.path).exists());assert!(!Path::new(&first.path).exists());
    }
    #[test]
    fn changed_preview_rejects_deletion_and_database_failure_rolls_back() {
        let (root,server)=fixture();let first=backup::create(&root.0,&server.id).unwrap();backup::create(&root.0,&server.id).unwrap();count_policy(&root.0,&server.id,1);
        let plan=preview(&root.0,&server.id,&[]).unwrap();fs::write(Path::new(&first.path).join("changed"),b"change").unwrap();
        assert!(apply(&root.0,&server.id,Some(&plan.fingerprint),&[]).is_err());assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),2);
        storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER fail_prune BEFORE DELETE ON backups BEGIN SELECT RAISE(ABORT,'forced failure'); END;").unwrap();
        assert!(apply(&root.0,&server.id,None,&[]).is_err());
        for copy in storage::list_backups(&root.0,&server.id).unwrap(){assert!(Path::new(&copy.path).is_dir());}
        assert!(!backup::backup_base(&root.0,&server.id).unwrap().join(".tch-retention.json").exists());
        let saved=backup::create(&root.0,&server.id).unwrap();assert!(saved.retention_warning.is_some());assert!(Path::new(&saved.path).is_dir());assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),3);
    }
    #[test]
    fn interrupted_cleanup_recovers_before_and_after_database_commit() {
        for committed in [false,true] {
            let (root,server)=fixture();let copy=backup::create(&root.0,&server.id).unwrap();let base=backup::backup_base(&root.0,&server.id).unwrap();let tomb=base.join(format!(".tch-prune-{}",copy.id));
            fs::write(base.join(".tch-retention.json"),serde_json::to_vec(&vec![copy.id.clone()]).unwrap()).unwrap();fs::rename(&copy.path,&tomb).unwrap();
            if committed {storage::connect(&root.0).unwrap().execute("DELETE FROM backups WHERE id=?1",[&copy.id]).unwrap();}
            recover(&root.0,&server.id).unwrap();assert_eq!(Path::new(&copy.path).exists(),!committed);assert!(!tomb.exists());assert!(!base.join(".tch-retention.json").exists());
        }
    }
    #[test]
    fn restore_source_survives_retention_during_safety_copy() {
        let (root,server)=fixture();let file=Path::new(&server.directory).join("value.txt");fs::write(&file,b"original").unwrap();let source=backup::create(&root.0,&server.id).unwrap();
        fs::write(&file,b"current").unwrap();backup::create(&root.0,&server.id).unwrap();count_policy(&root.0,&server.id,1);
        backup::restore(&root.0,&server.id,&source.id).unwrap();assert_eq!(fs::read(&file).unwrap(),b"original");assert!(Path::new(&source.path).exists());assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),2);
    }
    #[test]
    fn foreign_copy_path_and_invalid_journal_never_delete_files() {
        let (root,server)=fixture();let copy=backup::create(&root.0,&server.id).unwrap();let foreign=root.0.join("foreign");fs::create_dir(&foreign).unwrap();fs::write(foreign.join("keep"),b"keep").unwrap();count_policy(&root.0,&server.id,1);
        storage::connect(&root.0).unwrap().execute("UPDATE backups SET path=?2 WHERE id=?1",rusqlite::params![copy.id,foreign.to_string_lossy()]).unwrap();assert!(preview(&root.0,&server.id,&[]).is_err());assert!(foreign.join("keep").exists());
        let base=backup::backup_base(&root.0,&server.id).unwrap();fs::write(base.join(".tch-retention.json"),b"[\"../foreign\"]").unwrap();assert!(recover(&root.0,&server.id).is_err());assert!(foreign.join("keep").exists());assert!(Path::new(&copy.path).exists());
    }
}
