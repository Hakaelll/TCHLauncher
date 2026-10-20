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
            let data=task.json(crate::neoforge::catalog_url(target),"Revisando NeoForge para el nuevo Minecraft")?;
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
