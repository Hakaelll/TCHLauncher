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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consent_is_explicit_persists_and_applies_to_later_servers() {
        let root=std::env::temp_dir().join(format!("tch-consent-{}",uuid::Uuid::new_v4()));
        let create=|name:&str|storage::create(&root,crate::model::CreateServer{name:name.into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        let first=create("First");
        assert!(!recorded(&root).unwrap());assert!(!file_accepted(&first));
        assert!(require_download_consent(&root,&first).is_err());
        assert!(!Path::new(&first.directory).join("eula.txt").exists());
        accept(&root,&first).unwrap();assert!(recorded(&root).unwrap());assert!(file_accepted(&first));
        let second=create("Second");assert!(file_accepted(&second));
        // A restored older eula.txt is prepared from recorded consent at the next start.
        fs::write(Path::new(&second.directory).join("eula.txt"),"eula=false\n").unwrap();
        apply_recorded(&root,&second).unwrap();assert!(file_accepted(&second));
        let other=root.join("other-installation");assert!(!recorded(&other).unwrap());
        let _=fs::remove_dir_all(root);
    }
}
