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
    pub dimensions: Vec<String>,
    pub icon_data: Option<String>,
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

fn valid_folder_name(name: &str) -> Result<(), String> {
    match valid_name(name) {
        Ok(()) => Ok(()),
        Err(error) => {
            if DIMENSION_SUFFIXES.iter().any(|suffix| name.strip_suffix(suffix).is_some_and(|base| valid_name(base).is_ok())) { Ok(()) }
            else { Err(error) }
        }
    }
}

pub(crate) fn plain(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() { return false; }
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return false; }
    }
    true
}

pub(crate) fn valid_level_dat(folder: &Path) -> Result<(), String> {
    let path = folder.join("level.dat");
    let meta = fs::symlink_metadata(&path).map_err(|_| "La carpeta no contiene level.dat".to_string())?;
    if !meta.is_file() || !plain(&meta) || meta.len() > 32_000_000 {
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

pub(crate) fn checked_world(server: &Server, name: &str) -> Result<PathBuf, String> {
    valid_folder_name(name)?;
    let root = server_root(server)?;
    checked_world_at(&root, name)
}

fn checked_world_at(root: &Path, name: &str) -> Result<PathBuf, String> {
    valid_folder_name(name)?;
    let path = root.join(name);
    let meta = fs::symlink_metadata(&path).map_err(|_| "No se encontró ese mundo".to_string())?;
    if !meta.is_dir() || !plain(&meta) {
        return Err("La carpeta del mundo no es segura".into());
    }
    let actual = path.canonicalize().map_err(|e| e.to_string())?;
    if actual.parent() != Some(root) {
        return Err("La carpeta del mundo sale del servidor".into());
    }
    valid_level_dat(&actual)?;
    Ok(actual)
}

fn active_name(server: &Server) -> Result<String, String> {
    let props = properties::read(&server_root(server)?.join("server.properties"))?;
    Ok(props.get("level-name").cloned().unwrap_or_else(|| "world".into()))
}

pub(crate) fn is_world(server:&Server,name:&str)->bool {checked_world(server,name).is_ok()}

pub(crate) const DIMENSION_SUFFIXES: [&str; 2] = ["_nether", "_the_end"];

fn paired_parent(server: &Server, name: &str) -> Option<String> {
    if server.platform != "paper" { return None; }
    DIMENSION_SUFFIXES.iter().find_map(|suffix| {
        let parent = name.strip_suffix(suffix)?;
        checked_world(server, parent).ok().map(|_| parent.to_owned())
    })
}

// Paper stores dimensions beside the primary folder; other engines store them inside it.
pub(crate) fn world_family(server: &Server, name: &str) -> Result<Vec<(String, PathBuf)>, String> {
    let primary = checked_world(server, name)?;
    if let Some(parent) = paired_parent(server, name) {
        return Err(format!("Esa dimensión pertenece al mundo {parent}. Selecciona el mundo principal"));
    }
    valid_name(name)?;
    let mut family = vec![(name.to_owned(), primary)];
    if server.platform == "paper" {
        let root = server_root(server)?;
        for suffix in DIMENSION_SUFFIXES {
            let sibling = format!("{name}{suffix}");
            match fs::symlink_metadata(root.join(&sibling)) {
                Ok(_) => family.push((sibling.clone(), checked_world(server, &sibling).map_err(|error| format!("No se pudo revisar la dimensión {sibling}: {error}"))?)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Ok(family)
}

pub(crate) fn world_info(name: String, family: &[(String, PathBuf)], active: bool) -> WorldInfo {
    let icon = family[0].1.join("icon.png");
    let icon_data = fs::symlink_metadata(&icon).ok()
        .filter(|meta| meta.is_file() && plain(meta))
        .and_then(|_| storage::preview_image(&icon).ok());
    WorldInfo {
        name, size_bytes: family.iter().map(|(_, path)| backup::size(path)).sum(), active, icon_data,
        dimensions: family.iter().skip(1).map(|(name, _)| name.clone()).collect(),
    }
}

pub fn list(server: &Server) -> Result<Vec<WorldInfo>, String> {
    let root = server_root(server)?;
    let active = active_name(server)?;
    let mut worlds = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if valid_name(&name).is_err() || checked_world(server, &name).is_err() || paired_parent(server, &name).is_some() { continue; }
        let family = world_family(server, &name)?;
        let is_active = family.iter().any(|(part, _)| part == &active);
        worlds.push(world_info(name, &family, is_active));
    }
    worlds.sort_by(|a, b| b.active.cmp(&a.active).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(worlds)
}

fn absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
        Ok(_) => Err("Ya existe un archivo o mundo con ese nombre".into()),
    }
}

fn check_destination(root: &Path, name: &str, paper: bool) -> Result<(), String> {
    valid_name(name)?;
    if paper {
        for suffix in DIMENSION_SUFFIXES {
            if let Some(parent) = name.strip_suffix(suffix).filter(|parent| !parent.is_empty()) {
                match fs::symlink_metadata(root.join(parent)) {
                    Ok(_) => {
                        checked_world_at(root, parent)?;
                        return Err(format!("Esa dimensión pertenece al mundo {parent}. Selecciona el mundo principal"));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(error.to_string()),
                }
            }
        }
    }
    absent(&root.join(name))?;
    if paper {
        for suffix in DIMENSION_SUFFIXES { absent(&root.join(format!("{name}{suffix}")))?; }
    }
    Ok(())
}

fn checked_tree(path: &Path) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if !plain(&meta) || !(meta.is_file() || meta.is_dir()) {
            return Err("La copia no admite enlaces ni carpetas redirigidas".into());
        }
    }
    Ok(())
}

fn copy_family(root: &Path, server: &Server, name: &str, new_name: &str, parent: &Path, task: Option<&crate::download::DownloadTask>) -> Result<Vec<(String, PathBuf)>, String> {
    let family = world_family(server, name)?;
    copy_sources(root, server, &family, name, new_name, parent, task)
}

pub(crate) fn copy_sources(root: &Path, server: &Server, family: &[(String, PathBuf)], name: &str, new_name: &str, parent: &Path, task: Option<&crate::download::DownloadTask>) -> Result<Vec<(String, PathBuf)>, String> {
    valid_name(new_name)?;
    for (_, source) in family {
        if parent.starts_with(source) { return Err("Elige una carpeta de destino fuera del mundo".into()); }
        checked_tree(source)?;
    }
    // Reserve missing dimensions too, so copying cannot adopt an unrelated destination folder.
    check_destination(parent, new_name, server.platform == "paper")?;
    let parts: Vec<_> = family.iter().map(|(part, source)| {
        let target_name = format!("{new_name}{}", &part[name.len()..]);
        valid_folder_name(&target_name)?;
        Ok((source.clone(), target_name.clone(), parent.join(target_name), parent.join(format!(".tch-world-{}.part", Uuid::new_v4()))))
    }).collect::<Result<_, String>>()?;
    let prepared: Result<(), String> = (|| {
        for (source, _, _, stage) in &parts {
            if let Some(task) = task {
                task.cancellation.check()?;
                backup::copy_tree_tracked(source, stage, task, match task.kind() { "world_export" => "Exportando archivos del mundo", "world_import" => "Importando archivos del mundo", _ => "Duplicando archivos del mundo" })?;
                task.cancellation.check()?;
                task.report("Comprobando la copia del mundo", "verifying");
            } else { backup::copy_tree(source, stage)?; }
            valid_level_dat(stage)?;
            checked_tree(stage)?;
        }
        // Recheck every destination after copying and before the first visible change.
        check_destination(parent, new_name, server.platform == "paper")?;
        if let Some(task) = task { task.begin_publication("Guardando la copia del mundo")?; }
        check_destination(parent, new_name, server.platform == "paper")?;
        Ok(())
    })();
    let discard_stages = |error: String| {
        let mut failures = Vec::new();
        for (_, _, _, stage) in &parts { if stage.exists() { if let Err(error) = fs::remove_dir_all(stage) { failures.push(format!("{}: {error}", stage.display())); } } }
        if failures.is_empty() { error } else { format!("{error}; no se pudieron retirar algunas copias: {}", failures.join("; ")) }
    };
    if let Err(error) = prepared { return Err(discard_stages(error)); }
    let publication: Vec<_> = parts.iter().map(|(_, _, target, stage)| crate::world_publication::Part { stage: stage.clone(), target: target.clone() }).collect();
    let mut reserved = vec![new_name.to_owned()];
    if server.platform == "paper" { reserved.extend(DIMENSION_SUFFIXES.iter().map(|suffix| format!("{new_name}{suffix}"))); }
    match crate::world_publication::publish(root, &server.id, parent, &publication, &reserved) {
        Ok(()) => (),
        Err(crate::world_publication::Failure::Unprepared(error)) => return Err(discard_stages(error)),
        Err(crate::world_publication::Failure::Pending(error)) => return Err(error),
    }
    Ok(parts.iter().map(|(_, name, target, _)| (name.clone(), target.clone())).collect())
}


pub fn duplicate_tracked(root:&Path,server:&Server,name:&str,new_name:&str,task:&crate::download::DownloadTask)->Result<WorldInfo,String>{
    task.cancellation.check()?;task.report("Comprobando el mundo original","preparing");
    let copied = copy_family(root, server, name, new_name, &server_root(server)?, Some(task))?;
    Ok(world_info(new_name.into(), &copied, false))
}


fn export_inner(root: &Path, server: &Server, name: &str, parent: &Path, task: Option<&crate::download::DownloadTask>) -> Result<PathBuf, String> {
    let parent = parent.canonicalize().map_err(|e| format!("No se pudo abrir el destino: {e}"))?;
    if !parent.is_dir() { return Err("Elige una carpeta de destino fuera del mundo".into()); }
    let copied = copy_family(root, server, name, name, &parent, task)?;
    Ok(copied[0].1.clone())
}

pub fn export_tracked(root:&Path,server:&Server,name:&str,parent:&Path,task:&crate::download::DownloadTask)->Result<PathBuf,String>{
    task.cancellation.check()?;task.report("Comprobando el destino del mundo","preparing");
    export_inner(root,server,name,parent,Some(task))
}


pub fn set_active_tracked(root:&Path,server:&Server,name:&str,task:&crate::download::DownloadTask)->Result<Option<Backup>,String>{
    set_active_inner(root,server,name,Some(task))
}

fn set_active_inner(root:&Path,server:&Server,name:&str,task:Option<&crate::download::DownloadTask>)->Result<Option<Backup>,String>{
    if let Some(task)=task{task.cancellation.check()?;task.report("Comprobando el mundo principal","preparing");}
    world_family(server, name)?;
    if active_name(server)? == name { return Ok(None); }
    let snapshot = if let Some(task)=task{backup::create_tracked(root,&server.id,task)?}else{backup::create_for(root,&server.id,backup::Reason::BeforeWorldChange)?};
    if let Some(task)=task{
        task.report("Copia guardada; preparando el cambio de mundo","preparing");
        task.cancellation.check()?;
    }
    world_family(server,name)?;
    let updates: BTreeMap<String, String> = [("level-name".into(), name.into())].into();
    if let Some(task)=task{task.begin_publication("Guardando el mundo principal")?;}
    properties::write(&server_root(server)?.join("server.properties"), updates)?;
    Ok(Some(snapshot))
}


pub fn delete_tracked(root:&Path,server:&Server,name:&str,task:&crate::download::DownloadTask)->Result<Backup,String>{
    delete_inner(root,server,name,Some(task))
}

fn delete_inner(root:&Path,server:&Server,name:&str,task:Option<&crate::download::DownloadTask>)->Result<Backup,String>{
    if let Some(task)=task{task.cancellation.check()?;task.report("Comprobando el mundo que se eliminará","preparing");}
    let family = world_family(server, name)?;
    let active = active_name(server)?;
    if family.iter().any(|(part, _)| part == &active) {
        return Err("No se puede borrar el mundo principal. Activa otro primero".into());
    }
    let snapshot = if let Some(task)=task{backup::create_tracked(root,&server.id,task)?}else{backup::create_for(root,&server.id,backup::Reason::BeforeWorldDeletion)?};
    if let Some(task)=task{
        task.report("Copia guardada; preparando la eliminación del mundo","preparing");
        task.cancellation.check()?;
    }
    let family = world_family(server, name)?;
    let active = active_name(server)?;
    if family.iter().any(|(part, _)| part == &active) { return Err("No se puede borrar el mundo principal. Activa otro primero".into()); }
    if let Some(task)=task{task.begin_publication("Eliminando el mundo; la copia está guardada")?;}
    for (_, target) in family {
        fs::remove_dir_all(target).map_err(|e| format!("No se pudo borrar el mundo: {e}"))?;
    }
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
