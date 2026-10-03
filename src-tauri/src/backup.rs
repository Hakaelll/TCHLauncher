use crate::model::Backup;
use crate::storage;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;
use serde::{Serialize,Deserialize};

const SNAPSHOT:&str=".tch-backup.json";
#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Reason{Manual,Scheduled,ServerStop,BeforeRestore,BeforeMinecraftUpdate,BeforeLoaderUpdate,BeforeContentUpdate,BeforeContentRemoval,BeforeWorldChange,BeforeWorldDeletion}
impl Reason{
    fn for_task(kind:&str)->Self{match kind{
        "restore"=>Self::BeforeRestore,"minecraft_update"=>Self::BeforeMinecraftUpdate,"loader_update"=>Self::BeforeLoaderUpdate,
        "content_install"=>Self::BeforeContentUpdate,"content_remove"=>Self::BeforeContentRemoval,
        "world_activate"=>Self::BeforeWorldChange,"world_delete"=>Self::BeforeWorldDeletion,_=>Self::Manual,
    }}
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Details{
    #[serde(flatten)]pub backup:Backup,
    pub reason:Option<Reason>,pub minecraft_version:Option<String>,pub platform:Option<String>,pub metadata_error:Option<String>,
}
pub fn list_details(root:&Path,id:&str)->Result<Vec<Details>,String>{
    storage::get(root,id)?;let copies=storage::list_backups(root,id)?;if copies.is_empty(){return Ok(Vec::new());}
    let base=backup_base(root,id).and_then(|base|base.canonicalize().map_err(|error|error.to_string()));
    Ok(copies.into_iter().map(|copy|{
        let metadata=(||->Result<Option<Snapshot>,String>{
            let base=base.as_ref().map_err(Clone::clone)?;Uuid::parse_str(&copy.id).map_err(|_|"La copia no tiene un identificador válido")?;
            let path=Path::new(&copy.path);let meta=fs::symlink_metadata(path).map_err(|_|"La carpeta de la copia no está disponible")?;
            if !meta.is_dir()||meta.file_type().is_symlink()||path.canonicalize().map_err(|e|e.to_string())?!=base.join(&copy.id){return Err("La carpeta de la copia no coincide con la biblioteca".into());}
            let path=path.join(SNAPSHOT);let meta=match fs::symlink_metadata(&path){Ok(meta)=>meta,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string())};
            if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>32768{return Err("Los metadatos de la copia no son válidos".into());}
            use std::io::Read;let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(32769).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
            if bytes.len()>32768{return Err("Los metadatos de la copia son demasiado grandes".into());}
            let metadata:Snapshot=serde_json::from_slice(&bytes).map_err(|_|"No se pudieron leer los metadatos de la copia")?;validate_snapshot(&metadata)?;Ok(Some(metadata))
        })();
        match metadata{
            Ok(Some(metadata))=>Details{backup:copy,reason:metadata.reason,minecraft_version:Some(metadata.version),platform:Some(metadata.platform),metadata_error:None},
            Ok(None)=>Details{backup:copy,reason:None,minecraft_version:None,platform:None,metadata_error:None},
            Err(error)=>Details{backup:copy,reason:None,minecraft_version:None,platform:None,metadata_error:Some(error)},
        }
    }).collect())
}


#[derive(Serialize,Deserialize)]
struct Snapshot {
    format:u32,version:String,platform:String,jar:Option<String>,
    #[serde(default)]reason:Option<Reason>,
    memory_min_mb:u32,memory_max_mb:u32,storage_target_gb:u32,jvm:crate::jvm::Settings,
}

fn relative_jar(server:&crate::model::Server)->Result<Option<String>,String> {
    let Some(jar)=&server.jar_path else {return Ok(None);};
    if !Path::new(jar).is_file(){return Ok(None);}
    let root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let jar=Path::new(jar).canonicalize().map_err(|e|e.to_string())?;
    let relative=jar.strip_prefix(root).map_err(|_|"El archivo de arranque está fuera del servidor")?.to_string_lossy().replace('\\',"/");
    crate::modpack::relative_path(&relative)?;Ok(Some(relative))
}

fn snapshot(root:&Path,server:&crate::model::Server)->Result<Snapshot,String> {
    Ok(Snapshot {format:1,reason:None,version:server.version.clone(),platform:server.platform.clone(),jar:relative_jar(server)?,memory_min_mb:server.memory_min_mb,memory_max_mb:server.memory_max_mb,storage_target_gb:server.storage_target_gb,jvm:crate::jvm::get(root,&server.id)?})
}

fn validate_snapshot(value:&Snapshot)->Result<(),String> {
    crate::fabric::validate_version(&value.version)?;
    if value.format!=1 || !matches!(value.platform.as_str(),"vanilla"|"paper"|"fabric"|"neoforge") || value.memory_min_mb<512 || value.memory_min_mb>value.memory_max_mb || value.memory_max_mb>65536 || !(1..=2048).contains(&value.storage_target_gb) {return Err("Configuración del backup no válida".into());}
    if let Some(jar)=&value.jar {crate::modpack::relative_path(jar)?;}
    crate::jvm::options(&value.jvm)?;Ok(())
}

fn read_snapshot(source:&Path,fallback:Snapshot)->Result<Snapshot,String> {
    let path=source.join(SNAPSHOT);
    if !path.exists(){
        let mut fallback=fallback;
        if fallback.jar.is_some() {
            let inspected=storage::inspect_import(source)?;
            fallback.version=inspected.detected_version.ok_or("Esta copia antigua no identifica la versión de Minecraft. Importa su carpeta para revisar el motor antes de usarla.")?;
        }
        return Ok(fallback);
    }
    let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>32768 {return Err("Metadatos del backup no válidos".into());}
    let value:Snapshot=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|_|"No se pudieron leer los metadatos del backup")?;
    validate_snapshot(&value)?;Ok(value)
}

pub fn size(path: &Path) -> u64 {
    WalkDir::new(path).into_iter().filter_map(Result::ok)
        .filter_map(|e| if e.file_type().is_file() { e.metadata().ok().map(|m| m.len()) } else { None })
        .sum()
}

pub(crate) fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    for entry in WalkDir::new(from).into_iter() {
        let entry = entry.map_err(|e| e.to_string())?;
        let relative = entry.path().strip_prefix(from).map_err(|e| e.to_string())?;
        let dest = to.join(relative);
        if entry.file_type().is_symlink() { return Err("Los enlaces simbólicos no se copian por seguridad".into()); }
        if entry.file_type().is_dir() { fs::create_dir_all(dest).map_err(|e| e.to_string())?; }
        else if entry.file_type().is_file() { fs::copy(entry.path(),dest).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

pub(crate) fn copy_tree_tracked(from:&Path,to:&Path,task:&crate::download::DownloadTask,label:&str)->Result<(),String>{
    use std::io::{Read,Write};
    let mut total=0u64;
    for entry in WalkDir::new(from){
        task.cancellation.check()?;
        let entry=entry.map_err(|e|e.to_string())?;
        if entry.file_type().is_symlink(){return Err("Los enlaces simbólicos no se copian por seguridad".into());}
        if entry.file_type().is_file(){total=total.checked_add(entry.metadata().map_err(|e|e.to_string())?.len()).ok_or("La copia supera el tamaño admitido")?;}
    }
    let mut received=0u64;let began=std::time::Instant::now();let mut last=began;
    let expected=if total>0{Some(total)}else{None};
    task.progress(label,"copying",0,expected,None,1,None);
    let mut buffer=vec![0u8;256*1024];
    for entry in WalkDir::new(from){
        task.cancellation.check()?;
        let entry=entry.map_err(|e|e.to_string())?;
        if entry.file_type().is_symlink(){return Err("Los enlaces simbólicos no se copian por seguridad".into());}
        let relative=entry.path().strip_prefix(from).map_err(|e|e.to_string())?;let dest=to.join(relative);
        if entry.file_type().is_dir(){fs::create_dir_all(dest).map_err(|e|e.to_string())?;}
        else if entry.file_type().is_file(){
            let mut source=fs::File::open(entry.path()).map_err(|e|e.to_string())?;
            let mut target=fs::File::create(&dest).map_err(|e|e.to_string())?;
            loop{
                task.cancellation.check()?;
                let count=source.read(&mut buffer).map_err(|e|e.to_string())?;if count==0{break;}
                target.write_all(&buffer[..count]).map_err(|e|e.to_string())?;
                received=received.checked_add(count as u64).ok_or("La copia supera el tamaño admitido")?;
                if received>total{return Err("Los archivos cambiaron durante la copia. Vuelve a intentarlo.".into());}
                if last.elapsed()>=std::time::Duration::from_millis(100){task.progress(label,"copying",received,expected,Some(received as f64/began.elapsed().as_secs_f64()),1,None);last=std::time::Instant::now();}
            }
            target.sync_all().map_err(|e|e.to_string())?;
            fs::set_permissions(&dest,source.metadata().map_err(|e|e.to_string())?.permissions()).map_err(|e|e.to_string())?;
        }
    }
    if received!=total{return Err("Los archivos cambiaron durante la copia. Vuelve a intentarlo.".into());}
    task.progress(label,"copying",received,expected,None,1,None);
    Ok(())
}

pub fn create(root: &Path, server_id: &str) -> Result<Backup, String> {
    create_preserving(root,server_id,&[],None,Reason::Manual)
}

pub fn create_tracked(root:&Path,server_id:&str,task:&crate::download::DownloadTask)->Result<Backup,String>{
    create_preserving(root,server_id,&[],Some(task),Reason::for_task(task.kind()))
}

pub fn create_for(root:&Path,id:&str,reason:Reason)->Result<Backup,String>{create_preserving(root,id,&[],None,reason)}
pub fn create_tracked_for(root:&Path,id:&str,task:&crate::download::DownloadTask,reason:Reason)->Result<Backup,String>{create_preserving(root,id,&[],Some(task),reason)}
fn create_preserving(root:&Path,server_id:&str,protected:&[String],task:Option<&crate::download::DownloadTask>,reason:Reason)->Result<Backup,String> {
    if let Some(task)=task{task.cancellation.check()?;task.report("Revisando archivos de la copia","preparing");}
    crate::backup_retention::recover(root,server_id)?;
    let server = storage::get(root,server_id)?;
    let mut metadata=snapshot(root,&server)?;metadata.reason=Some(reason);
    validate_snapshot(&metadata)?;
    let source = Path::new(&server.directory).canonicalize().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let base = backup_base(root,server_id)?;
    if base.canonicalize().map_err(|e| e.to_string())?.starts_with(&source) {
        return Err("La carpeta de backups está dentro del servidor; elige otra ubicación para el servidor".into());
    }
    let stage = base.join(format!(".{id}.part"));
    let target = base.join(&id);
    let result:Result<Backup,String> = (|| {
        if let Some(task)=task{copy_tree_tracked(&source,&stage,task,if task.is_restore(){"Copia de seguridad antes de restaurar"}else{"Copiando archivos del servidor"})?;task.begin_publication("Guardando copia y aplicando retención")?;}else{copy_tree(&source,&stage)?;}
        fs::write(stage.join(SNAPSHOT),serde_json::to_vec_pretty(&metadata).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        fs::rename(&stage,&target).map_err(|e| e.to_string())?;
        let backup = Backup {id,server_id:server_id.into(),created_at:chrono::Utc::now().to_rfc3339(),size_bytes:size(&target),path:target.to_string_lossy().into(),retention_warning:None};
        storage::add_backup(root,&backup)?;
        Ok(backup)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(stage);let _=fs::remove_dir_all(target); }
    let mut backup=result?;
    let mut protected=protected.to_vec();protected.push(backup.id.clone());
    match crate::backup_retention::apply(root,server_id,None,&protected) {
        Ok(plan) if !plan.warnings.is_empty()=>backup.retention_warning=Some(plan.warnings.join(" ")),
        Err(error)=>backup.retention_warning=Some(format!("La copia está guardada, pero no se pudo aplicar la retención: {error}")),
        _=>(),
    }
    Ok(backup)
}

pub fn restore(root: &Path, server_id: &str, backup_id: &str) -> Result<(), String> {
    restore_inner(root,server_id,backup_id,None)
}

pub fn restore_tracked(root:&Path,server_id:&str,backup_id:&str,task:&crate::download::DownloadTask)->Result<(),String>{
    restore_inner(root,server_id,backup_id,Some(task))
}

fn restore_inner(root:&Path,server_id:&str,backup_id:&str,task:Option<&crate::download::DownloadTask>)->Result<(),String>{
    if let Some(task)=task{task.cancellation.check()?;task.report("Revisando copia para restaurar","preparing");}
    recover_restore(root,server_id)?;
    let server = storage::get(root,server_id)?;
    let source = storage::backup_path(root,server_id,backup_id)?;
    Uuid::parse_str(backup_id).map_err(|_|"Identificador del backup no válido")?;
    let expected=backup_base(root,server_id)?.join(backup_id);
    if source!=expected || fs::symlink_metadata(&source).map_err(|e|e.to_string())?.file_type().is_symlink() || source.canonicalize().map_err(|e|e.to_string())?!=expected.parent().unwrap().canonicalize().map_err(|e|e.to_string())?.join(backup_id) {return Err("La copia sale de la biblioteca de backups".into());}
    if !source.is_dir() || !source.join("server.properties").is_file() { return Err("El backup está incompleto o no es válido".into()); }
    let metadata=read_snapshot(&source,snapshot(root,&server)?)?;
    if let Some(relative)=&metadata.jar {
        let file=source.join(relative).canonicalize().map_err(|_|"El backup no incluye su archivo de arranque. Revisa la copia seleccionada.")?;
        if !file.starts_with(source.canonicalize().map_err(|e|e.to_string())?) {return Err("El arranque del backup sale de su carpeta".into());}
    }
    create_preserving(root,server_id,&[backup_id.to_owned()],task,Reason::BeforeRestore)?;
    if let Some(task)=task{task.report("Preparando archivos de la restauración","preparing");task.cancellation.check()?;}
    let (target,stage,previous,journal)=restore_paths(root,&server)?;
    let token=Uuid::new_v4().to_string();
    let result=(||->Result<(),String>{
        if let Some(task)=task{copy_tree_tracked(&source,&stage,task,"Restaurando archivos de la copia")?;}else{copy_tree(&source,&stage)?;}
        fs::remove_file(stage.join(SNAPSHOT)).or_else(|e|if e.kind()==std::io::ErrorKind::NotFound {Ok(())}else{Err(e)}).map_err(|e|e.to_string())?;
        if let Some(task)=task{task.begin_publication("Aplicando restauración y configuración")?;}
        let pending=stage.join(".tch-restore.pending");let mut file=fs::File::create(&pending).map_err(|e|e.to_string())?;
        use std::io::Write;file.write_all(token.as_bytes()).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);fs::rename(pending,&journal).map_err(|e|e.to_string())?;
        fs::rename(&target,&previous).map_err(|e|e.to_string())?;fs::rename(&stage,&target).map_err(|e|e.to_string())?;
        commit_snapshot(root,&server,&metadata,&token)?;
        recover_restore(root,server_id)
    })();
    if result.is_err() {if journal.exists(){recover_restore(root,server_id)?;}else if stage.exists(){fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}}
    result
}

fn restore_paths(root:&Path,server:&crate::model::Server)->Result<(std::path::PathBuf,std::path::PathBuf,std::path::PathBuf,std::path::PathBuf),String> {
    Uuid::parse_str(&server.id).map_err(|_|"Servidor no válido")?;
    let target=std::path::PathBuf::from(&server.directory);let parent=target.parent().ok_or("Carpeta del servidor no válida")?;
    let stage=parent.join(format!(".tch-restore-stage-{}",server.id));let previous=parent.join(format!(".tch-restore-old-{}",server.id));
    let journal=root.join("database").join(format!(".tch-restore-{}.journal",server.id));
    for path in [&target,&stage,&previous,&journal] {if fs::symlink_metadata(path).is_ok_and(|meta|meta.file_type().is_symlink()){return Err("La restauración no admite enlaces simbólicos".into());}}
    Ok((target,stage,previous,journal))
}

fn commit_snapshot(root:&Path,server:&crate::model::Server,value:&Snapshot,token:&str)->Result<(),String> {
    validate_snapshot(value)?;let mut db=storage::connect(root)?;let transaction=db.transaction().map_err(|e|e.to_string())?;
    let jar=value.jar.as_ref().map(|relative|{
        let mut path=std::path::PathBuf::from(&server.directory);
        for component in Path::new(relative).components(){path.push(component.as_os_str());}
        path.to_string_lossy().into_owned()
    });
    // Keep the explicitly configured runtime only if the Minecraft version is
    // unchanged. A different version must prepare/verify its own Java runtime.
    let java=if server.version==value.version {server.java_path.clone()}else{None};
    let changed=transaction.execute("UPDATE servers SET version=?2,platform=?3,jar_path=?4,java_path=?5,memory_min_mb=?6,memory_max_mb=?7,storage_target_gb=?8 WHERE id=?1",rusqlite::params![server.id,value.version,value.platform,jar,java,value.memory_min_mb,value.memory_max_mb,value.storage_target_gb]).map_err(|e|e.to_string())?;
    if changed!=1 {return Err("El servidor cambió durante la restauración".into());}
    transaction.execute("INSERT INTO jvm_settings(server_id,mode,arguments) VALUES(?1,?2,?3) ON CONFLICT(server_id) DO UPDATE SET mode=excluded.mode,arguments=excluded.arguments",rusqlite::params![server.id,value.jvm.mode,value.jvm.arguments]).map_err(|e|e.to_string())?;
    transaction.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",rusqlite::params![format!("restore:{}",server.id),token]).map_err(|e|e.to_string())?;
    transaction.commit().map_err(|e|e.to_string())
}

pub fn recover_restore(root:&Path,id:&str)->Result<(),String> {
    let server=storage::get(root,id)?;let (target,stage,previous,journal)=restore_paths(root,&server)?;
    if !journal.exists() {
        if previous.exists(){return Err("Hay una restauración sin registro de recuperación".into());}
        if stage.exists(){fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}return Ok(());
    }
    if fs::metadata(&journal).map_err(|e|e.to_string())?.len()>64 {return Err("Registro de restauración no válido".into());}
    let token=fs::read_to_string(&journal).map_err(|e|e.to_string())?;Uuid::parse_str(&token).map_err(|_|"Registro de restauración no válido")?;
    use rusqlite::OptionalExtension;
    let marker:Option<String>=storage::connect(root)?.query_row("SELECT value FROM application_settings WHERE key=?1",[format!("restore:{id}")],|row|row.get(0)).optional().map_err(|e|e.to_string())?;
    if marker.as_deref()==Some(&token) {
        if !target.is_dir(){return Err("Falta la restauración confirmada".into());}
        if previous.exists(){fs::remove_dir_all(&previous).map_err(|e|e.to_string())?;}
    }else if previous.exists(){
        if target.exists(){fs::remove_dir_all(&target).map_err(|e|e.to_string())?;}
        fs::rename(&previous,&target).map_err(|e|e.to_string())?;
    }else if !target.exists(){return Err("Falta la carpeta original del servidor".into());}
    if stage.exists(){fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}
    fs::remove_file(journal).map_err(|e|e.to_string())
}

pub fn recover_all(root:&Path)->Result<(),String> {for server in storage::list(root)? {recover_restore(root,&server.id)?;crate::backup_retention::recover(root,&server.id)?;}Ok(())}

pub(crate) fn backup_base(root:&Path,id:&str)->Result<std::path::PathBuf,String> {
    Uuid::parse_str(id).map_err(|_|"Servidor no válido")?;
    let canonical=root.canonicalize().map_err(|e|e.to_string())?;
    let parent=root.join("backups");fs::create_dir_all(&parent).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&parent).map_err(|e|e.to_string())?.file_type().is_symlink() || parent.canonicalize().map_err(|e|e.to_string())?!=canonical.join("backups") {return Err("La carpeta de backups sale de la biblioteca".into());}
    let base=parent.join(id);fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&base).map_err(|e|e.to_string())?.file_type().is_symlink() || base.canonicalize().map_err(|e|e.to_string())?!=canonical.join("backups").join(id) {return Err("La carpeta de backups del servidor no es válida".into());}
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Library(std::path::PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,crate::model::Server) {
        let root=std::env::temp_dir().join(format!("tch-backup-metadata-{}",Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let server=storage::create(&root,crate::model::CreateServer{name:"Backup".into(),version:"1.21.4".into(),platform:"fabric".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        (Library(root),server)
    }
    #[test]
    fn backup_list_uses_saved_reason_and_version_without_guessing_old_metadata(){
        let (root,server)=fixture();let task=crate::download::DownloadTask::new(&server.id,"minecraft_update",|_|{}).unwrap();
        let update=create_tracked(&root.0,&server.id,&task).unwrap();let scheduled_task=crate::download::DownloadTask::new(&server.id,"backup",|_|{}).unwrap();let scheduled=create_tracked_for(&root.0,&server.id,&scheduled_task,Reason::Scheduled).unwrap();let manual=create(&root.0,&server.id).unwrap();
        let details=list_details(&root.0,&server.id).unwrap();assert_eq!(details.len(),3);
        let saved=details.iter().find(|value|value.backup.id==update.id).unwrap();assert_eq!(saved.reason,Some(Reason::BeforeMinecraftUpdate));assert_eq!(saved.minecraft_version.as_deref(),Some("1.21.4"));assert_eq!(saved.platform.as_deref(),Some("fabric"));assert!(saved.metadata_error.is_none());
        assert_eq!(details.iter().find(|value|value.backup.id==scheduled.id).unwrap().reason,Some(Reason::Scheduled));assert_eq!(details.iter().find(|value|value.backup.id==manual.id).unwrap().reason,Some(Reason::Manual));
        let path=Path::new(&manual.path).join(SNAPSHOT);let mut old:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();old.as_object_mut().unwrap().remove("reason");fs::write(&path,serde_json::to_vec(&old).unwrap()).unwrap();
        let old=list_details(&root.0,&server.id).unwrap().into_iter().find(|value|value.backup.id==manual.id).unwrap();assert!(old.reason.is_none());assert_eq!(old.minecraft_version.as_deref(),Some("1.21.4"));
        fs::remove_file(&path).unwrap();let missing=list_details(&root.0,&server.id).unwrap().into_iter().find(|value|value.backup.id==manual.id).unwrap();assert!(missing.minecraft_version.is_none());assert!(missing.reason.is_none());assert!(missing.metadata_error.is_none());
        fs::write(Path::new(&update.path).join(SNAPSHOT),b"broken").unwrap();let details=list_details(&root.0,&server.id).unwrap();assert_eq!(details.len(),3);assert!(details.iter().find(|value|value.backup.id==update.id).unwrap().metadata_error.is_some());assert_eq!(details.iter().find(|value|value.backup.id==scheduled.id).unwrap().reason,Some(Reason::Scheduled));
        let foreign=root.0.join("foreign");fs::create_dir(&foreign).unwrap();fs::write(foreign.join(SNAPSHOT),b"{} ").unwrap();storage::connect(&root.0).unwrap().execute("UPDATE backups SET path=?2 WHERE id=?1",rusqlite::params![manual.id,foreign.to_string_lossy()]).unwrap();assert!(list_details(&root.0,&server.id).unwrap().iter().find(|value|value.backup.id==manual.id).unwrap().metadata_error.is_some());
    }
    #[test]
    fn tracked_backup_reports_real_bytes_before_publication(){
        let (root,server)=fixture();let source=Path::new(&server.directory);
        fs::write(source.join("large.bin"),vec![42u8;600_000]).unwrap();
        let expected=size(source);let events=std::sync::Arc::new(std::sync::Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();
        let task=crate::download::DownloadTask::new(&server.id,"backup",move|event|capture.lock().unwrap().push(event)).unwrap();
        let saved=create_tracked(&root.0,&server.id,&task).unwrap();
        assert_eq!(fs::read(Path::new(&saved.path).join("large.bin")).unwrap(),vec![42u8;600_000]);
        assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),1);
        let events=events.lock().unwrap();let measured=events.iter().filter(|event|event.stage=="copying").last().unwrap();
        assert_eq!(measured.received,expected);assert_eq!(measured.total,Some(expected));assert_eq!(events.last().unwrap().stage,"publishing");
    }
    #[test]
    fn cancelled_tracked_backup_never_publishes_or_changes_source(){
        let (root,server)=fixture();let source=Path::new(&server.directory);fs::write(source.join("keep.bin"),vec![7u8;600_000]).unwrap();
        let slot=std::sync::Arc::new(std::sync::Mutex::new(None::<std::sync::Arc<crate::download::Cancellation>>));let capture=slot.clone();
        let task=crate::download::DownloadTask::new(&server.id,"backup",move|event|{if event.stage=="copying"&&event.received>0{capture.lock().unwrap().as_ref().unwrap().cancel();}}).unwrap();
        *slot.lock().unwrap()=Some(task.cancellation.clone());
        assert_eq!(create_tracked(&root.0,&server.id,&task).err().unwrap(),crate::download::CANCELLED);
        assert!(storage::list_backups(&root.0,&server.id).unwrap().is_empty());
        assert_eq!(fs::read(source.join("keep.bin")).unwrap(),vec![7u8;600_000]);
        assert_eq!(fs::read_dir(backup_base(&root.0,&server.id).unwrap()).unwrap().count(),0);
    }
    #[test]
    fn tracked_restore_measures_safety_copy_and_restore_separately(){
        let (root,server)=fixture();let source=Path::new(&server.directory);fs::write(source.join("value.bin"),vec![3u8;500_000]).unwrap();
        let saved=create(&root.0,&server.id).unwrap();fs::write(source.join("value.bin"),vec![5u8;600_000]).unwrap();
        let current_size=size(source);let saved_size=size(Path::new(&saved.path));
        let events=std::sync::Arc::new(std::sync::Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();
        let task=crate::download::DownloadTask::new(&server.id,"restore",move|event|capture.lock().unwrap().push(event)).unwrap();
        restore_tracked(&root.0,&server.id,&saved.id,&task).unwrap();
        assert_eq!(fs::read(source.join("value.bin")).unwrap(),vec![3u8;500_000]);assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),2);
        let events=events.lock().unwrap();
        let safety=events.iter().filter(|event|event.label=="Copia de seguridad antes de restaurar").last().unwrap();assert_eq!(safety.received,current_size);assert_eq!(safety.total,Some(current_size));
        let restored=events.iter().filter(|event|event.label=="Restaurando archivos de la copia").last().unwrap();assert_eq!(restored.received,saved_size);assert_eq!(restored.total,Some(saved_size));
        assert_eq!(events.iter().filter(|event|event.stage=="publishing").count(),2);
    }
    #[test]
    fn tracked_restore_cancelled_before_publication_preserves_current_server_and_safety_copy(){
        let (root,server)=fixture();let source=Path::new(&server.directory);fs::write(source.join("value.bin"),vec![3u8;500_000]).unwrap();
        let saved=create(&root.0,&server.id).unwrap();fs::write(source.join("value.bin"),vec![5u8;600_000]).unwrap();
        let slot=std::sync::Arc::new(std::sync::Mutex::new(None::<std::sync::Arc<crate::download::Cancellation>>));let capture=slot.clone();
        let task=crate::download::DownloadTask::new(&server.id,"restore",move|event|{if event.label=="Restaurando archivos de la copia"&&event.received>0{capture.lock().unwrap().as_ref().unwrap().cancel();}}).unwrap();
        *slot.lock().unwrap()=Some(task.cancellation.clone());
        assert_eq!(restore_tracked(&root.0,&server.id,&saved.id,&task).unwrap_err(),crate::download::CANCELLED);
        assert_eq!(fs::read(source.join("value.bin")).unwrap(),vec![5u8;600_000]);assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),2);
        assert!(Path::new(&saved.path).is_dir());let (_,stage,previous,journal)=restore_paths(&root.0,&server).unwrap();assert!(!stage.exists());assert!(!previous.exists());assert!(!journal.exists());
    }
    #[test]
    fn restore_rebases_snapshot_launcher_version_capacity_and_jvm() {
        let (root,server)=fixture();let directory=Path::new(&server.directory);
        let old=directory.join(".tch-fabric").join(Uuid::new_v4().to_string()).join("fabric-server-launch.jar");fs::create_dir_all(old.parent().unwrap()).unwrap();fs::write(&old,b"old launcher").unwrap();
        storage::update_install(&root.0,&server.id,&old,Path::new("java21.exe")).unwrap();let settings=crate::jvm::Settings{mode:"custom".into(),arguments:"-XX:+UseG1GC".into()};crate::jvm::save(&root.0,&server.id,&settings).unwrap();
        let backup=create(&root.0,&server.id).unwrap();
        let new=directory.join(".tch-fabric").join(Uuid::new_v4().to_string()).join("fabric-server-launch.jar");fs::create_dir_all(new.parent().unwrap()).unwrap();fs::write(&new,b"new launcher").unwrap();
        storage::update_install(&root.0,&server.id,&new,Path::new("java25.exe")).unwrap();storage::connect(&root.0).unwrap().execute("UPDATE servers SET version='1.21.5',memory_max_mb=4096 WHERE id=?1",[&server.id]).unwrap();crate::jvm::save(&root.0,&server.id,&crate::jvm::Settings::default()).unwrap();
        restore(&root.0,&server.id,&backup.id).unwrap();let restored=storage::get(&root.0,&server.id).unwrap();
        assert_eq!(restored.jar_path,Some(old.to_string_lossy().into()));assert_eq!(restored.version,"1.21.4");assert_eq!(restored.memory_max_mb,2048);assert!(restored.java_path.is_none());assert!(old.is_file());assert!(!new.exists());assert_eq!(crate::jvm::get(&root.0,&server.id).unwrap(),settings);assert!(!directory.join(SNAPSHOT).exists());
    }
    #[test]
    fn interrupted_restore_recovers_before_and_after_database_commit() {
        for committed in [false,true] {
            let (root,server)=fixture();let (target,stage,previous,journal)=restore_paths(&root.0,&server).unwrap();
            fs::write(target.join("original.txt"),b"original").unwrap();fs::create_dir(&stage).unwrap();fs::write(stage.join("restored.txt"),b"restored").unwrap();
            let token=Uuid::new_v4().to_string();fs::write(&journal,&token).unwrap();fs::rename(&target,&previous).unwrap();fs::rename(&stage,&target).unwrap();
            if committed {commit_snapshot(&root.0,&server,&snapshot(&root.0,&server).unwrap(),&token).unwrap();}
            recover_all(&root.0).unwrap();assert_eq!(target.join("restored.txt").exists(),committed);assert_eq!(target.join("original.txt").exists(),!committed);assert!(!previous.exists());assert!(!journal.exists());
        }
    }
    #[test]
    fn database_failure_rolls_directory_back_and_retains_safety_backup() {
        let (root,server)=fixture();let directory=Path::new(&server.directory);fs::write(directory.join("value.txt"),b"old").unwrap();let saved=create(&root.0,&server.id).unwrap();fs::write(directory.join("value.txt"),b"current").unwrap();
        storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER fail_restore BEFORE UPDATE ON servers BEGIN SELECT RAISE(ABORT,'forced failure'); END;").unwrap();
        assert!(restore(&root.0,&server.id,&saved.id).is_err());assert_eq!(fs::read(directory.join("value.txt")).unwrap(),b"current");assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),2);
    }
    #[test]
    fn corrupt_snapshot_never_overwrites_server_or_creates_safety_backup() {
        let (root,server)=fixture();let saved=create(&root.0,&server.id).unwrap();let source=Path::new(&saved.path);let mut value:serde_json::Value=serde_json::from_slice(&fs::read(source.join(SNAPSHOT)).unwrap()).unwrap();value["jar"]="../outside".into();fs::write(source.join(SNAPSHOT),serde_json::to_vec(&value).unwrap()).unwrap();
        let original=fs::read(Path::new(&server.directory).join("server.properties")).unwrap();assert!(restore(&root.0,&server.id,&saved.id).is_err());assert_eq!(fs::read(Path::new(&server.directory).join("server.properties")).unwrap(),original);assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),1);
    }
    #[test]
    fn recovery_restores_folder_if_interrupted_between_directory_renames() {
        let (root,server)=fixture();let (target,stage,previous,journal)=restore_paths(&root.0,&server).unwrap();let token=Uuid::new_v4().to_string();
        fs::create_dir(&stage).unwrap();fs::write(&journal,token).unwrap();fs::rename(&target,&previous).unwrap();
        recover_all(&root.0).unwrap();assert!(target.join("server.properties").is_file());assert!(!stage.exists());assert!(!previous.exists());assert!(!journal.exists());
    }
    #[test]
    fn copies_nested_files() {
        let root = std::env::temp_dir().join(format!("tch-backup-test-{}",Uuid::new_v4()));
        let from = root.join("from"); let to = root.join("to");
        fs::create_dir_all(from.join("world/region")).unwrap();
        fs::write(from.join("world/region/r.0.0.mca"),b"abc").unwrap();
        copy_tree(&from,&to).unwrap();
        assert_eq!(fs::read(to.join("world/region/r.0.0.mca")).unwrap(),b"abc");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn backup_and_restore_roundtrip() {
        let root = std::env::temp_dir().join(format!("tch-roundtrip-{}",Uuid::new_v4()));
        let server = storage::create(&root,crate::model::CreateServer {
            name:"Roundtrip".into(),version:"1.21.4".into(),platform:"vanilla".into(),
            memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None,
        }).unwrap();
        let world = Path::new(&server.directory).join("world");
        fs::create_dir_all(&world).unwrap();
        fs::write(world.join("level.dat"),b"original").unwrap();
        let saved = create(&root,&server.id).unwrap();
        fs::write(world.join("level.dat"),b"changed").unwrap();
        restore(&root,&server.id,&saved.id).unwrap();
        assert_eq!(fs::read(world.join("level.dat")).unwrap(),b"original");
        assert_eq!(storage::list_backups(&root,&server.id).unwrap().len(),2);
        let _ = fs::remove_dir_all(root);
    }
}
