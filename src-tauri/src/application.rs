use crate::storage;
use serde::Serialize;
use std::{fs,path::{Path,PathBuf},process::Command};

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Location {pub key:String,pub path:String,pub exists:bool}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Info {pub version:String,pub os:String,pub architecture:String,pub locations:Vec<Location>,pub eula_recorded:bool}

pub fn info(root:&Path)->Result<Info,String>{
    let locations=[("library",root.to_path_buf()),("servers",root.join("servers")),("runtimes",root.join("runtimes")),("backups",root.join("backups"))].into_iter().map(|(key,path)|Location{key:key.into(),exists:path.is_dir(),path:path.to_string_lossy().into()}).collect();
    Ok(Info{version:env!("CARGO_PKG_VERSION").into(),os:std::env::consts::OS.into(),architecture:std::env::consts::ARCH.into(),locations,eula_recorded:crate::eula::recorded(root)?})
}

fn folder(root:&Path,key:&str,id:Option<&str>)->Result<PathBuf,String>{
    if key=="server"{
        let server=storage::get(root,id.ok_or("Selecciona un servidor")?)?;
        let path=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
        if !path.is_dir(){return Err("La carpeta del servidor no está disponible".into());}
        return Ok(path);
    }
    let path=match key{"library"=>root.to_path_buf(),"servers"|"runtimes"|"backups"=>root.join(key),_=>return Err("Carpeta de aplicación no válida".into())};
    let base=root.canonicalize().map_err(|e|e.to_string())?;
    if let Ok(meta)=fs::symlink_metadata(&path){if !meta.is_dir()||meta.file_type().is_symlink(){return Err("La carpeta de aplicación no es segura".into());}}
    fs::create_dir_all(&path).map_err(|e|e.to_string())?;
    let actual=path.canonicalize().map_err(|e|e.to_string())?;
    if !actual.starts_with(&base){return Err("La carpeta sale de la biblioteca".into());}
    Ok(actual)
}

pub fn open_folder(root:&Path,key:&str,id:Option<&str>)->Result<(),String>{
    let path=folder(root,key,id)?;
    #[cfg(target_os="windows")] let command="explorer";
    #[cfg(target_os="linux")] let command="xdg-open";
    #[cfg(target_os="macos")] let command="open";
    Command::new(command).arg(path).spawn().map_err(|e|format!("No se pudo abrir la carpeta: {e}"))?;
    Ok(())
}

#[cfg(test)] mod tests{
    use super::*;
    #[test] fn reports_actual_library_and_resolves_only_supported_locations(){
        let root=std::env::temp_dir().join(format!("tch-application-settings-{}",uuid::Uuid::new_v4()));
        storage::connect(&root).unwrap();let details=info(&root).unwrap();
        assert_eq!(details.version,env!("CARGO_PKG_VERSION"));assert!(!details.eula_recorded);assert_eq!(details.locations.len(),4);
        assert_eq!(details.locations[0].path,root.to_string_lossy());assert!(details.locations[0].exists);assert!(!details.locations[2].exists);
        let runtime=folder(&root,"runtimes",None).unwrap();assert!(runtime.is_dir());assert_eq!(runtime.parent().unwrap(),root.canonicalize().unwrap());
        for key in ["../outside","unknown","server"]{assert!(folder(&root,key,None).is_err());}
        let server=storage::create(&root,crate::model::CreateServer{name:"Settings".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        assert_eq!(folder(&root,"server",Some(&server.id)).unwrap(),Path::new(&server.directory).canonicalize().unwrap());
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn refuses_files_in_place_of_application_folders(){
        let root=std::env::temp_dir().join(format!("tch-application-path-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();fs::write(root.join("backups"),b"keep").unwrap();
        assert!(folder(&root,"backups",None).is_err());assert_eq!(fs::read(root.join("backups")).unwrap(),b"keep");fs::remove_dir_all(root).unwrap();
    }
}
