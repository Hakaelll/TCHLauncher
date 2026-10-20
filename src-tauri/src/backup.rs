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

#[cfg(test)]
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

#[cfg(test)]
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
    fn corrupt_snapshot_never_overwrites_server_or_creates_safety_backup() {
        let (root,server)=fixture();let saved=create(&root.0,&server.id).unwrap();let source=Path::new(&saved.path);let mut value:serde_json::Value=serde_json::from_slice(&fs::read(source.join(SNAPSHOT)).unwrap()).unwrap();value["jar"]="../outside".into();fs::write(source.join(SNAPSHOT),serde_json::to_vec(&value).unwrap()).unwrap();
        let original=fs::read(Path::new(&server.directory).join("server.properties")).unwrap();assert!(restore(&root.0,&server.id,&saved.id).is_err());assert_eq!(fs::read(Path::new(&server.directory).join("server.properties")).unwrap(),original);assert_eq!(storage::list_backups(&root.0,&server.id).unwrap().len(),1);
    }

}
