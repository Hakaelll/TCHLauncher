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
fn numeric(version:&str)->Option<Vec<u32>> {let values=version.split('-').next()?.split('.').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;(values.len()==3).then_some(values)}
fn newer(target:&str,current:&str)->bool {match (numeric(target),numeric(current)){(Some(target),Some(old))=>target>old||(target==old&&current.contains('-')),_=>false}}
pub fn review(server:&Server,task:&DownloadTask)->Result<Review,String> {
    let (current,fingerprint)=installed(server)?;
    let versions=if server.platform=="fabric" {
        let data=task.json(&format!("https://meta.fabricmc.net/v2/versions/loader/{}",server.version),"Consultando loaders Fabric")?;
        data.as_array().ok_or("Catálogo Fabric no válido")?.iter().filter(|entry|entry["loader"]["stable"]==true).filter_map(|entry|entry["loader"]["version"].as_str().map(str::to_owned)).collect::<Vec<_>>()
    }else{crate::neoforge::stable_loaders(&task.json("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge","Consultando loaders NeoForge")?,&server.version)?};
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore="Official downloads and Java; set TCH_TEST_JAVA and optionally TCH_TEST_UPDATE_PLATFORM=neoforge"]
    fn updates_real_loader_and_restores_previous_generation() {
        let platform=std::env::var("TCH_TEST_UPDATE_PLATFORM").unwrap_or_else(|_|"fabric".into());
        assert!(matches!(platform.as_str(),"fabric"|"neoforge"));
        let root=std::env::temp_dir().join(format!("tch loader live {}",Uuid::new_v4()));
        let result=(||->Result<(),String>{
            fs::create_dir_all(&root).map_err(|e|e.to_string())?;
            let java=PathBuf::from(std::env::var("TCH_TEST_JAVA").map_err(|_|"Set TCH_TEST_JAVA")?).canonicalize().map_err(|e|e.to_string())?;
            let runtime=java.parent().and_then(Path::parent).ok_or("Missing runtime directory")?;
            crate::backup::copy_tree(runtime,&root.join("runtimes").join(format!("temurin-21-{}-{}",std::env::consts::OS,std::env::consts::ARCH)))?;
            let mut server=storage::create(&root,crate::model::CreateServer{name:"Live loader update".into(),version:"1.21.4".into(),platform:platform.clone(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None})?;
            let task=DownloadTask::new(&server.id,"loader_update",|event|{if event.stage!="downloading"{eprintln!("{}: {}",event.stage,event.label);}})?;
            let initial=if platform=="fabric" {"0.16.10".to_owned()}else{
                let data=task.json("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge","Reviewing official NeoForge catalog")?;
                crate::neoforge::stable_loaders(&data,&server.version)?.get(1).cloned().ok_or("Need two stable NeoForge versions")?
            };
            crate::install::install_requested(&root,&server,&task,Some(&initial))?;
            if std::env::var_os("TCH_TEST_UPDATE_IMPORTED").is_some() {
                let original=storage::get(&root,&server.id)?;let jar=Path::new(original.jar_path.as_ref().unwrap());
                let relative=relative_launcher(&original,jar)?;let generation=generation(&original,&relative)?;
                let source=Path::new(&original.directory).join(&generation);let external=root.join("existing server");
                crate::backup::copy_tree(&source,&external)?;
                fs::copy(Path::new(&original.directory).join("server.properties"),external.join("server.properties")).map_err(|e|e.to_string())?;
                fs::remove_file(external.join("installation.json")).map_err(|e|e.to_string())?;
                if platform=="fabric"{fs::rename(external.join("vanilla.jar"),external.join("server.jar")).map_err(|e|e.to_string())?;}
                let launcher=relative.strip_prefix(&generation).map_err(|e|e.to_string())?.to_string_lossy().replace('\\',"/");
                server=storage::import(&root,crate::model::ImportServer{path:external.to_string_lossy().into(),name:"Imported loader update".into(),version:original.version,platform:platform.clone(),jar_file:launcher,copy:false,storage_target_gb:2,image_path:None})?;
                storage::update_java(&root,&server.id,Path::new(original.java_path.as_ref().unwrap()))?;
            }
            let task=DownloadTask::new(&server.id,"loader_update",|event|{if event.stage!="downloading"{eprintln!("{}: {}",event.stage,event.label);}})?;
            let before=storage::get(&root,&server.id)?;
            let settings=crate::jvm::Settings{mode:"custom".into(),arguments:"-Dfile.encoding=UTF-8".into()};
            crate::jvm::save(&root,&server.id,&settings)?;
            let directory=Path::new(&server.directory);
            fs::create_dir_all(directory.join("config")).map_err(|e|e.to_string())?;
            fs::write(directory.join("config/preserved.txt"),b"preserved configuration").map_err(|e|e.to_string())?;
            let properties=fs::read(directory.join("server.properties")).map_err(|e|e.to_string())?;
            let reviewed=review(&before,&task)?;assert_eq!(reviewed.current,initial);
            let target=reviewed.available.first().ok_or("No newer stable loader")?;
            update(&root,&server.id,target,&reviewed.fingerprint,&task)?;
            let updated=storage::get(&root,&server.id)?;
            assert_eq!(installed(&updated)?.0,*target);assert_ne!(updated.jar_path,before.jar_path);
            assert_eq!(updated.java_path,before.java_path);assert!(Path::new(before.jar_path.as_ref().unwrap()).is_file());
            assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);
            assert_eq!(crate::jvm::get(&root,&server.id)?,settings);
            let boot=|server:&Server,label:&str|->Result<(),String>{
                use std::process::{Command,Stdio};use std::time::{Instant,Duration};
                let log=root.join(format!("{label}.log"));let output=fs::File::create(&log).map_err(|e|e.to_string())?;
                let mut command=Command::new(server.java_path.as_ref().unwrap());
                #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
                let mut child=command.args(crate::jvm::launch_arguments(server,&settings)?).current_dir(&server.directory).stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output).spawn().map_err(|e|e.to_string())?;
                let began=Instant::now();loop {
                    match child.try_wait(){Ok(Some(_))=>break,Ok(None)=>(),Err(error)=>{let _=child.kill();let _=child.wait();return Err(error.to_string());}}
                    if began.elapsed()>Duration::from_secs(90){let _=child.kill();let _=child.wait();return Err(format!("Timed out; inspect {}",log.display()));}
                    std::thread::sleep(Duration::from_millis(100));
                }
                let text=fs::read_to_string(&log).map_err(|e|e.to_string())?;
                assert!(text.to_lowercase().contains("eula"),"{text}");
                assert!(fs::read_to_string(directory.join("eula.txt")).map_err(|e|e.to_string())?.lines().any(|line|line.trim()=="eula=false"));
                assert!(!directory.join("world/level.dat").exists());Ok(())
            };
            boot(&updated,"updated")?;
            let backups=storage::list_backups(&root,&server.id)?;assert_eq!(backups.len(),1);
            crate::backup::restore(&root,&server.id,&backups[0].id)?;
            let restored=storage::get(&root,&server.id)?;
            assert_eq!(installed(&restored)?.0,initial);
            assert_eq!(Path::new(restored.jar_path.as_ref().unwrap()).canonicalize().unwrap(),Path::new(before.jar_path.as_ref().unwrap()).canonicalize().unwrap());
            assert_eq!(restored.java_path,before.java_path);
            assert_eq!(crate::jvm::get(&root,&server.id)?,settings);
            assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);
            assert_eq!(fs::read(directory.join("config/preserved.txt")).unwrap(),b"preserved configuration");
            boot(&restored,"restored")?;
            assert_eq!(storage::list_backups(&root,&server.id)?.len(),2);
            assert!(!crate::eula::recorded(&root)?);
            let (stage,journal)=paths(&root,&server)?;assert!(!stage.exists());assert!(!journal.exists());
            eprintln!("Verified {platform}: {initial} -> {target} -> {initial}");Ok(())
        })();
        if result.is_ok(){let _=fs::remove_dir_all(&root);}else{eprintln!("Loader diagnostics retained at {}",root.display());}
        result.unwrap();
    }
    struct Library(PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,Server) {
        let root=std::env::temp_dir().join(format!("tch-loader-update-{}",Uuid::new_v4()));let server=storage::create(&root,crate::model::CreateServer{name:"Loader".into(),version:"1.21.4".into(),platform:"fabric".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        let folder=Path::new(&server.directory).join(".tch-fabric").join(Uuid::new_v4().to_string());fs::create_dir_all(&folder).unwrap();let jar=folder.join("fabric-server-launch.jar");fs::write(&jar,b"old launcher").unwrap();fs::write(folder.join("installation.json"),br#"{"minecraft":"1.21.4","loader":"0.16.10"}"#).unwrap();storage::update_install(&root,&server.id,&jar,Path::new("java.exe")).unwrap();let server=storage::get(&root,&server.id).unwrap();(Library(root),server)
    }
    fn prepared(root:&Path,server:&Server)->PathBuf {
        let (stage,_)=paths(root,server).unwrap();let folder=stage.join(".tch-fabric").join(Uuid::new_v4().to_string());fs::create_dir_all(&folder).unwrap();let jar=folder.join("fabric-server-launch.jar");fs::write(&jar,b"new launcher").unwrap();fs::write(folder.join("installation.json"),br#"{"minecraft":"1.21.4","loader":"0.19.0"}"#).unwrap();jar
    }
    #[test]
    fn imported_launcher_survives_publication_failure_and_backup_restore() {
        use std::io::Write;
        for fail in [false,true] {
            let (root,server)=fixture();let launcher=Path::new(&server.directory).join("renamed.jar");
            let mut zip=zip::ZipWriter::new(fs::File::create(&launcher).unwrap());
            for (name,text) in [("META-INF/MANIFEST.MF","Manifest-Version: 1.0\nMain-Class: net.fabricmc.installer.ServerLauncher\n\n"),("install.properties","fabric-loader-version=0.16.10\ngame-version=1.21.4\n")] {
                zip.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();zip.write_all(text.as_bytes()).unwrap();
            }
            zip.finish().unwrap();storage::update_install(&root.0,&server.id,&launcher,Path::new("java.exe")).unwrap();let server=storage::get(&root.0,&server.id).unwrap();
            assert_eq!(installed(&server).unwrap().0,"0.16.10");let bytes=fs::read(&launcher).unwrap();let backup=crate::backup::create(&root.0,&server.id).unwrap();let jar=prepared(&root.0,&server);
            if fail{storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER fail_loader BEFORE UPDATE ON servers BEGIN SELECT RAISE(ABORT,'failure'); END;").unwrap();}
            assert_eq!(publish(&root.0,&server,&jar).is_err(),fail);
            assert_eq!(fs::read(&launcher).unwrap(),bytes);
            if fail{assert_eq!(storage::get(&root.0,&server.id).unwrap().jar_path,server.jar_path);}else{
                assert_eq!(installed(&storage::get(&root.0,&server.id).unwrap()).unwrap().0,"0.19.0");
                crate::backup::restore(&root.0,&server.id,&backup.id).unwrap();let restored=storage::get(&root.0,&server.id).unwrap();
                assert_eq!(installed(&restored).unwrap().0,"0.16.10");assert_eq!(restored.jar_path,server.jar_path);
            }
            let (stage,journal)=paths(&root.0,&server).unwrap();assert!(!stage.exists());assert!(!journal.exists());
        }
    }
    #[test]
    fn canonical_launch_path_is_identified_and_external_path_rejected() {
        let (root,mut server)=fixture();
        let jar=Path::new(server.jar_path.as_ref().unwrap()).canonicalize().unwrap();
        server.jar_path=Some(jar.to_string_lossy().into());
        assert_eq!(installed(&server).unwrap().0,"0.16.10");
        let next=prepared(&root.0,&server).canonicalize().unwrap();publish(&root.0,&server,&next).unwrap();
        assert_eq!(installed(&storage::get(&root.0,&server.id).unwrap()).unwrap().0,"0.19.0");
        assert!(relative_launcher(&server,&root.0.join("external.jar")).is_err());
    }
    #[test]
    fn successful_publication_keeps_old_generation_world_and_configuration() {
        let (root,server)=fixture();let directory=Path::new(&server.directory);fs::create_dir(directory.join("world")).unwrap();fs::write(directory.join("world/level.dat"),b"world").unwrap();let properties=fs::read(directory.join("server.properties")).unwrap();let jar=prepared(&root.0,&server);
        publish(&root.0,&server,&jar).unwrap();let current=storage::get(&root.0,&server.id).unwrap();assert_eq!(installed(&current).unwrap().0,"0.19.0");assert!(Path::new(server.jar_path.as_ref().unwrap()).is_file());assert_eq!(current.java_path,server.java_path);assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);assert_eq!(fs::read(directory.join("world/level.dat")).unwrap(),b"world");assert!(!paths(&root.0,&server).unwrap().0.exists());
    }
    #[test]
    fn database_failure_removes_unpublished_generation_and_preserves_old_launcher() {
        let (root,mut server)=fixture();let old=Path::new(server.jar_path.as_ref().unwrap()).canonicalize().unwrap();
        storage::update_install(&root.0,&server.id,&old,Path::new("java.exe")).unwrap();server=storage::get(&root.0,&server.id).unwrap();
        let jar=prepared(&root.0,&server);let relative=jar.strip_prefix(paths(&root.0,&server).unwrap().0).unwrap();let new=Path::new(&server.directory).join(relative);
        storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER fail_loader BEFORE UPDATE ON servers BEGIN SELECT RAISE(ABORT,'failure'); END;").unwrap();assert!(publish(&root.0,&server,&jar).is_err());assert_eq!(storage::get(&root.0,&server.id).unwrap().jar_path,server.jar_path);assert!(!new.exists());assert!(Path::new(server.jar_path.as_ref().unwrap()).is_file());
    }
    #[test]
    fn interrupted_publish_recovers_before_and_after_commit() {
        for committed in [false,true] {
            let (root,server)=fixture();let jar=prepared(&root.0,&server);let (stage,journal)=paths(&root.0,&server).unwrap();let relative=jar.strip_prefix(&stage).unwrap();let folder=generation(&server,relative).unwrap();let new=Path::new(&server.directory).join(relative);
            fs::write(&journal,serde_json::to_vec(&Journal{old:server.jar_path.clone().unwrap(),new:relative.to_string_lossy().replace('\\',"/")}).unwrap()).unwrap();fs::rename(stage.join(&folder),Path::new(&server.directory).join(folder)).unwrap();if committed{storage::update_install(&root.0,&server.id,&new,Path::new("java.exe")).unwrap();}
            recover_all(&root.0).unwrap();assert_eq!(new.is_file(),committed);assert!(!stage.exists());assert!(!journal.exists());assert!(Path::new(server.jar_path.as_ref().unwrap()).is_file());
        }
    }
    #[test]
    fn active_generation_is_protected_and_fingerprint_detects_changes() {
        let (root,server)=fixture();let first=installed(&server).unwrap();fs::write(server.jar_path.as_ref().unwrap(),b"modified launcher").unwrap();assert_ne!(installed(&server).unwrap().1,first.1);
        let (_,journal)=paths(&root.0,&server).unwrap();let folder=Path::new(server.jar_path.as_ref().unwrap()).parent().unwrap();let other=folder.join("other.jar");let relative=other.strip_prefix(&server.directory).unwrap();fs::write(&journal,serde_json::to_vec(&Journal{old:server.jar_path.clone().unwrap(),new:relative.to_string_lossy().replace('\\',"/")}).unwrap()).unwrap();assert!(recover_all(&root.0).is_err());assert!(Path::new(server.jar_path.as_ref().unwrap()).is_file());assert!(newer("0.19.0","0.16.10"));assert!(!newer("0.16.9","0.16.10"));
    }
}
