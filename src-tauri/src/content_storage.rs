use crate::{backup, content::{ContentPlan, PlannedContent}, download::DownloadTask, model::Server};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::HashMap, fs, io::Read, path::{Path, PathBuf}};

const MANIFEST: &str = ".tch-content.json";
const JOURNAL: &str = ".tch-content-transaction.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct InstalledContent {
    pub filename: String,
    pub title: String,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub version_number: Option<String>,
    pub sha512: String,
    pub size: u64,
    #[serde(default)]
    pub modified: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct ContentChange {pub title:String, pub filename:String, pub action:String, pub previous_version:Option<String>, pub version:String}

#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct ContentReview {
    pub token:String,
    pub plan:ContentPlan,
    pub changes:Vec<ContentChange>,
    pub warnings:Vec<String>,
}

pub struct PreparedPlan {
    pub server_id:String,
    pub minecraft:String,
    pub platform:String,
    pub snapshot:String,
    pub created:std::time::Instant,
    pub review:ContentReview,
}

fn folder(server:&Server)->Result<PathBuf,String> {
    crate::content::loader(server)?;
    let base=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let path=base.join(if server.platform=="paper" {"plugins"} else {"mods"});
    if let Ok(meta)=fs::symlink_metadata(&path) {
        if !meta.is_dir() || meta.file_type().is_symlink() || !path.canonicalize().map_err(|e|e.to_string())?.starts_with(&base) {
            return Err("La carpeta de contenido no puede ser un archivo ni un enlace".into());
        }
    }
    Ok(path)
}

fn safe_name(name:&str)->bool {
    !name.is_empty() && name.len()<=180 && !name.starts_with('.') && name.ends_with(".jar") &&
        !name.chars().any(|c| c.is_control() || matches!(c,'/'|'\\'|':'|'<'|'>'|'"'|'|'|'?'|'*'))
}

fn hash(path:&Path)->Result<String,String> {
    let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>512_000_000 {return Err("Archivo de contenido no válido o mayor de 512 MB".into());}
    let mut file=fs::File::open(path).map_err(|e|e.to_string())?;
    let mut digest=Sha512::new();let mut buffer=[0u8;65536];let mut count=0u64;
    loop {let n=file.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break;}count+=n as u64;if count>512_000_000 {return Err("Archivo de contenido demasiado grande".into());}digest.update(&buffer[..n]);}
    Ok(format!("{:x}",digest.finalize()))
}

fn manifest(path:&Path)->Result<Vec<InstalledContent>,String> {
    let path=path.join(MANIFEST);
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(vec![]),
        Err(e)=>return Err(e.to_string()),
        Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>2_000_000=>return Err("El inventario de contenido no es válido".into()),
        _=>{},
    }
    let records:Vec<InstalledContent>=serde_json::from_slice(&fs::read(path).map_err(|e|e.to_string())?).map_err(|_|"El inventario de contenido está dañado")?;
    if records.len()>512 {return Err("Demasiados archivos en el inventario".into());}
    let mut names=std::collections::HashSet::new();let mut ids=std::collections::HashSet::new();
    for record in &records {
        if !safe_name(&record.filename) || record.sha512.len()!=128 || !record.sha512.bytes().all(|c|c.is_ascii_hexdigit()) ||
            !names.insert(record.filename.to_lowercase()) || record.project_id.as_ref().is_some_and(|id|!ids.insert(id.clone())) {
            return Err("El inventario contiene nombres, hashes o proyectos duplicados no válidos".into());
        }
    }
    Ok(records)
}

pub fn list(server:&Server)->Result<Vec<InstalledContent>,String> {
    let path=folder(server)?;
    if !path.exists() {return Ok(vec![]);}
    let records=manifest(&path)?;
    let records:HashMap<_,_>=records.into_iter().map(|r|(r.filename.clone(),r)).collect();
    let mut result=vec![];
    for entry in fs::read_dir(path).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;
        let name=entry.file_name().into_string().map_err(|_|"Nombre de archivo no válido")?;
        if !name.ends_with(".jar") {continue;}
        if !safe_name(&name) || result.len()>=512 {return Err("El directorio tiene archivos de contenido no válidos o demasiados archivos".into());}
        let digest=hash(&entry.path())?;let size=entry.metadata().map_err(|e|e.to_string())?.len();
        let record=if let Some(record)=records.get(&name) {
            let mut record=record.clone();record.modified=!record.sha512.eq_ignore_ascii_case(&digest);record.sha512=digest;record.size=size;record
        } else {InstalledContent {title:name.trim_end_matches(".jar").into(),filename:name,project_id:None,version_id:None,version_number:None,sha512:digest,size,modified:false}};
        result.push(record);
    }
    result.sort_by(|a,b|a.filename.cmp(&b.filename));Ok(result)
}

fn snapshot(server:&Server, files:&[InstalledContent])->Result<String,String> {
    let mut digest=Sha512::new();
    for file in files {digest.update(file.filename.as_bytes());digest.update([0]);digest.update(file.sha512.as_bytes());digest.update([0]);}
    let path=folder(server)?.join(MANIFEST);
    if path.exists() {digest.update(fs::read(path).map_err(|e|e.to_string())?);}
    Ok(format!("{:x}",digest.finalize()))
}

pub fn review(server:&Server,plan:ContentPlan)->Result<PreparedPlan,String> {
    let files=list(server)?;let fingerprint=snapshot(server,&files)?;
    let mut versions:HashMap<String,String>=files.iter().filter_map(|file|Some((file.project_id.clone()?,file.version_id.clone()?))).collect();
    for entry in &plan.entries {versions.insert(entry.project.id.clone(),entry.version.id.clone());}
    for dependency in &plan.incompatible {
        let conflict=if let Some(id)=&dependency.version_id {versions.values().any(|v|v==id)} else {dependency.project_id.as_ref().is_some_and(|id|versions.contains_key(id))};
        if conflict {return Err("Una dependencia declara una incompatibilidad con contenido ya instalado".into());}
    }
    // Check reverse declarations from managed installed releases as well in the command layer.
    let mut changes=vec![];
    for entry in &plan.entries {
        let previous=files.iter().find(|file|file.project_id.as_deref()==Some(&entry.project.id));
        let occupied=files.iter().find(|file|file.filename.eq_ignore_ascii_case(&entry.file.filename));
        if occupied.is_some_and(|file|file.project_id.as_deref()!=Some(&entry.project.id)) {
            return Err(format!("{} ya existe y pertenece a otro archivo. No se sobrescribirá.",entry.file.filename));
        }
        if previous.is_some_and(|file|file.modified) {return Err(format!("{} ha cambiado fuera del launcher. Revisa el archivo antes de actualizarlo.",entry.project.title));}
        let present=previous.is_some_and(|file|file.filename==entry.file.filename && file.version_id.as_deref()==Some(&entry.version.id) && file.sha512.eq_ignore_ascii_case(&entry.file.hashes["sha512"]));
        changes.push(ContentChange {title:entry.project.title.clone(),filename:entry.file.filename.clone(),action:if present {"present"} else if previous.is_some() {"update"} else {"install"}.into(),previous_version:previous.and_then(|file|file.version_number.clone()),version:entry.version.version_number.clone()});
    }
    let mut warnings=vec![];
    let unknown=files.iter().filter(|file|file.project_id.is_none() || file.modified).count();
    if unknown>0 {warnings.push(format!("Hay {unknown} archivos añadidos o modificados fuera del launcher. Sus dependencias no se pueden comprobar todavía."));}
    if !plan.optional.is_empty() {warnings.push(format!("{} dependencias opcionales no se instalarán automáticamente.",plan.optional.len()));}
    Ok(PreparedPlan {server_id:server.id.clone(),minecraft:server.version.clone(),platform:server.platform.clone(),snapshot:fingerprint,created:std::time::Instant::now(),review:ContentReview {token:uuid::Uuid::new_v4().to_string(),plan,changes,warnings}})
}

#[derive(Serialize,Deserialize)]
struct Journal {id:String, folder:String}

fn checked_directory(path:&Path)->Result<(),String> {
    if let Ok(meta)=fs::symlink_metadata(path) {if !meta.is_dir() || meta.file_type().is_symlink() {return Err("La transacción contiene una ruta no válida".into());}}
    Ok(())
}

// Called under the server operation guard, including before starting Java.
pub fn recover(server:&Server)->Result<(),String> {
    let base=Path::new(&server.directory);let path=base.join(JOURNAL);
    let meta=match fs::symlink_metadata(&path) {Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(()),Err(e)=>return Err(e.to_string()),Ok(meta)=>meta};
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>1024 {return Err("Registro de instalación de contenido no válido".into());}
    let journal:Journal=serde_json::from_slice(&fs::read(&path).map_err(|e|e.to_string())?).map_err(|_|"Registro de instalación dañado")?;
    if uuid::Uuid::parse_str(&journal.id).is_err() || !matches!(journal.folder.as_str(),"mods"|"plugins") {return Err("Rutas de recuperación no válidas".into());}
    let target=base.join(&journal.folder);let stage=base.join(format!(".tch-content-{}",journal.id));let old=base.join(format!(".tch-content-old-{}",journal.id));
    for path in [&target,&stage,&old] {checked_directory(path)?;}
    if old.exists() && !target.exists() {fs::rename(&old,&target).map_err(|e|e.to_string())?;}
    if old.exists() {fs::remove_dir_all(old).map_err(|e|e.to_string())?;}
    if stage.exists() {fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}
    fs::remove_file(path).map_err(|e|e.to_string())
}

fn validate_jar(path:&Path,platform:&str)->Result<(),String> {
    let descriptor=crate::content_diagnostics::inspect(path,platform)?;
    if descriptor.environment.as_deref()==Some("client") {return Err("El JAR es solo de cliente y no se cargará en el servidor".into());}
    Ok(())
}

fn record(entry:&PlannedContent)->InstalledContent {
    InstalledContent {filename:entry.file.filename.clone(),title:entry.project.title.clone(),project_id:Some(entry.project.id.clone()),version_id:Some(entry.version.id.clone()),version_number:Some(entry.version.version_number.clone()),sha512:entry.file.hashes["sha512"].clone(),size:entry.file.size,modified:false}
}

fn publish(server:&Server,target:&Path,stage:&Path,old:&Path,id:&str)->Result<(),String> {
    let base=target.parent().ok_or("Carpeta de contenido no válida")?;
    let journal=Journal {id:id.into(),folder:target.file_name().ok_or("Carpeta no válida")?.to_string_lossy().into()};
    let mut journal_file=fs::OpenOptions::new().write(true).create_new(true).open(base.join(JOURNAL)).map_err(|e|e.to_string())?;
    use std::io::Write;
    journal_file.write_all(&serde_json::to_vec(&journal).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;journal_file.sync_all().map_err(|e|e.to_string())?;drop(journal_file);
    if target.exists() {fs::rename(target,old).map_err(|e|e.to_string())?;}
    if let Err(error)=fs::rename(stage,target) {if old.exists() {fs::rename(old,target).map_err(|e|format!("No se pudo recuperar la instalación: {e}"))?;}return Err(error.to_string());}
    recover(server)
}

#[cfg(test)]
pub fn remove(root:&Path,server:&Server,filename:&str,expected_hash:&str)->Result<(),String> {
    let task=DownloadTask::new(&server.id,"content_remove",|_|{})?;
    remove_tracked(root,server,filename,expected_hash,&task)
}

pub fn remove_tracked(root:&Path,server:&Server,filename:&str,expected_hash:&str,task:&DownloadTask)->Result<(),String> {
    task.cancellation.check()?;task.report("Comprobando el contenido que se va a retirar","preparing");
    if !safe_name(filename) {return Err("Nombre de contenido no válido".into());}
    recover(server)?;
    let files=list(server)?;let file=files.iter().find(|file|file.filename==filename).ok_or("El archivo ya no existe")?;
    if file.sha512!=expected_hash {return Err("El archivo ha cambiado desde la confirmación. Revisa el contenido de nuevo.".into());}
    let snapshot_before=snapshot(server,&files)?;
    let remaining:Vec<_>=files.iter().filter(|file|file.filename!=filename).cloned().collect();
    let empty=ContentPlan {minecraft_version:server.version.clone(),loader:crate::content::loader(server)?.into(),entries:vec![],optional:vec![],incompatible:vec![]};
    crate::content::check_existing(task,&remaining,&empty)?;
    backup::create_tracked(root,&server.id,task)?;
    task.report("Preparando la retirada del contenido","preparing");task.cancellation.check()?;
    let target=folder(server)?;let base=target.parent().ok_or("Carpeta no válida")?;let id=uuid::Uuid::new_v4().to_string();
    let stage=base.join(format!(".tch-content-{id}"));let old=base.join(format!(".tch-content-old-{id}"));
    let result=(|| {
        backup::copy_tree_tracked(&target,&stage,task,"Preparando los archivos de contenido")?;
        task.cancellation.check()?;
        fs::remove_file(stage.join(filename)).map_err(|e|e.to_string())?;
        let mut records=manifest(&target)?;records.retain(|record|record.filename!=filename);
        fs::write(stage.join(MANIFEST),serde_json::to_vec_pretty(&records).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        if snapshot(server,&list(server)?)?!=snapshot_before {return Err("Los archivos han cambiado. No se ha eliminado el contenido.".into());}
        task.begin_publication("Guardando la retirada del contenido")?;
        publish(server,&target,&stage,&old,&id)
    })();
    if result.is_err() && !base.join(JOURNAL).exists() {let _=fs::remove_dir_all(stage);}
    result
}

pub fn install(root:&Path,server:&Server,prepared:&PreparedPlan,task:&DownloadTask)->Result<(),String> {
    install_internal(Some(root),server,prepared,task)
}

// The caller owns the isolated instance directory and has already saved the
// original server's safety backup. This never publishes the active instance.
pub(crate) fn install_staged(server:&Server,plan:ContentPlan,task:&DownloadTask)->Result<(),String> {
    let prepared=review(server,plan)?;
    install_internal(None,server,&prepared,task)
}

fn install_internal(root:Option<&Path>,server:&Server,prepared:&PreparedPlan,task:&DownloadTask)->Result<(),String> {
    if prepared.server_id!=server.id || prepared.minecraft!=server.version || prepared.platform!=server.platform || prepared.created.elapsed()>std::time::Duration::from_secs(900) {return Err("La revisión ha caducado. Revisa los cambios de nuevo.".into());}
    recover(server)?;
    let files=list(server)?;
    if snapshot(server,&files)?!=prepared.snapshot {return Err("Los archivos han cambiado desde la revisión. Revisa los cambios de nuevo.".into());}
    if prepared.review.changes.iter().all(|change|change.action=="present") {return Ok(());}
    let target=folder(server)?;let base=target.parent().ok_or("Carpeta no válida")?;
    let id=uuid::Uuid::new_v4().to_string();let stage=base.join(format!(".tch-content-{id}"));let old=base.join(format!(".tch-content-old-{id}"));
    let result=(|| {
        task.cancellation.check()?;
        if let Some(root)=root {
            if prepared.review.changes.iter().any(|change|change.action=="update") {task.report("Guardando copia antes de actualizar","preparing");backup::create_tracked(root,&server.id,task)?;}
        }
        task.report("Preparando archivos","preparing");
        if target.exists() {backup::copy_tree(&target,&stage)?;} else {fs::create_dir(&stage).map_err(|e|e.to_string())?;}
        let mut records=manifest(&target)?;
        for (entry,change) in prepared.review.plan.entries.iter().zip(&prepared.review.changes) {
            task.cancellation.check()?;
            if change.action=="present" {continue;}
            if let Some(previous)=files.iter().find(|file|file.project_id.as_deref()==Some(&entry.project.id)) {
                if previous.filename!=entry.file.filename {fs::remove_file(stage.join(&previous.filename)).map_err(|e|e.to_string())?;}
            }
            let path=stage.join(&entry.file.filename);
            task.download(&entry.file.url,&path,Some(("sha512",&entry.file.hashes["sha512"])),&entry.project.title)?;
            if fs::metadata(&path).map_err(|e|e.to_string())?.len()!=entry.file.size {return Err("El tamaño descargado no coincide con Modrinth".into());}
            validate_jar(&path,&server.platform)?;
            records.retain(|record|record.project_id.as_deref()!=Some(&entry.project.id));records.push(record(entry));
        }
        fs::write(stage.join(MANIFEST),serde_json::to_vec_pretty(&records).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        task.cancellation.check()?;
        if snapshot(server,&list(server)?)?!=prepared.snapshot {return Err("Los archivos cambiaron durante la descarga. No se ha instalado el plan.".into());}
        task.cancellation.check()?;
        publish(server,&target,&stage,&old,&id)
    })();
    if result.is_err() && !base.join(JOURNAL).exists() {let _=fs::remove_dir_all(stage);}
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {root:PathBuf,server:Server}
    impl Fixture {
        fn new()->Self {
            Self::platform("fabric")
        }
        fn platform(platform:&str)->Self {
            let root=std::env::temp_dir().join(format!("tch-content-test-{}",uuid::Uuid::new_v4()));
            let server=crate::storage::create(&root,crate::model::CreateServer {name:"Content test".into(),version:"1.21.4".into(),platform:platform.into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
            Self {root,server}
        }
    }
    impl Drop for Fixture {fn drop(&mut self) {let _=fs::remove_dir_all(&self.root);}}
    fn empty_plan()->ContentPlan {ContentPlan {minecraft_version:"1.21.4".into(),loader:"fabric".into(),entries:vec![],optional:vec![],incompatible:vec![]}}
    #[test]
    fn tracked_removal_preserves_content_on_cancellation_and_keeps_a_safety_copy(){
        use std::sync::{Arc,Mutex};
        for mode in ["backup_cancel","after_backup_cancel","content_cancel","complete"]{
            let fixture=Fixture::platform("paper");let directory=folder(&fixture.server).unwrap();fs::create_dir_all(directory.join("config")).unwrap();
            fs::write(directory.join("manual.jar"),vec![6u8;300_000]).unwrap();fs::write(directory.join("config/settings.yml"),b"keep: true\n").unwrap();
            let digest=hash(&directory.join("manual.jar")).unwrap();let backup_size=backup::size(Path::new(&fixture.server.directory));let content_size=backup::size(&directory);
            let jobs:crate::file_tasks::Registry=Arc::new(Mutex::new(HashMap::new()));let observed=jobs.clone();let events=Arc::new(Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();
            let result=crate::file_tasks::remove_content(&fixture.root,&fixture.server,"manual.jar",&digest,&jobs,move|event|{
                let task=observed.lock().unwrap()[&event.server_id].clone();assert_eq!(event.task_id,task.id);assert!(task.is_file_operation());
                if (mode=="backup_cancel"&&event.stage=="copying"&&event.label=="Copiando archivos del servidor"&&event.received>0)
                    || (mode=="after_backup_cancel"&&event.label=="Preparando la retirada del contenido")
                    || (mode=="content_cancel"&&event.stage=="copying"&&event.label=="Preparando los archivos de contenido"&&event.received>0){assert!(task.request_cancel());}
                if event.stage=="publishing"{assert!(!task.request_cancel());}
                capture.lock().unwrap().push(event);
            });
            assert!(jobs.lock().unwrap().is_empty());let events=events.lock().unwrap();
            if mode=="complete"{
                result.unwrap();assert_eq!(events.last().unwrap().stage,"completed");assert!(!directory.join("manual.jar").exists());
                for (label,total) in [("Copiando archivos del servidor",backup_size),("Preparando los archivos de contenido",content_size)]{let measured=events.iter().filter(|event|event.label==label&&event.stage=="copying").last().unwrap();assert_eq!(measured.received,total);assert_eq!(measured.total,Some(total));}
                assert!(list(&fixture.server).unwrap().is_empty());
            }else{assert_eq!(result.unwrap_err(),crate::download::CANCELLED);assert_eq!(events.last().unwrap().stage,"cancelled");assert_eq!(fs::read(directory.join("manual.jar")).unwrap(),vec![6u8;300_000]);}
            assert_eq!(fs::read(directory.join("config/settings.yml")).unwrap(),b"keep: true\n");
            let backups=crate::storage::list_backups(&fixture.root,&fixture.server.id).unwrap();assert_eq!(backups.len(),usize::from(mode!="backup_cancel"));
            if let Some(copy)=backups.first(){assert_eq!(fs::read(Path::new(&copy.path).join("plugins/manual.jar")).unwrap(),vec![6u8;300_000]);}
            assert!(!Path::new(&fixture.server.directory).join(JOURNAL).exists());
            assert!(fs::read_dir(&fixture.server.directory).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().starts_with(".tch-content-")));
        }
    }

    #[test]
    fn inventory_detects_manual_and_modified_files_and_stale_review() {
        let fixture=Fixture::new();let dir=folder(&fixture.server).unwrap();fs::create_dir(&dir).unwrap();
        fs::write(dir.join("manual.jar"),b"manual").unwrap();
        let files=list(&fixture.server).unwrap();assert_eq!(files.len(),1);assert!(files[0].project_id.is_none());
        let mut managed=files[0].clone();managed.project_id=Some("project".into());managed.version_id=Some("version".into());
        fs::write(dir.join(MANIFEST),serde_json::to_vec(&vec![managed]).unwrap()).unwrap();
        let prepared=review(&fixture.server,empty_plan()).unwrap();
        fs::write(dir.join("manual.jar"),b"modified externally").unwrap();
        assert!(list(&fixture.server).unwrap()[0].modified);
        let task=DownloadTask::new("test","content_install",|_|{}).unwrap();
        assert!(install(&fixture.root,&fixture.server,&prepared,&task).unwrap_err().contains("cambiado"));
        assert_eq!(fs::read(dir.join("manual.jar")).unwrap(),b"modified externally");
    }
    #[test]
    fn identical_artifact_with_new_release_id_requires_inventory_update() {
        let fixture=Fixture::new();
        let dir=folder(&fixture.server).unwrap();fs::create_dir(&dir).unwrap();
        let filename="shared.jar";
        fs::write(dir.join(filename),b"same published artifact").unwrap();
        let digest=hash(&dir.join(filename)).unwrap();
        let old=InstalledContent {filename:filename.into(),title:"Shared mod".into(),project_id:Some("project".into()),version_id:Some("old-release".into()),version_number:Some("1.0".into()),sha512:digest.clone(),size:23,modified:false};
        fs::write(dir.join(MANIFEST),serde_json::to_vec(&vec![old]).unwrap()).unwrap();
        let file=crate::content::ContentFile {filename:filename.into(),url:"https://cdn.modrinth.com/shared.jar".into(),hashes:std::collections::BTreeMap::from([("sha512".into(),digest)]),size:23,primary:true};
        let version:crate::content::ContentVersion=serde_json::from_value(serde_json::json!({
            "id":"new-release","project_id":"project","name":"Compatible release","version_number":"1.1",
            "version_type":"release","date_published":"2026-01-01T00:00:00Z",
            "game_versions":["1.21.4"],"loaders":["fabric"],"dependencies":[],"files":[file]
        })).unwrap();
        let project:crate::content::Project=serde_json::from_value(serde_json::json!({"id":"project","title":"Shared mod","description":"","icon_url":null})).unwrap();
        let mut plan=empty_plan();
        plan.entries.push(PlannedContent {project,version,file,required:true});
        let prepared=review(&fixture.server,plan.clone()).unwrap();
        assert_eq!(prepared.review.changes[0].action,"update");
        assert_eq!(prepared.review.changes[0].previous_version.as_deref(),Some("1.0"));
        let record=record(&plan.entries[0]);
        fs::write(dir.join(MANIFEST),serde_json::to_vec(&vec![record]).unwrap()).unwrap();
        assert_eq!(review(&fixture.server,plan).unwrap().review.changes[0].action,"present");
    }
    #[test]
    fn recovery_restores_old_directory_after_interrupted_switch() {
        let fixture=Fixture::new();let base=Path::new(&fixture.server.directory);let id=uuid::Uuid::new_v4().to_string();
        let old=base.join(format!(".tch-content-old-{id}"));let stage=base.join(format!(".tch-content-{id}"));
        fs::create_dir(&old).unwrap();fs::write(old.join("preserved.jar"),b"old").unwrap();fs::create_dir(&stage).unwrap();fs::write(stage.join("new.jar"),b"new").unwrap();
        fs::write(base.join(JOURNAL),serde_json::to_vec(&Journal {id,folder:"mods".into()}).unwrap()).unwrap();
        recover(&fixture.server).unwrap();assert_eq!(fs::read(base.join("mods/preserved.jar")).unwrap(),b"old");
        assert!(!stage.exists());assert!(!old.exists());assert!(!base.join(JOURNAL).exists());
    }
    #[test]
    fn recovery_keeps_committed_directory_and_rejects_unsafe_journal() {
        let fixture=Fixture::new();let base=Path::new(&fixture.server.directory);let id=uuid::Uuid::new_v4().to_string();
        let old=base.join(format!(".tch-content-old-{id}"));fs::create_dir(&old).unwrap();fs::write(old.join("old.jar"),b"old").unwrap();
        fs::create_dir(base.join("mods")).unwrap();fs::write(base.join("mods/new.jar"),b"new").unwrap();
        fs::write(base.join(JOURNAL),serde_json::to_vec(&Journal {id,folder:"mods".into()}).unwrap()).unwrap();
        recover(&fixture.server).unwrap();assert!(base.join("mods/new.jar").exists());assert!(!old.exists());
        fs::write(base.join(JOURNAL),b"{\"id\":\"../escape\",\"folder\":\"../outside\"}").unwrap();assert!(recover(&fixture.server).is_err());
        assert!(base.join("mods/new.jar").exists());
    }

    #[test]
    #[ignore="Downloads official Modrinth releases, installs and updates in an isolated server without executing Java"]
    fn real_install_update_preserves_files_and_creates_backup() {
        let fixture=Fixture::new();let task=DownloadTask::new(&fixture.server.id,"content_install",|_|{}).unwrap();
        let versions=crate::content::versions(&task,&fixture.server,"lithium").unwrap();
        let stable:Vec<_>=versions.iter().filter(|v|v.version_type=="release").collect();assert!(stable.len()>=2);
        let dir=folder(&fixture.server).unwrap();fs::create_dir(&dir).unwrap();fs::write(dir.join("manual.jar"),b"user file").unwrap();fs::create_dir(dir.join("config")).unwrap();fs::write(dir.join("config/settings.txt"),b"user settings").unwrap();
        let first=crate::content::plan(&task,&fixture.server,"lithium",Some(&stable[1].id)).unwrap();
        let first=review(&fixture.server,first).unwrap();install(&fixture.root,&fixture.server,&first,&task).unwrap();
        let files=list(&fixture.server).unwrap();assert_eq!(files.len(),2);assert!(files.iter().any(|f|f.version_id.as_deref()==Some(&stable[1].id)));
        let latest=crate::content::updates(&task,&fixture.server).unwrap();
        crate::content::check_existing(&task,&files,&latest).unwrap();let latest=review(&fixture.server,latest).unwrap();
        assert!(latest.review.changes.iter().any(|c|c.action=="update"));install(&fixture.root,&fixture.server,&latest,&task).unwrap();
        let files=list(&fixture.server).unwrap();assert_eq!(files.len(),2);assert!(files.iter().any(|f|f.version_id.as_deref()==Some(&stable[0].id)));
        assert_eq!(fs::read(dir.join("manual.jar")).unwrap(),b"user file");assert_eq!(fs::read(dir.join("config/settings.txt")).unwrap(),b"user settings");
        assert_eq!(crate::storage::list_backups(&fixture.root,&fixture.server.id).unwrap().len(),1);
        assert!(!Path::new(&fixture.server.directory).join(JOURNAL).exists());
        assert!(!crate::eula::recorded(&fixture.root).unwrap());
        let installed=files.iter().find(|file|file.project_id.is_some()).unwrap();
        remove(&fixture.root,&fixture.server,&installed.filename,&installed.sha512).unwrap();
        assert_eq!(list(&fixture.server).unwrap().len(),1);assert!(dir.join("manual.jar").exists());assert!(dir.join("config/settings.txt").exists());
        assert_eq!(crate::storage::list_backups(&fixture.root,&fixture.server.id).unwrap().len(),2);
    }

    #[test]
    #[ignore="Installs a verified Paper plugin in an isolated directory without executing Java"]
    fn installs_real_paper_plugin_in_plugins_directory() {
        let fixture=Fixture::platform("paper");let task=DownloadTask::new(&fixture.server.id,"content_install",|_|{}).unwrap();
        let plan=crate::content::plan(&task,&fixture.server,"luckperms",None).unwrap();
        let prepared=review(&fixture.server,plan).unwrap();install(&fixture.root,&fixture.server,&prepared,&task).unwrap();
        let files=list(&fixture.server).unwrap();assert!(!files.is_empty());
        assert!(Path::new(&fixture.server.directory).join("plugins").join(&files[0].filename).is_file());
        assert!(!Path::new(&fixture.server.directory).join("mods").exists());
        assert!(!crate::eula::recorded(&fixture.root).unwrap());
    }

    #[test]
    #[ignore="Reads the live catalog and project icon from official Modrinth endpoints"]
    fn search_and_icon_use_live_official_catalog() {
        let fixture=Fixture::platform("paper");let task=DownloadTask::new(&fixture.server.id,"content_search",|_|{}).unwrap();
        let result=crate::content::search(&task,&fixture.server,"luckperms",0,&crate::content::SearchOptions::default()).unwrap();
        let hits=result["hits"].as_array().unwrap();let project=hits.iter().find(|hit|hit["title"]=="LuckPerms").expect("LuckPerms should appear in compatible Paper search");
        let icon=crate::content::icon(&fixture.root,project["icon_url"].as_str().unwrap()).unwrap();
        assert!(icon.starts_with("data:image/png;base64,"));
    }
}
