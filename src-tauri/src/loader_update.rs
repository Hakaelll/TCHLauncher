use crate::{model::Server,storage,download::DownloadTask};
use serde::{Serialize,Deserialize};
use sha2::{Digest,Sha512};
use std::{fs,io::{Read,Write},path::{Path,PathBuf}};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Review {pub current:String,pub available:Vec<String>,pub fingerprint:String}
#[derive(Serialize,Deserialize)]
struct Journal {old:String,new:String}
fn container(server:&Server)->Result<&'static str,String>{match server.platform.as_str(){"fabric"=>Ok(".tch-fabric"),"neoforge"=>Ok(".tch-neoforge"),_=>Err("Solo Fabric y NeoForge admiten actualizaciones del loader".into())}}
fn generation(server:&Server,relative:&Path)->Result<PathBuf,String> {
    let parts=relative.components().collect::<Vec<_>>();
    if parts.len()<3||parts[0].as_os_str()!=container(server)?||parts.iter().any(|part|!matches!(part,std::path::Component::Normal(_))){return Err("La generación del loader sale de su carpeta".into());}
    Uuid::parse_str(parts[1].as_os_str().to_str().ok_or("Generación no válida")?).map_err(|_|"Generación no válida")?;
    Ok(PathBuf::from(parts[0].as_os_str()).join(parts[1].as_os_str()))
}
fn paths(root:&Path,server:&Server)->Result<(PathBuf,PathBuf),String> {
    Uuid::parse_str(&server.id).map_err(|_|"Servidor no válido")?;
    let stage=Path::new(&server.directory).join(format!(".tch-loader-stage-{}",server.id));let journal=root.join("database").join(format!(".tch-loader-{}.json",server.id));
    for path in [&stage,&journal]{if fs::symlink_metadata(path).is_ok_and(|m|m.file_type().is_symlink()){return Err("La actualización no admite enlaces simbólicos".into());}}
    Ok((stage,journal))
}
fn relative_launcher(server:&Server,path:&Path)->Result<PathBuf,String> {
    // NeoForge publishes canonical Windows paths, while the server record can
    // retain the ordinary spelling. Accept either root without resolving a
    // launcher symlink or weakening the generation/path checks.
    let root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    path.strip_prefix(&server.directory).or_else(|_|path.strip_prefix(&root))
        .map(Path::to_owned).map_err(|_|"El arranque no pertenece a este servidor".into())
}
pub fn installed(server:&Server)->Result<(String,String),String> {
    let root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let jar=PathBuf::from(server.jar_path.as_ref().ok_or("Instala primero el servidor")?);
    let relative=relative_launcher(server,&jar)?;
    if !relative.starts_with(container(server)?) {
        let identity=crate::loader_identity::imported(server)?;
        let mut digest=Sha512::new();
        for value in [&server.id,&server.platform,&server.version,server.jar_path.as_ref().unwrap(),server.java_path.as_deref().unwrap_or(""),&identity.version]{digest.update(value.as_bytes());digest.update([0]);}
        for path in identity.evidence {
            digest.update(path.to_string_lossy().as_bytes());digest.update([0]);
            let mut file=fs::File::open(path).map_err(|e|e.to_string())?;let mut buffer=[0u8;65536];let mut total=0usize;
            loop {let count=file.read(&mut buffer).map_err(|e|e.to_string())?;if count==0{break;}total+=count;if total>64*1024*1024{return Err("El archivo del loader cambió durante la revisión".into());}digest.update(&buffer[..count]);}
        }
        return Ok((identity.version,format!("{:x}",digest.finalize())));
    }
    let folder=generation(server,&relative)?;
    let actual=root.join(&folder);if actual.canonicalize().map_err(|e|e.to_string())?!=actual{return Err("La generación del loader contiene enlaces".into());}
    let metadata=actual.join("installation.json");let info=fs::symlink_metadata(&metadata).map_err(|_|"No se pudo identificar la versión instalada del loader")?;
    if !info.is_file()||info.file_type().is_symlink()||info.len()>131072{return Err("Metadatos de instalación no válidos".into());}
    let bytes=fs::read(&metadata).map_err(|e|e.to_string())?;let data:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|"Metadatos del loader no válidos")?;
    if data["minecraft"].as_str()!=Some(&server.version){return Err("El loader instalado no corresponde a Minecraft".into());}
    let version=data["loader"].as_str().ok_or("No se identificó la versión del loader")?;crate::fabric::validate_version(version)?;
    let jar_meta=fs::symlink_metadata(&jar).map_err(|e|e.to_string())?;
    if !jar_meta.is_file()||jar_meta.file_type().is_symlink()||jar_meta.len()>1048576||jar.canonicalize().map_err(|e|e.to_string())?!=root.join(&relative){return Err("Arranque del loader no válido".into());}
    let mut digest=Sha512::new();for value in [&server.id,&server.platform,&server.version,server.jar_path.as_ref().unwrap(),server.java_path.as_deref().unwrap_or("")]{digest.update(value.as_bytes());digest.update([0]);}digest.update(bytes);digest.update(fs::read(jar).map_err(|e|e.to_string())?);
    Ok((version.into(),format!("{:x}",digest.finalize())))
}
fn numeric(version:&str)->Option<Vec<u32>> {let version=version.strip_prefix("1.20.1-").unwrap_or(version);let values=version.split('-').next()?.split('.').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;(values.len()==3).then_some(values)}
fn newer(target:&str,current:&str)->bool {match (numeric(target),numeric(current)){(Some(target),Some(old))=>target>old||(target==old&&current.strip_prefix("1.20.1-").unwrap_or(current).contains('-')),_=>false}}
pub fn review(server:&Server,task:&DownloadTask)->Result<Review,String> {
    let (current,fingerprint)=installed(server)?;
    let versions=if server.platform=="fabric" {
        let data=task.json(&format!("https://meta.fabricmc.net/v2/versions/loader/{}",server.version),"Consultando loaders Fabric")?;
        data.as_array().ok_or("Catálogo Fabric no válido")?.iter().filter(|entry|entry["loader"]["stable"]==true).filter_map(|entry|entry["loader"]["version"].as_str().map(str::to_owned)).collect::<Vec<_>>()
    }else{crate::neoforge::stable_loaders(&task.json(crate::neoforge::catalog_url(&server.version),"Consultando loaders NeoForge")?,&server.version)?};
    let mut available=versions.into_iter().filter(|version|newer(version,&current)).collect::<Vec<_>>();available.sort_by_key(|v|std::cmp::Reverse(numeric(v).unwrap()));available.dedup();
    Ok(Review{current,available,fingerprint})
}
fn checked_container(server:&Server)->Result<PathBuf,String> {
    let root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;let base=Path::new(&server.directory).join(container(server)?);
    fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&base).map_err(|e|e.to_string())?.file_type().is_symlink()||base.canonicalize().map_err(|e|e.to_string())?!=root.join(container(server)?){return Err("La carpeta del loader sale del servidor".into());}Ok(base)
}
pub fn recover(root:&Path,server:&Server)->Result<(),String> {
    let (stage,journal)=paths(root,server)?;
    if journal.exists(){
        if fs::metadata(&journal).map_err(|e|e.to_string())?.len()>16384{return Err("Registro de actualización no válido".into());}
        let record:Journal=serde_json::from_slice(&fs::read(&journal).map_err(|e|e.to_string())?).map_err(|_|"Registro de actualización no válido")?;
        let text=crate::modpack::relative_path(&record.new)?;let mut relative=PathBuf::new();for component in Path::new(&text).components(){relative.push(component.as_os_str());}
        let folder=generation(server,&relative)?;let new=Path::new(&server.directory).join(&relative);
        if server.jar_path.as_deref()==Some(new.to_string_lossy().as_ref()) {
            if !new.is_file(){return Err("Falta el arranque actualizado".into());}
        }else if server.jar_path.as_deref()==Some(&record.old){
            let old=relative_launcher(server,Path::new(&record.old))?;
            crate::loader_identity::checked_file(Path::new(&server.directory),Path::new(&record.old),64*1024*1024)?;
            if old.starts_with(&folder){return Err("La recuperación no puede eliminar la generación activa".into());}
            checked_container(server)?;let target=Path::new(&server.directory).join(folder);
            if target.exists(){if fs::symlink_metadata(&target).map_err(|e|e.to_string())?.file_type().is_symlink()||target.canonicalize().map_err(|e|e.to_string())?!=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?.join(target.strip_prefix(&server.directory).map_err(|e|e.to_string())?){return Err("La generación nueva contiene enlaces".into());}fs::remove_dir_all(target).map_err(|e|e.to_string())?;}
        }else{return Err("El arranque cambió durante la recuperación del loader".into());}
        fs::remove_file(&journal).map_err(|e|e.to_string())?;
    }
    if stage.exists(){fs::remove_dir_all(stage).map_err(|e|e.to_string())?;}Ok(())
}
fn publish(root:&Path,server:&Server,prepared:&Path)->Result<(),String> {
    let (stage,journal)=paths(root,server)?;let mut staged=server.clone();staged.directory=stage.to_string_lossy().into();
    let relative=relative_launcher(&staged,prepared)?;let folder=generation(server,&relative)?;
    checked_container(server)?;let target=Path::new(&server.directory).join(&folder);
    if target.exists(){return Err("La generación preparada ya existe".into());}
    let record=Journal{old:server.jar_path.clone().ok_or("Falta el arranque anterior")?,new:relative.to_string_lossy().replace('\\',"/")};
    let pending=stage.join(".tch-loader.pending");let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&pending).map_err(|e|e.to_string())?;file.write_all(&serde_json::to_vec(&record).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);fs::rename(pending,&journal).map_err(|e|e.to_string())?;
    let result=(||->Result<(),String>{fs::rename(stage.join(&folder),target).map_err(|e|e.to_string())?;storage::update_install(root,&server.id,&Path::new(&server.directory).join(relative),Path::new(server.java_path.as_ref().ok_or("Falta Java")?))})();
    recover(root,&storage::get(root,&server.id)?)?;result
}
pub fn update(root:&Path,id:&str,target:&str,fingerprint:&str,task:&DownloadTask)->Result<Option<String>,String> {
    let server=storage::get(root,id)?;recover(root,&server)?;
    let review=review(&server,task)?;
    if review.fingerprint!=fingerprint||!review.available.iter().any(|value|value==target){return Err("La instalación o el catálogo cambiaron. Revisa la actualización de nuevo.".into());}
    task.cancellation.check()?;task.report("Guardando copia antes de actualizar","preparing");let backup=crate::backup::create_tracked(root,id,task)?;
    let (stage,_)=paths(root,&server)?;fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let result=(||->Result<(),String>{let mut staged=server.clone();staged.directory=stage.to_string_lossy().into();staged.jar_path=None;let jar=crate::install::prepare_loader_update(root,&staged,task,target)?;task.cancellation.check()?;publish(root,&server,&jar)})();
    if result.is_err(){recover(root,&storage::get(root,id)?)?;}result?;Ok(backup.retention_warning)
}
pub fn recover_all(root:&Path)->Result<(),String>{for server in storage::list(root)?{if matches!(server.platform.as_str(),"fabric"|"neoforge"){recover(root,&server)?;}}Ok(())}
