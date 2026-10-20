use crate::storage;
use rusqlite::{params,OptionalExtension};
use serde::Serialize;
use std::path::{Path,PathBuf};

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct OsStatus {pub supported:bool,pub enabled:bool,pub current_path:bool,pub note:Option<String>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct ServerOption {pub id:String,pub name:String,pub enabled:bool,pub last_result:Option<String>}
fn connect(root:&Path)->Result<rusqlite::Connection,String> {
    let db=storage::connect(root)?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS startup_servers(server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,enabled INTEGER NOT NULL DEFAULT 0,last_result TEXT);").map_err(|e|e.to_string())?;Ok(db)
}
pub fn servers(root:&Path)->Result<Vec<ServerOption>,String> {
    let db=connect(root)?;let mut statement=db.prepare("SELECT servers.id,servers.name,COALESCE(startup_servers.enabled,0),startup_servers.last_result FROM servers LEFT JOIN startup_servers ON servers.id=startup_servers.server_id ORDER BY servers.pinned DESC,servers.name,servers.id").map_err(|e|e.to_string())?;
    let rows=statement.query_map([],|row|Ok(ServerOption{id:row.get(0)?,name:row.get(1)?,enabled:row.get(2)?,last_result:row.get(3)?})).map_err(|e|e.to_string())?;rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}
pub fn set_server(root:&Path,id:&str,enabled:bool)->Result<(),String> {
    storage::get(root,id)?;connect(root)?.execute("INSERT INTO startup_servers(server_id,enabled) VALUES(?1,?2) ON CONFLICT(server_id) DO UPDATE SET enabled=excluded.enabled",params![id,enabled]).map_err(|e|e.to_string())?;Ok(())
}
pub fn enabled(root:&Path,id:&str)->Result<bool,String> {
    connect(root)?.query_row("SELECT enabled FROM startup_servers WHERE server_id=?1",[id],|row|row.get(0)).optional().map_err(|e|e.to_string()).map(|value|value.unwrap_or(false))
}
pub fn finish(root:&Path,id:&str,result:&str)->Result<(),String> {
    connect(root)?.execute("UPDATE startup_servers SET last_result=?2 WHERE server_id=?1",params![id,result]).map_err(|e|e.to_string())?;Ok(())
}

fn executable()->Result<PathBuf,String> {
    #[cfg(target_os="linux")]
    if let Some(path)=std::env::var_os("APPIMAGE") {let path=PathBuf::from(path);if path.is_absolute()&&path.is_file(){return Ok(path);}}
    std::env::current_exe().map_err(|e|e.to_string())
}
#[cfg(any(windows,test))]
fn windows_command(path:&Path)->Result<String,String> {
    let text=path.to_str().ok_or("La ruta del launcher no se puede registrar")?;
    if text.contains(['"','\r','\n','\0'])||text.is_empty(){return Err("La ruta del launcher no es válida".into());}
    let command=format!("\"{text}\" --autostart");
    if command.encode_utf16().count()>260 {return Err("La ruta del launcher supera el límite de inicio de Windows. Instálalo en una carpeta más corta.".into());}Ok(command)
}
#[cfg(windows)]
mod os {
    use super::*;
    use winreg::{RegKey,enums::{HKEY_CURRENT_USER,KEY_READ,KEY_SET_VALUE}};
    const KEY:&str="Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const VALUE:&str="TCHLauncher";
    fn read()->Result<Option<String>,String> {
        let key=match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY,KEY_READ){Ok(key)=>key,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string())};
        match key.get_value(VALUE){Ok(value)=>Ok(Some(value)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(error.to_string())}
    }
    pub fn status()->Result<OsStatus,String> {
        let command=read()?;let expected=windows_command(&executable()?)?;
        Ok(OsStatus{supported:!cfg!(debug_assertions),enabled:command.is_some(),current_path:command.as_deref()==Some(&expected),note:if cfg!(debug_assertions){Some("El inicio con Windows se configura desde la versión instalada, para no registrar un ejecutable de desarrollo.".into())}else if command.is_some()&&command.as_deref()!=Some(&expected){Some("El inicio apunta a otra ubicación. Actualízalo para usar esta instalación.".into())}else{None}})
    }
    pub fn set(enabled:bool)->Result<OsStatus,String> {
        if cfg!(debug_assertions)&&enabled{return Err("Configura el inicio con Windows desde la versión instalada.".into());}
        if enabled {
            let (key,_)=RegKey::predef(HKEY_CURRENT_USER).create_subkey(KEY).map_err(|e|e.to_string())?;
            key.set_value(VALUE,&windows_command(&executable()?)?).map_err(|e|e.to_string())?;
        }else{
            match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY,KEY_SET_VALUE){Ok(key)=>match key.delete_value(VALUE){Ok(())=>(),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>(),Err(error)=>return Err(error.to_string())},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>(),Err(error)=>return Err(error.to_string())}
        }
        status()
    }
}

#[cfg(target_os="linux")]
const MARKER:&str="# Managed by TCHLauncher\n";
#[cfg(target_os="linux")]
fn desktop(path:&Path)->Result<String,String> {
    let text=path.to_str().ok_or("La ruta del launcher no es UTF-8")?;
    // Exec field codes inside quoted arguments are undefined by the standard.
    if !text.starts_with('/')||text.contains(['\r','\n','\0','=','%']){return Err("La ruta del launcher no es compatible con el inicio de escritorio.".into());}
    let escaped=text.replace('\\',"\\\\\\\\").replace('"',"\\\\\"").replace('`',"\\\\`").replace('$',"\\\\$");
    Ok(format!("{MARKER}[Desktop Entry]\nType=Application\nName=TCHLauncher\nExec=\"{escaped}\" --autostart\nTerminal=false\n"))
}
#[cfg(target_os="linux")]
fn read_desktop(path:&Path)->Result<Option<String>,String> {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink()||!meta.is_file()||meta.len()>32768=>return Err("La entrada de inicio no es un archivo normal de TCHLauncher.".into()),
        Ok(_)=>(),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string()),
    }
    let value=std::fs::read_to_string(path).map_err(|e|e.to_string())?;
    if !value.starts_with(MARKER){return Err("Ya existe una entrada de inicio ajena al launcher. No se modificará.".into());}Ok(Some(value))
}
#[cfg(target_os="linux")]
fn write_desktop(path:&Path,value:Option<&str>)->Result<(),String> {
    read_desktop(path)?;
    if let Some(value)=value {
        let parent=path.parent().ok_or("Carpeta de inicio no válida")?;std::fs::create_dir_all(parent).map_err(|e|e.to_string())?;
        let pending=parent.join(format!(".tch-startup-{}.tmp",uuid::Uuid::new_v4()));
        let result=(||->Result<(),String>{use std::io::Write;let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&pending).map_err(|e|e.to_string())?;file.write_all(value.as_bytes()).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);std::fs::rename(&pending,path).map_err(|e|e.to_string())})();
        if result.is_err(){let _=std::fs::remove_file(pending);}result
    }else{match std::fs::remove_file(path){Ok(())=>Ok(()),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(()),Err(error)=>Err(error.to_string())}}
}
#[cfg(target_os="linux")]
mod os {
    use super::*;
    fn path()->Result<PathBuf,String> {
        let config=std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).filter(|path|path.is_absolute()).or_else(||std::env::var_os("HOME").map(|home|PathBuf::from(home).join(".config"))).ok_or("No se encontró la carpeta de configuración del usuario")?;
        if !config.is_absolute(){return Err("La carpeta de configuración no es absoluta".into());}Ok(config.join("autostart/com.tchlauncher.desktop.desktop"))
    }
    pub fn status()->Result<OsStatus,String> {
        let entry=read_desktop(&path()?)?;let expected=desktop(&executable()?)?;
        Ok(OsStatus{supported:!cfg!(debug_assertions),enabled:entry.is_some(),current_path:entry.as_deref()==Some(&expected),note:if cfg!(debug_assertions){Some("El inicio con el sistema se configura desde la versión instalada.".into())}else if entry.is_some()&&entry.as_deref()!=Some(&expected){Some("Actualiza el inicio para usar esta ubicación del launcher.".into())}else{None}})
    }
    pub fn set(enabled:bool)->Result<OsStatus,String> {
        if cfg!(debug_assertions)&&enabled{return Err("Configura el inicio con el sistema desde la versión instalada.".into());}
        let value=if enabled{Some(desktop(&executable()?)?)}else{None};write_desktop(&path()?,value.as_deref())?;status()
    }
}
#[cfg(not(any(windows,target_os="linux")))]
mod os {
    use super::*;
    pub fn status()->Result<OsStatus,String>{Ok(OsStatus{supported:false,enabled:false,current_path:false,note:Some("El inicio con el sistema está disponible en Windows y Linux.".into())})}
    pub fn set(_:bool)->Result<OsStatus,String>{Err("El inicio con el sistema no está disponible en esta plataforma.".into())}
}
pub fn os_status()->Result<OsStatus,String>{os::status()}
pub fn set_os(enabled:bool)->Result<OsStatus,String>{os::set(enabled)}
