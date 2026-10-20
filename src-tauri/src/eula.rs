use crate::{model::Server, storage};
use rusqlite::OptionalExtension;
use std::{fs, io::Read, path::Path};

const KEY: &str = "minecraft_eula_acceptance";
const URL: &str = "https://www.minecraft.net/en-us/eula";

pub fn recorded(root: &Path) -> Result<bool,String> {
    let value=storage::connect(root)?.query_row("SELECT value FROM application_settings WHERE key=?1",[KEY],|row|row.get::<_,String>(0)).optional().map_err(|e|e.to_string())?;
    Ok(value.and_then(|text|serde_json::from_str::<serde_json::Value>(&text).ok())
        .is_some_and(|value|value["accepted"]==true && value["document"].as_str()==Some(URL) && value["acceptedAt"].as_str().is_some()))
}

pub fn file_accepted(server: &Server) -> bool {
    let path=Path::new(&server.directory).join("eula.txt");
    if !fs::metadata(&path).is_ok_and(|meta|meta.is_file() && meta.len()<=65536) {return false;}
    let Ok(file)=fs::File::open(path) else{return false;};
    let mut text=String::new();
    // Bound the actual read too: the file may grow after the metadata check.
    file.take(65537).read_to_string(&mut text).is_ok()&&text.len()<=65536&&text.lines().any(|line|line.trim()=="eula=true")
}

fn write(server: &Server) -> Result<(),String> {
    let directory=Path::new(&server.directory);
    let target=directory.join("eula.txt");
    if let Ok(meta)=fs::symlink_metadata(&target) {
        if !meta.file_type().is_file() {return Err("eula.txt no puede ser una carpeta ni un enlace simbólico".into());}
    }
    let temp=directory.join(format!(".eula-{}.part",uuid::Uuid::new_v4()));
    let result=(|| {
        fs::write(&temp,"# Acceptance explicitly recorded by the user in TCHLauncher\neula=true\n").map_err(|e|e.to_string())?;
        fs::rename(&temp,&target).map_err(|e|format!("No se pudo preparar eula.txt: {e}"))
    })();
    if result.is_err() {let _=fs::remove_file(temp);}
    result
}

// Called only by the explicit acceptance command; server creation never records consent.
pub fn accept(root: &Path, server: &Server) -> Result<(),String> {
    let value=serde_json::json!({"accepted":true,"document":URL,"acceptedAt":chrono::Utc::now().to_rfc3339()}).to_string();
    storage::connect(root)?.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[KEY,&value]).map_err(|e|e.to_string())?;
    write(server)
}

pub fn apply_recorded(root: &Path, server: &Server) -> Result<bool,String> {
    if !recorded(root)? {return Ok(false);}
    if !file_accepted(server) {write(server)?;}
    Ok(true)
}

pub fn require_download_consent(root: &Path, server: &Server) -> Result<(),String> {
    if !apply_recorded(root,server)? {return Err("Acepta la EULA de Minecraft antes de la primera descarga. Se recordará en este equipo.".into());}
    Ok(())
}
