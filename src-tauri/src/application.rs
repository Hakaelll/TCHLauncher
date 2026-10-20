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
