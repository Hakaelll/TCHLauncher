use crate::{download::DownloadTask,model::Server,modpack,storage};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha512};
use std::{fs,io::Write,path::{Path,PathBuf}};
use uuid::Uuid;

#[derive(Serialize,Deserialize)]
struct Journal {id:String,jar:String}

fn paths(root:&Path,id:&str)->Result<(PathBuf,PathBuf,PathBuf,PathBuf),String> {
    Uuid::parse_str(id).map_err(|_|"Identificador del paquete no válido")?;
    let base=root.join("servers");fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    let canonical=root.canonicalize().map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&base).map_err(|e|e.to_string())?.file_type().is_symlink() || base.canonicalize().map_err(|e|e.to_string())?!=canonical.join("servers") {return Err("La carpeta de servidores sale de la biblioteca".into());}
    let paths=(base.join(id),base.join(format!(".tch-pack-stage-{id}")),base.join(format!(".tch-pack-old-{id}")),base.join(format!(".tch-pack-journal-{id}.json")));
    for path in [&paths.0,&paths.1,&paths.2,&paths.3] {
        if fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()) {return Err("La importación no admite enlaces simbólicos".into());}
    }
    Ok(paths)
}

fn baseline(server:&Server)->Result<String,String> {
    if server.jar_path.is_some() {return Err("Importa el paquete en un servidor nuevo".into());}
    let mut entries=fs::read_dir(&server.directory).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    entries.sort_by_key(|entry|entry.file_name());let mut digest=Sha512::new();
    for entry in entries {
        let name=entry.file_name();let name=name.to_str().ok_or("Nombre de archivo no válido")?;
        if !matches!(name,"server.properties"|"server-icon.png"|"eula.txt") {return Err("El servidor ya contiene archivos. Crea una instancia nueva para el modpack.".into());}
        let meta=fs::symlink_metadata(entry.path()).map_err(|e|e.to_string())?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>2_000_000 {return Err("Los archivos iniciales del servidor no son válidos".into());}
        digest.update(name.as_bytes());digest.update([0]);digest.update(fs::read(entry.path()).map_err(|e|e.to_string())?);
    }
    Ok(format!("{:x}",digest.finalize()))
}

pub fn recover(root:&Path,id:&str)->Result<(),String> {
    let (target,stage,old,journal)=paths(root,id)?;
    if !journal.exists() {
        if old.exists() {return Err("Hay una copia de importación sin registro de recuperación".into());}
        if stage.exists() {fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}
        return Ok(());
    }
    if fs::metadata(&journal).map_err(|e|e.to_string())?.len()>8192 {return Err("Registro de importación demasiado grande".into());}
    let record:Journal=serde_json::from_slice(&fs::read(&journal).map_err(|e|e.to_string())?).map_err(|_|"Registro de importación no válido")?;
    if record.id!=id {return Err("El registro pertenece a otro servidor".into());}
    let relative=modpack::relative_path(&record.jar)?;let server=storage::get(root,id)?;
    if Path::new(&server.directory)!=target {return Err("El servidor no pertenece a la biblioteca gestionada".into());}
    let committed=server.jar_path.as_deref()==Some(target.join(&relative).to_string_lossy().as_ref());
    if committed {
        if !target.join(relative).is_file() {return Err("Falta el arranque de la importación completada".into());}
        if old.exists() {fs::remove_dir_all(&old).map_err(|e|e.to_string())?;}
    } else {
        if server.jar_path.is_some() {return Err("El servidor cambió durante la recuperación del paquete".into());}
        if old.exists() {
            if target.exists() {fs::remove_dir_all(&target).map_err(|e|e.to_string())?;}
            fs::rename(&old,&target).map_err(|e|e.to_string())?;
        } else if !target.exists() {return Err("Falta la carpeta original del servidor".into());}
    }
    if stage.exists() {fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}
    fs::remove_file(journal).map_err(|e|e.to_string())
}

pub fn recover_all(root:&Path)->Result<(),String> {
    let base=root.join("servers");if !base.exists(){return Ok(());}
    let entries=fs::read_dir(base).map_err(|e|e.to_string())?;
    let mut ids=std::collections::HashSet::new();
    for entry in entries.take(10000) {
        let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name().to_string_lossy().into_owned();
        if let Some(id)=name.strip_prefix(".tch-pack-") {
            if Uuid::parse_str(id).is_ok() {
                let actual=entry.path();
                if fs::symlink_metadata(&actual).map_err(|e|e.to_string())?.file_type().is_symlink() || !actual.canonicalize().map_err(|e|e.to_string())?.starts_with(root.canonicalize().map_err(|e|e.to_string())?) {return Err("La carpeta temporal sale de la biblioteca".into());}
                fs::remove_dir_all(actual).map_err(|e|e.to_string())?;continue;
            }
        }
        let id=name.strip_prefix(".tch-pack-journal-").and_then(|s|s.strip_suffix(".json")).or_else(||name.strip_prefix(".tch-pack-stage-"));
        if let Some(id)=id {Uuid::parse_str(id).map_err(|_|"Nombre de recuperación no válido")?;ids.insert(id.to_owned());}
    }
    for id in ids {recover(root,&id)?;}Ok(())
}

pub fn install(root:&Path,server:&Server,path:&Path,hash:&str,selected:&[String],task:&DownloadTask)->Result<(),String> {
    recover(root,&server.id)?;let before=baseline(server)?;
    let (target,stage,_,journal)=paths(root,&server.id)?;
    if Path::new(&server.directory)!=target {return Err("El paquete requiere un servidor nuevo de la biblioteca".into());}
    let review=modpack::inspect(path)?.inspection;
    if review.minecraft!=server.version || review.loader!=server.platform {return Err("El motor o Minecraft del servidor no coincide con el paquete".into());}
    let prepared=modpack::prepare(path,hash,selected,target.parent().unwrap(),task)?;
    if prepared.inspection.minecraft!=server.version || prepared.inspection.loader!=server.platform {return Err("La copia revisada del paquete no coincide con el servidor".into());}
    let result=(||->Result<(),String>{
        fs::create_dir(&stage).map_err(|e|e.to_string())?;
        crate::backup::copy_tree(&target,&stage)?;
        crate::backup::copy_tree(&prepared.directory.join("contents"),&stage)?;
        let mut staged=server.clone();staged.directory=stage.to_string_lossy().into();
        let (jar,java)=crate::install::prepare_fresh_instance(root,&staged,task,prepared.inspection.loader_version.as_deref())?;
        let receipt=serde_json::json!({"name":prepared.inspection.name,"versionId":prepared.inspection.version_id,"archiveSha512":prepared.inspection.archive_sha512,"minecraft":prepared.inspection.minecraft,"loader":prepared.inspection.loader,"loaderVersion":prepared.inspection.loader_version,"files":prepared.inspection.entries.iter().filter(|entry|entry.selected).map(|entry|entry.path.clone()).collect::<Vec<_>>()});
        let mut receipt_file=fs::File::create(stage.join(".tch-modpack.json")).map_err(|e|e.to_string())?;receipt_file.write_all(&serde_json::to_vec_pretty(&receipt).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;receipt_file.sync_all().map_err(|e|e.to_string())?;drop(receipt_file);
        let canonical_stage=stage.canonicalize().map_err(|e|e.to_string())?;
        let canonical_jar=jar.canonicalize().map_err(|e|e.to_string())?;
        let relative=canonical_jar.strip_prefix(&canonical_stage).map_err(|_|"El arranque preparado está fuera de la instancia")?.to_string_lossy().replace('\\',"/");
        modpack::relative_path(&relative)?;
        if baseline(&storage::get(root,&server.id)?)?!=before {return Err("El servidor cambió durante la preparación del paquete".into());}
        task.cancellation.check()?;
        commit(root,server,&relative,&java,&journal,task)
    })();
    if result.is_err() {
        if journal.exists() {recover(root,&server.id).map_err(|e|format!("La importación falló y necesita recuperación: {e}"))?;}
        else if stage.exists() {fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}
    }
    result
}

fn commit(root:&Path,server:&Server,relative:&str,java:&Path,journal:&Path,task:&DownloadTask)->Result<(),String> {
    let (target,stage,old,_)=paths(root,&server.id)?;
    task.report("Guardando la instancia del modpack","verifying");
    let data=serde_json::to_vec(&Journal {id:server.id.clone(),jar:relative.into()}).map_err(|e|e.to_string())?;
    let pending=stage.join(".tch-pack-journal.pending");
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&pending).map_err(|e|e.to_string())?;file.write_all(&data).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);
    fs::rename(pending,journal).map_err(|e|e.to_string())?;
    // Do not cancel between the directory switch and the database commit.
    fs::rename(&target,&old).map_err(|e|e.to_string())?;
    fs::rename(&stage,&target).map_err(|e|e.to_string())?;
    storage::update_install(root,&server.id,&target.join(relative),java)?;
    recover(root,&server.id)
}
