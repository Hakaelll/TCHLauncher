//! Bounded launcher diagnostics. Minecraft console output and arbitrary error text
//! are deliberately excluded: they can contain commands, player names or secrets.
use serde::Serialize;
use std::fs::{self,OpenOptions};
use std::io::Write;
use std::path::{Path,PathBuf};
use std::sync::Mutex;

const MAX_BYTES:u64=2*1024*1024;
static LOCK:Mutex<()>=Mutex::new(());

#[derive(Clone,Copy,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Event {ApplicationStarted,ApplicationStopping,ServerStarting,ServerRunning,ServerStopping,ServerStopped,ServerCrashed,RouterMaintenanceFailed}
impl Event {
    fn level(self)->&'static str {
        match self {Self::ServerCrashed=>"warn",Self::RouterMaintenanceFailed=>"error",_=>"info"}
    }
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Entry<'a> {timestamp:String,level:&'static str,event:Event,version:&'static str,server_id:Option<&'a str>}

fn ordinary_path(path:&Path)->Result<(),String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink()=>Err("La ruta de logs no puede ser un enlace simbólico".into()),
        Ok(_)=>Ok(()),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(()),Err(error)=>Err(error.to_string()),
    }
}
pub fn directory(root:&Path)->Result<PathBuf,String> {
    let path=root.join("application-logs");ordinary_path(&path)?;
    fs::create_dir_all(&path).map_err(|error|error.to_string())?;
    Ok(path)
}
fn append(root:&Path,event:Event,server_id:Option<&str>,max_bytes:u64)->Result<(),String> {
    if let Some(id)=server_id {
        let uuid=uuid::Uuid::parse_str(id).map_err(|_|"Identificador de diagnóstico no válido")?;
        if uuid.to_string()!=id {return Err("Identificador de diagnóstico no válido".into());}
    }
    let entry=Entry {timestamp:chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis,true),
        level:event.level(),event,version:env!("CARGO_PKG_VERSION"),server_id};
    let mut line=serde_json::to_vec(&entry).map_err(|error|error.to_string())?;line.push(b'\n');
    let _lock=LOCK.lock().map_err(|_|"No se pudo acceder al registro de la aplicación")?;
    let directory=directory(root)?;
    let current=directory.join("launcher.jsonl");let previous=directory.join("launcher.previous.jsonl");
    ordinary_path(&current)?;ordinary_path(&previous)?;
    let length=match fs::metadata(&current) {Ok(metadata)=>metadata.len(),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>0,Err(error)=>return Err(error.to_string())};
    if length>0 && length.saturating_add(line.len() as u64)>max_bytes {
        if previous.exists(){fs::remove_file(&previous).map_err(|error|error.to_string())?;}
        fs::rename(&current,&previous).map_err(|error|error.to_string())?;
    }
    let mut file=OpenOptions::new().create(true).append(true).open(&current).map_err(|error|error.to_string())?;
    file.write_all(&line).map_err(|error|error.to_string())?;
    file.flush().map_err(|error|error.to_string())
}
pub fn record(root:&Path,event:Event,server_id:Option<&str>) {
    if append(root,event,server_id,MAX_BYTES).is_err(){eprintln!("Launcher diagnostic could not be recorded");}
}
pub fn open(root:&Path)->Result<(),String> {
    let path=directory(root)?;
    #[cfg(windows)] let command="explorer";
    #[cfg(target_os="linux")] let command="xdg-open";
    #[cfg(target_os="macos")] let command="open";
    let mut child=std::process::Command::new(command).arg(path).spawn().map_err(|error|format!("No se pudo abrir la carpeta de logs: {error}"))?;
    std::thread::spawn(move||{let _=child.wait();});
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sandbox(PathBuf);
    impl Sandbox {fn new()->Self {Self(std::env::temp_dir().join(format!("tch-app-log-test-{}",uuid::Uuid::new_v4())))}}
    impl Drop for Sandbox {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    #[test]
    fn diagnostic_records_have_levels_and_no_arbitrary_private_payload(){
        let root=Sandbox::new();let id=uuid::Uuid::new_v4().to_string();
        append(&root.0,Event::ApplicationStarted,None,MAX_BYTES).unwrap();
        append(&root.0,Event::ServerCrashed,Some(&id),MAX_BYTES).unwrap();
        append(&root.0,Event::RouterMaintenanceFailed,None,MAX_BYTES).unwrap();
        let entries:Vec<serde_json::Value>=fs::read_to_string(directory(&root.0).unwrap().join("launcher.jsonl")).unwrap().lines().map(|line|serde_json::from_str(line).unwrap()).collect();
        assert_eq!(entries.len(),3);assert_eq!(entries[0]["level"],"info");assert_eq!(entries[1]["level"],"warn");assert_eq!(entries[2]["level"],"error");
        assert_eq!(entries[1]["serverId"],id);assert!(entries[0]["serverId"].is_null());
        assert_eq!(entries[0].as_object().unwrap().len(),5);
        assert!(append(&root.0,Event::ServerStarting,Some("player-name-or-password"),MAX_BYTES).is_err());
    }
    #[test]
    fn log_rotation_retains_two_bounded_parseable_files(){
        let root=Sandbox::new();
        for _ in 0..30 {append(&root.0,Event::ApplicationStarted,None,400).unwrap();}
        let directory=directory(&root.0).unwrap();
        assert_eq!(fs::read_dir(&directory).unwrap().count(),2);
        for file in fs::read_dir(&directory).unwrap() {
            let file=file.unwrap();assert!(file.metadata().unwrap().len()<=400);
            for line in fs::read_to_string(file.path()).unwrap().lines(){assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());}
        }
    }
}
