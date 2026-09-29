use crate::backup;
use crate::model::{Backup, Server};
use crate::properties;
use crate::storage;
use flate2::read::GzDecoder;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    pub name: String,
    pub size_bytes: u64,
    pub active: bool,
    pub icon_data: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInspection {
    pub path: String,
    pub suggested_name: String,
    pub size_bytes: u64,
}

fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.chars().count() > 64 || name.starts_with(['.', ' '])
        || name.ends_with(['.', ' '])
        || !name.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.' | '(' | ')')) {
        return Err("Usa entre 1 y 64 caracteres: letras, números, espacios, guiones o puntos".into());
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || (stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit() && stem.as_bytes()[3] != b'0') {
        return Err("Ese nombre está reservado por el sistema".into());
    }
    Ok(())
}

fn valid_level_dat(folder: &Path) -> Result<(), String> {
    let path = folder.join("level.dat");
    let meta = fs::symlink_metadata(&path).map_err(|_| "La carpeta no contiene level.dat".to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 32_000_000 {
        return Err("level.dat no es un archivo de mundo válido".into());
    }
    let decoder = GzDecoder::new(File::open(path).map_err(|e| e.to_string())?);
    let mut content = Vec::new();
    decoder.take(8_000_001).read_to_end(&mut content)
        .map_err(|_| "No se pudo descomprimir level.dat".to_string())?;
    if content.len() > 8_000_000 || content.len() < 4 || content[0] != 10 {
        return Err("level.dat no contiene un mundo de Minecraft Java".into());
    }
    let root_name_len = u16::from_be_bytes([content[1], content[2]]) as usize;
    if root_name_len > 512 || content.len() <= 3 + root_name_len || content[3 + root_name_len] == 0 {
        return Err("level.dat no contiene datos de mundo".into());
    }
    Ok(())
}

fn server_root(server: &Server) -> Result<PathBuf, String> {
    Path::new(&server.directory).canonicalize().map_err(|e| format!("No se encontró la carpeta del servidor: {e}"))
}

fn checked_world(server: &Server, name: &str) -> Result<PathBuf, String> {
    valid_name(name)?;
    let root = server_root(server)?;
    let path = root.join(name);
    let meta = fs::symlink_metadata(&path).map_err(|_| "No se encontró ese mundo".to_string())?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("La carpeta del mundo no es segura".into());
    }
    let actual = path.canonicalize().map_err(|e| e.to_string())?;
    if actual.parent() != Some(root.as_path()) {
        return Err("La carpeta del mundo sale del servidor".into());
    }
    valid_level_dat(&actual)?;
    Ok(actual)
}

fn active_name(server: &Server) -> Result<String, String> {
    let props = properties::read(&server_root(server)?.join("server.properties"))?;
    Ok(props.get("level-name").cloned().unwrap_or_else(|| "world".into()))
}

fn world_info(name: String, path: &Path, active: bool) -> WorldInfo {
    let icon = path.join("icon.png");
    let icon_data = fs::symlink_metadata(&icon).ok()
        .filter(|meta| meta.is_file() && !meta.file_type().is_symlink())
        .and_then(|_| storage::preview_image(&icon).ok());
    WorldInfo { name, size_bytes: backup::size(path), active, icon_data }
}

pub fn list(server: &Server) -> Result<Vec<WorldInfo>, String> {
    let root = server_root(server)?;
    let active = active_name(server)?;
    let mut worlds = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if valid_name(&name).is_err() { continue; }
        let path = entry.path();
        if checked_world(server, &name).is_ok() {
            worlds.push(world_info(name.clone(), &path, name == active));
        }
    }
    worlds.sort_by(|a, b| b.active.cmp(&a.active).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(worlds)
}

pub fn inspect(path: &Path) -> Result<WorldInspection, String> {
    let source = path.canonicalize().map_err(|e| format!("No se pudo abrir la carpeta: {e}"))?;
    if !source.is_dir() { return Err("Selecciona una carpeta de mundo".into()); }
    valid_level_dat(&source)?;
    let suggested_name = source.file_name().and_then(|name| name.to_str()).unwrap_or("Mundo importado").to_string();
    Ok(WorldInspection { path: source.to_string_lossy().into_owned(), suggested_name, size_bytes: backup::size(&source) })
}

fn copy_as(source: &Path, root: &Path, name: &str) -> Result<PathBuf, String> {
    valid_name(name)?;
    let target = root.join(name);
    if target.exists() { return Err("Ya existe un archivo o mundo con ese nombre".into()); }
    let stage = root.join(format!(".tch-world-{}.part", Uuid::new_v4()));
    let result = (|| {
        backup::copy_tree(source, &stage)?;
        valid_level_dat(&stage)?;
        fs::rename(&stage, &target).map_err(|e| e.to_string())?;
        Ok(target)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&stage); }
    result
}

pub fn import(server: &Server, source: &Path, name: &str) -> Result<WorldInfo, String> {
    let inspection = inspect(source)?;
    let source = Path::new(&inspection.path);
    let root = server_root(server)?;
    if root.starts_with(source) { return Err("Elige una carpeta de mundo, no la carpeta completa del servidor".into()); }
    let target = copy_as(source, &root, name)?;
    Ok(world_info(name.into(), &target, name == active_name(server)?))
}

pub fn duplicate(server: &Server, name: &str, new_name: &str) -> Result<WorldInfo, String> {
    let source = checked_world(server, name)?;
    let target = copy_as(&source, &server_root(server)?, new_name)?;
    Ok(world_info(new_name.into(), &target, false))
}

pub fn export(server: &Server, name: &str, parent: &Path) -> Result<PathBuf, String> {
    let source = checked_world(server, name)?;
    let parent = parent.canonicalize().map_err(|e| format!("No se pudo abrir el destino: {e}"))?;
    if !parent.is_dir() || parent.starts_with(&source) {
        return Err("Elige una carpeta de destino fuera del mundo".into());
    }
    copy_as(&source, &parent, name)
}

pub fn set_active(root: &Path, server: &Server, name: &str) -> Result<Option<Backup>, String> {
    checked_world(server, name)?;
    if active_name(server)? == name { return Ok(None); }
    let snapshot = backup::create(root, &server.id)?;
    let updates: BTreeMap<String, String> = [("level-name".into(), name.into())].into();
    properties::write(&server_root(server)?.join("server.properties"), updates)?;
    Ok(Some(snapshot))
}

pub fn delete(root: &Path, server: &Server, name: &str) -> Result<Backup, String> {
    let target = checked_world(server, name)?;
    if active_name(server)? == name { return Err("No se puede borrar el mundo principal. Activa otro primero".into()); }
    let snapshot = backup::create(root, &server.id)?;
    fs::remove_dir_all(target).map_err(|e| format!("No se pudo borrar el mundo: {e}"))?;
    Ok(snapshot)
}

pub fn open_folder(server: &Server, name: &str) -> Result<(), String> {
    let path = checked_world(server, name)?;
    #[cfg(target_os = "windows")]
    let command = "explorer";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "macos")]
    let command = "open";
    Command::new(command).arg(path).spawn().map_err(|e| format!("No se pudo abrir la carpeta: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CreateServer;
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;

    fn fixture(root: &Path) -> Server {
        storage::create(root, CreateServer { name: "Mundos".into(), version: "1.21.4".into(), platform: "vanilla".into(), memory_min_mb: 1024, memory_max_mb: 2048, max_players: 20, port: 25565, storage_target_gb: 20, image_path: None }).unwrap()
    }

    fn make_world(path: &Path) {
        fs::create_dir_all(path.join("region")).unwrap();
        let file = File::create(path.join("level.dat")).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&[10, 0, 0, 10, 0, 4, b'D', b'a', b't', b'a', 0]).unwrap();
        encoder.finish().unwrap();
        fs::write(path.join("region/r.0.0.mca"), b"world data").unwrap();
    }

    #[test]
    fn imports_copies_switches_exports_and_deletes_worlds_safely() {
        let root = std::env::temp_dir().join(format!("tch-world-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let server = fixture(&root);
        let source = root.join("selected-world");
        make_world(&source);
        assert_eq!(inspect(&source).unwrap().suggested_name, "selected-world");
        import(&server, &source, "Aventura").unwrap();
        duplicate(&server, "Aventura", "Aventura copia").unwrap();
        assert_eq!(list(&server).unwrap().len(), 2);
        assert!(import(&server, &source, "Aventura").is_err());
        let snapshot = set_active(&root, &server, "Aventura").unwrap().unwrap();
        assert!(Path::new(&snapshot.path).is_dir());
        assert_eq!(list(&server).unwrap()[0].name, "Aventura");
        assert!(delete(&root, &server, "Aventura").is_err());
        let exported = export(&server, "Aventura", &root).unwrap();
        assert!(exported.join("level.dat").is_file());
        assert!(export(&server, "Aventura", &root).is_err());
        delete(&root, &server, "Aventura copia").unwrap();
        assert_eq!(list(&server).unwrap().len(), 1);
        assert!(source.join("level.dat").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_paths_and_invalid_world_data() {
        let root = std::env::temp_dir().join(format!("tch-world-invalid-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let server = fixture(&root);
        let invalid = root.join("invalid");
        fs::create_dir_all(&invalid).unwrap();
        fs::write(invalid.join("level.dat"), b"not gzip").unwrap();
        assert!(inspect(&invalid).is_err());
        make_world(&root.join("valid"));
        assert!(import(&server, &root.join("valid"), "../escape").is_err());
        assert!(import(&server, &root.join("valid"), "CON").is_err());
        assert!(list(&server).unwrap().is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
