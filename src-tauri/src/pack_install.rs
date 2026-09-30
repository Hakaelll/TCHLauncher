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

#[cfg(test)]
mod tests {
    use super::*;
    struct Library(PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,Server) {
        let root=std::env::temp_dir().join(format!("tch-pack-recovery-{}",Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let server=storage::create(&root,crate::model::CreateServer{name:"Pack".into(),version:"1.21.4".into(),platform:"fabric".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        (Library(root),server)
    }
    fn journal(root:&Path,server:&Server) {
        let (_,_,_,path)=paths(root,&server.id).unwrap();fs::write(path,serde_json::to_vec(&Journal{id:server.id.clone(),jar:"server.jar".into()}).unwrap()).unwrap();
    }
    #[test]
    fn interrupted_switch_restores_initial_files_before_database_commit() {
        for published in [false,true] {
            let (root,server)=fixture();let original=fs::read(Path::new(&server.directory).join("server.properties")).unwrap();
            let (target,stage,old,_)=paths(&root.0,&server.id).unwrap();fs::create_dir(&stage).unwrap();fs::write(stage.join("server.jar"),b"prepared").unwrap();journal(&root.0,&server);
            fs::rename(&target,&old).unwrap();if published {fs::rename(&stage,&target).unwrap();}
            recover(&root.0,&server.id).unwrap();
            assert_eq!(fs::read(target.join("server.properties")).unwrap(),original);assert!(!target.join("server.jar").exists());assert!(!stage.exists());assert!(!old.exists());assert!(storage::get(&root.0,&server.id).unwrap().jar_path.is_none());
        }
    }
    #[test]
    fn committed_switch_keeps_instance_and_cleans_recovery_files() {
        let (root,server)=fixture();let (target,stage,old,record)=paths(&root.0,&server.id).unwrap();fs::create_dir(&stage).unwrap();fs::write(stage.join("server.jar"),b"prepared").unwrap();
        let task=DownloadTask::new(&server.id,"pack_install",|_|{}).unwrap();commit(&root.0,&server,"server.jar",Path::new("java.exe"),&record,&task).unwrap();
        assert_eq!(fs::read(target.join("server.jar")).unwrap(),b"prepared");assert!(!old.exists());assert!(!record.exists());
        assert_eq!(storage::get(&root.0,&server.id).unwrap().jar_path,Some(target.join("server.jar").to_string_lossy().into()));
    }
    #[test]
    fn validates_journal_and_preserves_existing_servers() {
        let (root,server)=fixture();let (target,stage,_,record)=paths(&root.0,&server.id).unwrap();fs::create_dir(&stage).unwrap();
        fs::write(&record,serde_json::to_vec(&Journal{id:server.id.clone(),jar:"../outside".into()}).unwrap()).unwrap();assert!(recover(&root.0,&server.id).is_err());assert!(target.join("server.properties").is_file());
        fs::remove_file(record).unwrap();recover_all(&root.0).unwrap();assert!(!stage.exists());
        fs::write(target.join("world.dat"),b"user world").unwrap();assert!(baseline(&server).is_err());assert_eq!(fs::read(target.join("world.dat")).unwrap(),b"user world");
    }
    #[test]
    fn recovery_after_database_commit_keeps_published_generation() {
        let (root,server)=fixture();let (target,stage,old,record)=paths(&root.0,&server.id).unwrap();fs::create_dir(&stage).unwrap();fs::write(stage.join("server.jar"),b"prepared").unwrap();journal(&root.0,&server);
        fs::rename(&target,&old).unwrap();fs::rename(&stage,&target).unwrap();storage::update_install(&root.0,&server.id,&target.join("server.jar"),Path::new("java.exe")).unwrap();
        recover_all(&root.0).unwrap();assert!(target.join("server.jar").is_file());assert!(!record.exists());assert!(!old.exists());
    }

    #[test]
    #[ignore="Installs a real Fabric/Lithium mrpack and boots Java to EULA refusal without accepting it"]
    fn installs_real_pack_and_starts_published_instance() {
        use std::{process::{Command,Stdio},time::{Instant,Duration}};
        let (root,mut server)=fixture();
        let neo=std::env::var("TCH_TEST_PACK_PLATFORM").as_deref()==Ok("neoforge");
        let (minecraft,platform,loader_key,loader_version,project,mod_name)=if neo {("1.21.11","neoforge","neoforge","21.11.45","uXXizFIs","ferritecore")} else {("1.21.4","fabric","fabric-loader","0.16.10","gvQqBUqZ","lithium")};
        if neo {storage::connect(&root.0).unwrap().execute("UPDATE servers SET version=?2,platform=?3 WHERE id=?1",rusqlite::params![server.id,minecraft,platform]).unwrap();server.version=minecraft.into();server.platform=platform.into();}
        let task=DownloadTask::new(&server.id,"pack_install",|p|{if p.stage!="downloading" {eprintln!("{}: {}",p.stage,p.label);}}).unwrap();
        let versions=task.json(&format!("https://api.modrinth.com/v2/project/{project}/version?game_versions=%5B%22{minecraft}%22%5D&loaders=%5B%22{platform}%22%5D"),"Consultando mod del paquete").unwrap();
        let version=versions.as_array().unwrap().iter().find(|v|v["version_type"]=="release").unwrap();let artifact=version["files"].as_array().unwrap().iter().find(|f|f["primary"]==true).unwrap();
        let name=format!("mods/{}",artifact["filename"].as_str().unwrap());
        let index=serde_json::json!({"formatVersion":1,"game":"minecraft","versionId":"test","name":"Lithium server pack","dependencies":{"minecraft":minecraft,(loader_key):loader_version},"files":[{"path":name,"hashes":artifact["hashes"],"downloads":[artifact["url"]],"fileSize":artifact["size"],"env":{"client":"required","server":"required"}}]});
        let archive=root.0.join("lithium.mrpack");let mut zip=zip::ZipWriter::new(fs::File::create(&archive).unwrap());let options=zip::write::SimpleFileOptions::default();
        for (name,body) in [("modrinth.index.json",index.to_string()),("overrides/config/example.txt","common".into()),("server-overrides/config/example.txt","server".into()),("server-overrides/eula.txt","eula=true".into())] {zip.start_file(name,options).unwrap();zip.write_all(body.as_bytes()).unwrap();}zip.finish().unwrap();
        let review=modpack::inspect(&archive).unwrap().inspection;let selected=review.entries.iter().filter(|e|e.selected).map(|e|e.path.clone()).collect::<Vec<_>>();
        let properties=fs::read(Path::new(&server.directory).join("server.properties")).unwrap();
        install(&root.0,&server,&archive,&review.archive_sha512,&selected,&task).unwrap();
        let installed=storage::get(&root.0,&server.id).unwrap();let directory=Path::new(&installed.directory);
        assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);assert_eq!(fs::read_to_string(directory.join("config/example.txt")).unwrap(),"server");assert!(directory.join(&name).is_file());assert!(!directory.join("eula.txt").exists());
        let metadata:serde_json::Value=serde_json::from_slice(&fs::read(Path::new(installed.jar_path.as_ref().unwrap()).ancestors().find(|path|path.join("installation.json").is_file()).unwrap().join("installation.json")).unwrap()).unwrap();assert_eq!(metadata["loader"],loader_version);
        let output_path=root.0.join("boot.log");let output=fs::File::create(&output_path).unwrap();let mut command=Command::new(installed.java_path.as_ref().unwrap());
        #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let args=crate::jvm::launch_arguments(&installed,&crate::jvm::Settings::default()).unwrap();
        let mut child=command.args(args).current_dir(directory).stdin(Stdio::null()).stdout(output.try_clone().unwrap()).stderr(output).spawn().unwrap();let began=Instant::now();
        loop {if child.try_wait().unwrap().is_some(){break;}if began.elapsed()>Duration::from_secs(90){let _=child.kill();let _=child.wait();panic!("Pack did not reach EULA gate");}std::thread::sleep(Duration::from_millis(100));}
        let log=fs::read_to_string(output_path).unwrap();if !neo {assert!(log.contains("Fabric Loader 0.16.10"),"{log}");}assert!(log.to_lowercase().contains(mod_name),"{log}");assert!(log.to_lowercase().contains("agree to the eula"),"{log}");
        assert!(fs::read_to_string(directory.join("eula.txt")).unwrap().lines().any(|line|line.trim()=="eula=false"));assert!(!crate::eula::recorded(&root.0).unwrap());assert!(!directory.join("world/level.dat").exists());
        recover_all(&root.0).unwrap();assert!(directory.join(name).is_file());assert!(archive.is_file());
    }
}
