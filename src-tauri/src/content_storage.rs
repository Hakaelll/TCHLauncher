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
    list_kind(server,false)
}

/// Logical filenames remain .jar; only the on-disk suffix changes while disabled.
pub fn list_disabled(server:&Server)->Result<Vec<InstalledContent>,String> {
    list_kind(server,true)
}

fn list_kind(server:&Server,disabled:bool)->Result<Vec<InstalledContent>,String> {
    let path=folder(server)?;
    if !path.exists() {return Ok(vec![]);}
    let records=manifest(&path)?;
    let records:HashMap<_,_>=records.into_iter().map(|r|(r.filename.clone(),r)).collect();
    let mut result=vec![];
    for entry in fs::read_dir(path).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;
        let disk_name=entry.file_name().into_string().map_err(|_|"Nombre de archivo no válido")?;
        let name=if disabled {let Some(name)=disk_name.strip_suffix(".disabled") else{continue};name.to_string()}else{disk_name};
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
    digest.update(b"disabled\0");
    for file in list_disabled(server)? {digest.update(file.filename.as_bytes());digest.update([0]);digest.update(file.sha512.as_bytes());digest.update([0]);}
    let path=folder(server)?.join(MANIFEST);
    if path.exists() {digest.update(fs::read(path).map_err(|e|e.to_string())?);}
    Ok(format!("{:x}",digest.finalize()))
}

pub fn review(server:&Server,plan:ContentPlan)->Result<PreparedPlan,String> {
    let files=list(server)?;let fingerprint=snapshot(server,&files)?;
    let disabled=list_disabled(server)?;
    for entry in &plan.entries {
        if disabled.iter().any(|file|file.filename.eq_ignore_ascii_case(&entry.file.filename)||file.project_id.as_deref()==Some(&entry.project.id)) {
            return Err(format!("{} está desactivado. Reactiva el archivo antes de instalar o actualizar este proyecto.",entry.project.title));
        }
    }
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

pub fn set_enabled_tracked(root:&Path,server:&Server,filename:&str,expected_hash:&str,enabled:bool,task:&DownloadTask)->Result<(),String> {
    task.cancellation.check()?;task.report("Comprobando el cambio de contenido","preparing");
    if !safe_name(filename) {return Err("Nombre de contenido no válido".into());}
    recover(server)?;
    let files=list(server)?;let disabled=list_disabled(server)?;
    let source_files=if enabled {&disabled}else{&files};
    let file=source_files.iter().find(|file|file.filename==filename).ok_or("El archivo ya no existe en el estado seleccionado")?;
    if file.sha512!=expected_hash {return Err("El archivo ha cambiado desde la confirmación. Revisa el contenido de nuevo.".into());}
    let target=folder(server)?;
    let (source,destination)=if enabled {(format!("{filename}.disabled"),filename.to_string())}else{(filename.to_string(),format!("{filename}.disabled"))};
    if fs::symlink_metadata(target.join(&destination)).is_ok() {return Err("Ya existe un archivo en el destino. No se sobrescribirá.".into());}
    let snapshot_before=snapshot(server,&files)?;
    let mut remaining:Vec<_>=files.iter().filter(|file|file.filename!=filename).cloned().collect();
    if enabled {
        let descriptor=crate::content_diagnostics::inspect(&target.join(&source),&server.platform)?;
        if descriptor.environment.as_deref()==Some("client") {return Err("El JAR es solo de cliente y no se cargará en el servidor".into());}
        for active in &files {
            if active.project_id.is_some()&&active.project_id==file.project_id {return Err("El proyecto ya tiene un archivo activo".into());}
            if let Ok(other)=crate::content_diagnostics::inspect(&target.join(&active.filename),&server.platform) {
                if descriptor.ids.iter().any(|id|other.ids.contains(id)) {return Err("El archivo declara un identificador que ya está activo".into());}
            }
        }
        remaining.push(file.clone());remaining.sort_by(|a,b|a.filename.cmp(&b.filename));
    }
    let empty=ContentPlan {minecraft_version:server.version.clone(),loader:crate::content::loader(server)?.into(),entries:vec![],optional:vec![],incompatible:vec![]};
    crate::content::check_existing(task,&remaining,&empty)?;
    let before=if server.platform=="fabric" {Some(crate::fabric_presence::review(Path::new(&server.directory),&files))}else{None};
    backup::create_tracked(root,&server.id,task)?;
    task.report("Preparando el cambio de contenido","preparing");task.cancellation.check()?;
    let base=target.parent().ok_or("Carpeta no válida")?;let id=uuid::Uuid::new_v4().to_string();
    let stage=base.join(format!(".tch-content-{id}"));let old=base.join(format!(".tch-content-old-{id}"));
    let result=(|| {
        backup::copy_tree_tracked(&target,&stage,task,"Preparando los archivos de contenido")?;
        task.cancellation.check()?;
        if hash(&stage.join(&source))?!=expected_hash {return Err("El archivo ha cambiado durante la copia. No se ha aplicado el cambio.".into());}
        if fs::symlink_metadata(stage.join(&destination)).is_ok() {return Err("Ya existe un archivo en el destino. No se sobrescribirá.".into());}
        fs::rename(stage.join(&source),stage.join(&destination)).map_err(|e|e.to_string())?;
        if let Some(before)=&before {
            let after=crate::fabric_presence::review_at(base,&stage,&remaining);
            for (name,issues) in after.blocking {
                for issue in issues {
                    if !before.blocking.get(&name).is_some_and(|old|old.contains(&issue)) {return Err(format!("{name}: {issue}"));}
                }
            }
        }
        if snapshot(server,&list(server)?)?!=snapshot_before {return Err("Los archivos han cambiado. No se ha aplicado el cambio de contenido.".into());}
        task.begin_publication("Guardando el cambio de contenido")?;
        publish(server,&target,&stage,&old,&id)
    })();
    if result.is_err()&&!base.join(JOURNAL).exists() {let _=fs::remove_dir_all(stage);}
    result
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
