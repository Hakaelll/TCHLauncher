use crate::{download::DownloadTask,model::Server,content_storage::InstalledContent};
use serde::Serialize;
use sha2::{Digest,Sha512};
use std::{fs,io::Read,path::Path};

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ContentCheck {pub title:String,pub filename:String,pub status:String,pub reason:String}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Review {
    pub current_minecraft:String,pub target_minecraft:String,pub platform:String,
    pub current_loader:Option<String>,pub target_loader:Option<String>,pub java_major:u64,
    pub java_action:String,pub content:Vec<ContentCheck>,pub blockers:Vec<String>,pub warnings:Vec<String>,pub fingerprint:String,
    pub changes:Vec<crate::content_storage::ContentChange>,pub migration:Option<crate::content::ContentPlan>,
}
fn release_time(metadata:&serde_json::Value)->Result<chrono::DateTime<chrono::FixedOffset>,String> {
    chrono::DateTime::parse_from_rfc3339(metadata["releaseTime"].as_str().ok_or("Minecraft no indicó la fecha de publicación")?).map_err(|_|"Fecha de publicación Minecraft no válida".into())
}
fn check_target(current:&serde_json::Value,target:&serde_json::Value)->Result<(),String> {
    if target["type"].as_str()!=Some("release"){return Err("Selecciona una versión estable de Minecraft".into());}
    if release_time(target)?<=release_time(current)?{return Err("Elige una versión posterior. Para volver a una versión anterior, restaura su copia de seguridad.".into());}Ok(())
}
fn check_content(file:&InstalledContent,version:Option<&crate::content::ContentVersion>,target:&str,platform:&str)->ContentCheck {
    let result=|status:&str,reason:&str|ContentCheck{title:file.title.clone(),filename:file.filename.clone(),status:status.into(),reason:reason.into()};
    if file.modified||file.project_id.is_none()||file.version_id.is_none(){return result("unknown","Archivo manual o modificado: hay que comprobar su compatibilidad.");}
    let Some(version)=version else{return result("unknown","No se pudo comprobar esta versión en el catálogo.");};
    if file.version_id.as_deref()!=Some(&version.id)||file.project_id.as_deref()!=Some(&version.project_id)||!version.files.iter().any(|artifact|artifact.size==file.size&&artifact.hashes.get("sha512").is_some_and(|hash|hash.eq_ignore_ascii_case(&file.sha512))) {
        return result("unknown","El archivo instalado no coincide con la versión declarada por el catálogo.");
    }
    if version.environment.as_deref()==Some("client") {
        return result("incompatible","Esta versión está declarada para el cliente, no para el servidor.");
    }
    let supported=match platform{"paper"=>&["paper","spigot","bukkit"][..],"fabric"=>&["fabric"][..],"neoforge"=>&["neoforge"][..],_=>&[][..]};
    if !version.game_versions.iter().any(|game|game==target)||!version.loaders.iter().any(|loader|supported.contains(&loader.as_str())) {
        return result("incompatible","La versión instalada no declara compatibilidad con el Minecraft o motor elegido.");
    }
    result("compatible","El catálogo declara compatibilidad de esta versión y el archivo coincide con su hash.")
}
fn fingerprint(root:&Path,server:&Server,files:&[InstalledContent],review:&Review)->Result<String,String> {
    let mut digest=Sha512::new();
    for value in [&server.id,&server.directory,&server.platform,&server.version,server.jar_path.as_deref().unwrap_or(""),server.java_path.as_deref().unwrap_or("")] {digest.update(value.as_bytes());digest.update([0]);}
    digest.update(server.memory_min_mb.to_le_bytes());digest.update(server.memory_max_mb.to_le_bytes());digest.update(server.storage_target_gb.to_le_bytes());
    digest.update(serde_json::to_vec(&crate::jvm::get(root,&server.id)?).map_err(|e|e.to_string())?);
    digest.update(serde_json::to_vec(files).map_err(|e|e.to_string())?);
    // Exclude only the fingerprint itself. This binds the review to the chosen
    // release, runtime action, loader/build and every compatibility result.
    let mut evidence=review.clone();evidence.fingerprint.clear();
    digest.update(serde_json::to_vec(&evidence).map_err(|e|e.to_string())?);
    if matches!(server.platform.as_str(),"fabric"|"neoforge") {
        // The loader's identity includes installation metadata and imported
        // launcher evidence, which must also invalidate a stale review.
        digest.update(crate::loader_update::installed(server)?.1.as_bytes());
    }
    let jar=Path::new(server.jar_path.as_deref().ok_or("Instala primero el servidor")?);
    crate::loader_identity::checked_file(Path::new(&server.directory),jar,512_000_000)?;
    let mut input=fs::File::open(jar).map_err(|e|e.to_string())?;let mut buffer=[0u8;65536];let mut total=0u64;
    loop {let count=input.read(&mut buffer).map_err(|e|e.to_string())?;if count==0{break;}total+=count as u64;if total>512_000_000{return Err("El arranque cambió durante la revisión".into());}digest.update(&buffer[..count]);}
    let properties=Path::new(&server.directory).join("server.properties");crate::loader_identity::checked_file(Path::new(&server.directory),&properties,2_000_000)?;digest.update(fs::read(properties).map_err(|e|e.to_string())?);
    Ok(format!("{:x}",digest.finalize()))
}
pub fn review(root:&Path,server:&Server,target:&str,task:&DownloadTask)->Result<Review,String> {
    crate::fabric::validate_version(target)?;if target==server.version{return Err("Ya estás usando esta versión de Minecraft".into());}
    let current_metadata=crate::install::version_metadata(task,&server.version)?;let target_metadata=crate::install::version_metadata(task,target)?;
    if current_metadata["id"].as_str()!=Some(&server.version)||target_metadata["id"].as_str()!=Some(target){return Err("Los metadatos no corresponden a la versión de Minecraft elegida".into());}
    check_target(&current_metadata,&target_metadata)?;
    let major=target_metadata["javaVersion"]["majorVersion"].as_u64().ok_or("Minecraft no indicó el runtime necesario")?;
    let mut reviewed=Review{current_minecraft:server.version.clone(),target_minecraft:target.into(),platform:server.platform.clone(),current_loader:None,target_loader:None,java_major:major,java_action:"prepare_managed".into(),content:vec![],blockers:vec![],warnings:vec!["La primera apertura puede convertir el mundo. Para volver atrás tendrás que restaurar la copia previa, incluidos los mundos.".into()],fingerprint:String::new(),changes:vec![],migration:None};
    if let Some(java)=&server.java_path {
        let version=crate::runtime::version_output(Path::new(java)).ok().and_then(|output|crate::runtime::parse_version(&output));
        if version.as_deref().is_some_and(|version|crate::runtime::check_compatibility(version,major).is_ok()){reviewed.java_action="retain_configured".into();}
    }
    match server.platform.as_str() {
        "fabric"=>{
            reviewed.current_loader=Some(crate::loader_update::installed(server)?.0);
            let data=task.json(&format!("https://meta.fabricmc.net/v2/versions/loader/{target}"),"Revisando Fabric para el nuevo Minecraft")?;
            reviewed.target_loader=Some(crate::fabric::select_loader(&data)?);
        },
        "neoforge"=>{
            reviewed.current_loader=Some(crate::loader_update::installed(server)?.0);
            let data=task.json("https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge","Revisando NeoForge para el nuevo Minecraft")?;
            reviewed.target_loader=Some(crate::neoforge::requested_loader(&data,target,None)?);
        },
        "paper"=>{
            let data=task.json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{target}/builds"),"Revisando Paper para el nuevo Minecraft")?;
            let build=data.as_array().ok_or("Catálogo Paper no válido")?.iter().find(|build|build["channel"].as_str()==Some("STABLE")).ok_or("Paper no tiene un build estable para esta versión")?;
            reviewed.target_loader=Some(build["id"].as_u64().ok_or("Paper no identificó el build")?.to_string());
        },
        "vanilla"=>(),_=>return Err("Motor Minecraft no compatible".into()),
    }
    let files=if server.platform=="vanilla"{vec![]}else{crate::content_storage::list(server)?};
    let mut projects=vec![];
    for file in &files {
        task.cancellation.check()?;
        let metadata=if !file.modified&&file.project_id.is_some() {
            match file.version_id.as_deref().map(|id|crate::content::installed_version(task,id)).transpose() {
                Ok(value)=>value,
                Err(error)=>{task.cancellation.check()?;reviewed.warnings.push(format!("{}: no se pudo consultar el catálogo ({error}).",file.title));None},
            }
        }else{None};
        let checked=check_content(file,metadata.as_ref(),target,&server.platform);
        if checked.status!="unknown" {
            if let Some(project)=&file.project_id {projects.push(project.clone());}
        }
        if checked.status=="unknown"{reviewed.warnings.push(format!("{}: compatibilidad sin verificar.",file.title));}
        reviewed.content.push(checked);
    }
    if !projects.is_empty() {
        let mut destination=server.clone();destination.version=target.into();
        let planned=(||->Result<crate::content_storage::PreparedPlan,String>{
            let plan=crate::content::migration(task,&destination,&projects)?;
            crate::content::check_existing(task,&files,&plan)?;
            crate::content_storage::review(&destination,plan)
        })();
        match planned {
            Ok(prepared)=>{
                reviewed.changes=prepared.review.changes;
                reviewed.warnings.extend(prepared.review.warnings);
                reviewed.migration=Some(prepared.review.plan);
                for (checked,file) in reviewed.content.iter_mut().zip(&files) {
                    if reviewed.migration.as_ref().is_some_and(|plan|plan.entries.iter().any(|entry|
                        file.project_id.as_deref()==Some(&entry.project.id)&&
                        (file.version_id.as_deref()!=Some(&entry.version.id)||file.filename!=entry.file.filename||!file.sha512.eq_ignore_ascii_case(&entry.file.hashes["sha512"]))
                    )) {
                        checked.status="update".into();
                        checked.reason="Se instalará la versión compatible indicada en el plan de cambios.".into();
                    }
                }
            },
            Err(error)=>{task.cancellation.check()?;reviewed.blockers.push(format!("No se pudo preparar una migración compatible del contenido: {error}"));},
        }
    }
    reviewed.fingerprint=fingerprint(root,server,&files,&reviewed)?;Ok(reviewed)
}

pub fn update(root:&Path,id:&str,target:&str,expected:&str,confirmed_warnings:bool,task:&DownloadTask)->Result<Option<String>,String> {
    crate::instance_publication::recover(root,id)?;
    let before=crate::storage::get(root,id)?;
    let reviewed=review(root,&before,target,task)?;
    if reviewed.fingerprint!=expected{return Err("La instalación o el catálogo cambiaron. Revisa de nuevo el cambio de Minecraft.".into());}
    if !reviewed.blockers.is_empty(){return Err(format!("Hay contenido incompatible: {}",reviewed.blockers.join(" ")));}
    if !reviewed.warnings.is_empty()&&!confirmed_warnings{return Err("Confirma los avisos de compatibilidad y conversión del mundo antes de actualizar".into());}
    task.cancellation.check()?;
    task.report("Guardando copia antes de cambiar Minecraft","preparing");
    let backup=crate::backup::create_tracked(root,id,task)?;
    task.cancellation.check()?;
    let (directory,stage,_,_)=crate::instance_publication::paths(root,&before)?;
    let result=(||->Result<(),String>{
        task.report("Preparando una copia separada del servidor","preparing");
        crate::backup::copy_tree(&directory,&stage)?;
        task.cancellation.check()?;
        let mut staged=before.clone();staged.directory=stage.to_string_lossy().into();staged.version=target.into();staged.jar_path=None;
        if reviewed.java_action!="retain_configured"{staged.java_path=None;}
        if let Some(plan)=reviewed.migration.clone() {
            task.report("Preparando mods o plugins para el nuevo Minecraft","preparing");
            crate::content_storage::install_staged(&staged,plan,task)?;
        }
        let (jar,java)=crate::install::prepare_fresh_instance(root,&staged,task,reviewed.target_loader.as_deref())?;
        task.cancellation.check()?;
        let current=crate::storage::get(root,id)?;
        let files=if current.platform=="vanilla"{vec![]}else{crate::content_storage::list(&current)?};
        if fingerprint(root,&current,&files,&reviewed)?!=expected{return Err("Los archivos o ajustes cambiaron durante la preparación. La instalación original se conserva.".into());}
        task.report("Confirmando la instalación preparada","verifying");
        // Cancellation ends before the atomic publication/recovery section.
        task.cancellation.check()?;
        crate::instance_publication::publish(root,&before,target,&jar,&java)
    })();
    if result.is_err(){crate::instance_publication::recover(root,id)?;}
    result?;Ok(backup.retention_warning)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::content::{ContentFile,ContentVersion};

    #[test]
    #[ignore="Official Minecraft/loader downloads and real Java; set TCH_TEST_JAVA and optionally TCH_TEST_UPDATE_PLATFORM"]
    fn updates_real_minecraft_and_restores_previous_installation() {
        use std::process::{Command,Stdio};
        use std::time::{Duration,Instant};
        let java=std::path::PathBuf::from(std::env::var("TCH_TEST_JAVA").expect("Set TCH_TEST_JAVA"));
        let platform=std::env::var("TCH_TEST_UPDATE_PLATFORM").unwrap_or_else(|_|"vanilla".into());
        let target=std::env::var("TCH_TEST_UPDATE_MINECRAFT").unwrap_or_else(|_|if platform=="paper"{"1.21.8".into()}else{"1.21.5".into()});
        let root=std::env::temp_dir().join(format!("tch minecraft update live {}",uuid::Uuid::new_v4()));
        let result=(||->Result<(),String>{
            let mut server=crate::storage::create(&root,crate::model::CreateServer{name:"Minecraft upgrade".into(),version:"1.21.4".into(),platform:platform.clone(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None})?;
            server.java_path=Some(java.to_string_lossy().into());
            let task=DownloadTask::new(&server.id,"minecraft_update",|event|{if event.stage!="downloading"{eprintln!("{}: {}",event.stage,event.label);}})?;
            let (jar,prepared_java)=crate::install::prepare_fresh_instance(&root,&server,&task,None)?;
            crate::storage::update_install(&root,&server.id,&jar,&prepared_java)?;
            let before=crate::storage::get(&root,&server.id)?;
            let migration_project=std::env::var("TCH_TEST_UPDATE_CONTENT").ok();
            if let Some(project)=&migration_project {
                let plan=crate::content::plan(&task,&before,project,None)?;
                let prepared=crate::content_storage::review(&before,plan)?;
                crate::content_storage::install(&root,&before,&prepared,&task)?;
            }
            let original_content=if platform=="vanilla"{vec![]}else{crate::content_storage::list(&before)?};
            let settings=crate::jvm::Settings{mode:"custom".into(),arguments:"-Dfile.encoding=UTF-8".into()};crate::jvm::save(&root,&server.id,&settings)?;
            let directory=Path::new(&server.directory);fs::create_dir_all(directory.join("config")).map_err(|e|e.to_string())?;
            fs::write(directory.join("config/preserved.txt"),b"preserved settings").map_err(|e|e.to_string())?;
            let properties=fs::read(directory.join("server.properties")).map_err(|e|e.to_string())?;
            let reviewed=review(&root,&before,&target,&task)?;
            assert!(reviewed.blockers.is_empty());assert_eq!(reviewed.java_action,"retain_configured");assert_eq!(reviewed.java_major,21);
            if migration_project.is_some() {
                assert!(!reviewed.changes.is_empty());
                let plan=reviewed.migration.as_ref().ok_or("Missing content migration plan")?;
                assert!(plan.entries.iter().all(|entry|entry.version.game_versions.contains(&target)&&entry.version.version_type=="release"));
            }
            assert!(update(&root,&server.id,&target,&reviewed.fingerprint,false,&task).is_err());
            assert!(crate::storage::list_backups(&root,&server.id)?.is_empty());
            update(&root,&server.id,&target,&reviewed.fingerprint,true,&task)?;
            let updated=crate::storage::get(&root,&server.id)?;assert_eq!(updated.version,target);assert_eq!(updated.java_path,before.java_path);
            assert_eq!(crate::jvm::get(&root,&server.id)?,settings);
            assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);
            assert_eq!(fs::read(directory.join("config/preserved.txt")).unwrap(),b"preserved settings");
            if let Some(plan)=&reviewed.migration {
                let actual=crate::content_storage::list(&updated)?;
                for entry in &plan.entries {
                    assert!(actual.iter().any(|file|file.project_id.as_deref()==Some(&entry.project.id)&&file.version_id.as_deref()==Some(&entry.version.id)&&file.sha512.eq_ignore_ascii_case(&entry.file.hashes["sha512"])&&!file.modified));
                }
            }
            let boot=|server:&Server,label:&str|->Result<(),String>{
                let log=root.join(format!("{label}.log"));let output=fs::File::create(&log).map_err(|e|e.to_string())?;
                let mut command=Command::new(server.java_path.as_ref().ok_or("Missing runtime")?);
                #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
                let mut child=command.args(crate::jvm::launch_arguments(server,&settings)?).current_dir(&server.directory).stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output).spawn().map_err(|e|e.to_string())?;
                let began=Instant::now();loop{
                    match child.try_wait(){Ok(Some(_))=>break,Ok(None)=>(),Err(error)=>{let _=child.kill();let _=child.wait();return Err(error.to_string());}}
                    if began.elapsed()>Duration::from_secs(90){let _=child.kill();let _=child.wait();return Err(format!("Java timed out; inspect {}",log.display()));}
                    std::thread::sleep(Duration::from_millis(100));
                }
                let text=fs::read_to_string(&log).map_err(|e|e.to_string())?;
                assert!(text.to_lowercase().contains("eula"),"{text}");
                assert!(fs::read_to_string(directory.join("eula.txt")).map_err(|e|e.to_string())?.lines().any(|line|line.trim()=="eula=false"));
                assert!(!directory.join("world/level.dat").exists());Ok(())
            };
            boot(&updated,"updated")?;
            let backups=crate::storage::list_backups(&root,&server.id)?;assert_eq!(backups.len(),1);
            crate::backup::restore(&root,&server.id,&backups[0].id)?;
            let restored=crate::storage::get(&root,&server.id)?;assert_eq!(restored.version,"1.21.4");assert!(restored.java_path.is_none());
            // Cross-version restores deliberately require runtime verification.
            // This harness explicitly restores the already verified Java 21.
            crate::storage::update_java(&root,&server.id,&java)?;
            let restored=crate::storage::get(&root,&server.id)?;
            assert_eq!(Path::new(restored.jar_path.as_ref().unwrap()).canonicalize().unwrap(),Path::new(before.jar_path.as_ref().unwrap()).canonicalize().unwrap());
            assert_eq!(crate::jvm::get(&root,&server.id)?,settings);
            assert_eq!(fs::read(directory.join("config/preserved.txt")).unwrap(),b"preserved settings");
            if migration_project.is_some(){assert_eq!(serde_json::to_value(crate::content_storage::list(&restored)?).unwrap(),serde_json::to_value(&original_content).unwrap());}
            boot(&restored,"restored")?;
            assert_eq!(crate::storage::list_backups(&root,&server.id)?.len(),2);
            assert!(!crate::eula::recorded(&root)?);
            let (_,stage,old,journal)=crate::instance_publication::paths(&root,&restored)?;
            assert!(!stage.exists()&&!old.exists()&&!journal.exists());Ok(())
        })();
        if result.is_ok(){let _=fs::remove_dir_all(&root);}else{eprintln!("Diagnostics retained at {}",root.display());}
        result.unwrap();
    }

    fn content()->(InstalledContent,ContentVersion) {
        let hash="a".repeat(128);
        (InstalledContent{filename:"example.jar".into(),title:"Example".into(),project_id:Some("project".into()),version_id:Some("version".into()),version_number:Some("1.0".into()),sha512:hash.clone(),size:100,modified:false},
         ContentVersion{id:"version".into(),project_id:"project".into(),name:"Example".into(),version_number:"1.0".into(),version_type:"release".into(),date_published:String::new(),game_versions:vec!["1.21.5".into()],loaders:vec!["fabric".into()],environment:None,changelog:None,dependencies:vec![],files:vec![ContentFile{filename:"example.jar".into(),url:String::new(),hashes:std::collections::BTreeMap::from([("sha512".into(),hash)]),size:100,primary:true}]})
    }

    #[test]
    fn upgrades_follow_release_dates_including_new_year_numbering() {
        let old=json!({"id":"1.21.11","type":"release","releaseTime":"2025-12-09T12:00:00+00:00"});
        let new=json!({"id":"26.1","type":"release","releaseTime":"2026-03-24T12:00:00+00:00"});
        assert!(check_target(&old,&new).is_ok());
        assert!(check_target(&new,&old).is_err());
        assert!(check_target(&old,&old).is_err());
        assert!(check_target(&old,&json!({"type":"snapshot","releaseTime":"2026-01-01T12:00:00Z"})).is_err());
        assert!(check_target(&old,&json!({"type":"release","releaseTime":"invalid"})).is_err());
    }

    #[test]
    fn compatibility_requires_matching_artifact_identity_and_hash() {
        let (file,mut version)=content();
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"compatible");
        version.files[0].hashes.insert("sha512".into(),"b".repeat(128));
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"unknown");
        let (_,mut version)=content();version.project_id="other".into();
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"unknown");
        let (_,mut version)=content();version.files[0].size+=1;
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"unknown");
    }

    #[test]
    fn rejects_wrong_minecraft_loader_and_client_only_content() {
        let (file,mut version)=content();
        assert_eq!(check_content(&file,Some(&version),"1.21.4","fabric").status,"incompatible");
        assert_eq!(check_content(&file,Some(&version),"1.21.5","neoforge").status,"incompatible");
        version.environment=Some("client".into());
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"incompatible");
        version.environment=None;version.loaders=vec!["bukkit".into()];
        assert_eq!(check_content(&file,Some(&version),"1.21.5","paper").status,"compatible");
    }

    #[test]
    fn manual_modified_and_unavailable_files_are_never_certified() {
        let (mut file,version)=content();
        assert_eq!(check_content(&file,None,"1.21.5","fabric").status,"unknown");
        file.modified=true;
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"unknown");
        file.modified=false;file.version_id=None;
        assert_eq!(check_content(&file,Some(&version),"1.21.5","fabric").status,"unknown");
    }

    struct Library(std::path::PathBuf);
    impl Drop for Library {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,Server,Review) {
        let root=Library(std::env::temp_dir().join(format!("tch-minecraft-review-{}",uuid::Uuid::new_v4())));
        let mut server=crate::storage::create(&root.0,crate::model::CreateServer{name:"Review".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        let jar=Path::new(&server.directory).join("server.jar");fs::write(&jar,b"launcher").unwrap();
        server.jar_path=Some(jar.to_string_lossy().into());
        let review=Review{current_minecraft:"1.21.4".into(),target_minecraft:"1.21.5".into(),platform:"vanilla".into(),current_loader:None,target_loader:None,java_major:21,java_action:"prepare_managed".into(),content:vec![],blockers:vec![],warnings:vec![],fingerprint:String::new(),changes:vec![],migration:None};
        (root,server,review)
    }

    #[test]
    fn review_fingerprint_detects_settings_content_target_and_launcher_changes() {
        let (root,server,mut review)=fixture();
        let before=fingerprint(&root.0,&server,&[],&review).unwrap();
        review.fingerprint=before.clone();
        assert_eq!(fingerprint(&root.0,&server,&[],&review).unwrap(),before);
        review.target_minecraft="1.21.6".into();
        assert_ne!(fingerprint(&root.0,&server,&[],&review).unwrap(),before);
        review.target_minecraft="1.21.5".into();
        assert_ne!(fingerprint(&root.0,&server,&[content().0],&review).unwrap(),before);
        let mut changed=server.clone();changed.memory_max_mb=2048;
        assert_ne!(fingerprint(&root.0,&changed,&[],&review).unwrap(),before);
        fs::write(server.jar_path.as_ref().unwrap(),b"different launcher").unwrap();
        assert_ne!(fingerprint(&root.0,&server,&[],&review).unwrap(),before);
        fs::write(server.jar_path.as_ref().unwrap(),b"launcher").unwrap();
        fs::write(Path::new(&server.directory).join("server.properties"),b"max-players=3\n").unwrap();
        assert_ne!(fingerprint(&root.0,&server,&[],&review).unwrap(),before);
    }
}
