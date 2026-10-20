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

fn valid_folder_name(name: &str) -> Result<(), String> {
    match valid_name(name) {
        Ok(()) => Ok(()),
        Err(error) => {
            if DIMENSION_SUFFIXES.iter().any(|suffix| name.strip_suffix(suffix).is_some_and(|base| valid_name(base).is_ok())) { Ok(()) }
            else { Err(error) }
        }
    }
}

fn plain(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() { return false; }
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return false; }
    }
    true
}

fn valid_level_dat(folder: &Path) -> Result<(), String> {
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

const DIMENSION_SUFFIXES: [&str; 2] = ["_nether", "_the_end"];

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

fn world_info(name: String, family: &[(String, PathBuf)], active: bool) -> WorldInfo {
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

pub fn inspect(path: &Path) -> Result<WorldInspection, String> {
    let source = path.canonicalize().map_err(|e| format!("No se pudo abrir la carpeta: {e}"))?;
    if !source.is_dir() { return Err("Selecciona una carpeta de mundo".into()); }
    valid_level_dat(&source)?;
    let suggested_name = source.file_name().and_then(|name| name.to_str()).unwrap_or("Mundo importado").to_string();
    Ok(WorldInspection { path: source.to_string_lossy().into_owned(), suggested_name, size_bytes: backup::size(&source) })
}

fn copy_as_inner(source: &Path, root: &Path, name: &str, paper: bool, task: Option<&crate::download::DownloadTask>) -> Result<PathBuf, String> {
    check_destination(root, name, paper)?;
    let target = root.join(name);
    let stage = root.join(format!(".tch-world-{}.part", Uuid::new_v4()));
    let result = (|| {
        let action=match task.map(|task|task.kind()){Some("world_duplicate")=>"Duplicando archivos del mundo",Some("world_export")=>"Exportando archivos del mundo",_=>"Importando archivos del mundo"};
        if let Some(task)=task { backup::copy_tree_tracked(source,&stage,task,action)?; }
        else { backup::copy_tree(source, &stage)?; }
        if let Some(task)=task { task.cancellation.check()?; task.report("Comprobando la copia del mundo","verifying"); }
        valid_level_dat(&stage)?;
        check_destination(root, name, paper)?;
        if let Some(task)=task { task.begin_publication("Guardando la copia del mundo")?; }
        check_destination(root, name, paper)?;
        fs::rename(&stage, &target).map_err(|e| e.to_string())?;
        Ok(target)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&stage); }
    result
}

#[cfg(test)]
pub fn import(server: &Server, source: &Path, name: &str) -> Result<WorldInfo, String> {
    import_inner(server, source, name, None)
}

pub fn import_tracked(server: &Server, source: &Path, name: &str, task: &crate::download::DownloadTask) -> Result<WorldInfo, String> {
    import_inner(server, source, name, Some(task))
}

fn import_inner(server: &Server, source: &Path, name: &str, task: Option<&crate::download::DownloadTask>) -> Result<WorldInfo, String> {
    if let Some(task)=task { task.cancellation.check()?; task.report("Comprobando la carpeta del mundo","preparing"); }
    let source_path = source.canonicalize().map_err(|e|format!("No se pudo abrir la carpeta: {e}"))?;
    let source = source_path.as_path();
    if !source.is_dir() { return Err("Selecciona una carpeta de mundo".into()); }
    valid_level_dat(source)?;
    let root = server_root(server)?;
    if root.starts_with(source) { return Err("Elige una carpeta de mundo, no la carpeta completa del servidor".into()); }
    let active = name == active_name(server)?;
    let target = copy_as_inner(source, &root, name, server.platform == "paper", task)?;
    Ok(world_info(name.into(), &[(name.into(), target)], active))
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

fn copy_family(server: &Server, name: &str, new_name: &str, parent: &Path, task: Option<&crate::download::DownloadTask>) -> Result<Vec<(String, PathBuf)>, String> {
    valid_name(new_name)?;
    let family = world_family(server, name)?;
    for (_, source) in &family {
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
    let mut published = Vec::new();
    let result: Result<Vec<(String, PathBuf)>, String> = (|| {
        for (source, _, _, stage) in &parts {
            if let Some(task) = task {
                task.cancellation.check()?;
                backup::copy_tree_tracked(source, stage, task, if task.kind() == "world_export" { "Exportando archivos del mundo" } else { "Duplicando archivos del mundo" })?;
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
        for (_, _, target, stage) in &parts {
            absent(target)?;
            fs::rename(stage, target).map_err(|e| e.to_string())?;
            published.push(target.clone());
        }
        Ok(parts.iter().map(|(_, name, target, _)| (name.clone(), target.clone())).collect())
    })();
    if result.is_err() {
        let mut failures = Vec::new();
        for path in &published { if let Err(error) = fs::remove_dir_all(path) { failures.push(format!("{}: {error}", path.display())); } }
        for (_, _, _, stage) in &parts { if stage.exists() { if let Err(error) = fs::remove_dir_all(stage) { failures.push(format!("{}: {error}", stage.display())); } } }
        if !failures.is_empty() { return Err(format!("{}; no se pudieron retirar algunas copias: {}", result.unwrap_err(), failures.join("; "))); }
    }
    result
}

#[cfg(test)]
pub fn duplicate(server: &Server, name: &str, new_name: &str) -> Result<WorldInfo, String> {
    let copied = copy_family(server, name, new_name, &server_root(server)?, None)?;
    Ok(world_info(new_name.into(), &copied, false))
}

pub fn duplicate_tracked(server:&Server,name:&str,new_name:&str,task:&crate::download::DownloadTask)->Result<WorldInfo,String>{
    task.cancellation.check()?;task.report("Comprobando el mundo original","preparing");
    let copied = copy_family(server, name, new_name, &server_root(server)?, Some(task))?;
    Ok(world_info(new_name.into(), &copied, false))
}

#[cfg(test)]
pub fn export(server: &Server, name: &str, parent: &Path) -> Result<PathBuf, String> {
    export_inner(server, name, parent, None)
}

fn export_inner(server: &Server, name: &str, parent: &Path, task: Option<&crate::download::DownloadTask>) -> Result<PathBuf, String> {
    let parent = parent.canonicalize().map_err(|e| format!("No se pudo abrir el destino: {e}"))?;
    if !parent.is_dir() { return Err("Elige una carpeta de destino fuera del mundo".into()); }
    let copied = copy_family(server, name, name, &parent, task)?;
    Ok(copied[0].1.clone())
}

pub fn export_tracked(server:&Server,name:&str,parent:&Path,task:&crate::download::DownloadTask)->Result<PathBuf,String>{
    task.cancellation.check()?;task.report("Comprobando el destino del mundo","preparing");
    export_inner(server,name,parent,Some(task))
}

#[cfg(test)]
pub fn set_active(root: &Path, server: &Server, name: &str) -> Result<Option<Backup>, String> {
    set_active_inner(root,server,name,None)
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

#[cfg(test)]
pub fn delete(root: &Path, server: &Server, name: &str) -> Result<Backup, String> {
    delete_inner(root,server,name,None)
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
    fn tracked_import_measures_files_and_publishes_only_a_complete_world() {
        use std::sync::{Arc,Mutex};
        let root=std::env::temp_dir().join(format!("tch-world-progress-{}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();let server=fixture(&root);let source=root.join("original");make_world(&source);
        fs::write(source.join("region/r.0.0.mca"),vec![7u8;300_000]).unwrap();
        let expected=backup::size(&source);
        for (name,cancel,corrupt) in [("Complete",false,false),("Cancelled",true,false),("Invalid",false,true)] {
            let jobs:crate::file_tasks::Registry=Arc::new(Mutex::new(std::collections::HashMap::new()));
            let observed=jobs.clone();let events=Arc::new(Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();let directory=PathBuf::from(&server.directory);
            let result=crate::file_tasks::import_world(&server,&source,name,&jobs,move|event|{
                let task=observed.lock().unwrap().get(&event.server_id).unwrap().clone();
                assert_eq!(task.id,event.task_id);assert!(task.is_file_operation());
                if cancel&&event.stage=="copying"&&event.received>0 {assert!(task.request_cancel());}
                if corrupt&&event.stage=="verifying" {
                    let stage=fs::read_dir(&directory).unwrap().map(|entry|entry.unwrap().path()).find(|path|path.file_name().unwrap().to_string_lossy().starts_with(".tch-world-")).unwrap();
                    fs::write(stage.join("level.dat"),b"invalid").unwrap();
                }
                if event.stage=="publishing" {assert!(!task.request_cancel());}
                capture.lock().unwrap().push(event);
            });
            assert!(jobs.lock().unwrap().is_empty());
            let events=events.lock().unwrap();let terminal=&events.last().unwrap().stage;
            if cancel {assert_eq!(result.unwrap_err(),crate::download::CANCELLED);assert_eq!(terminal,"cancelled");}
            else if corrupt {assert!(result.is_err());assert_eq!(terminal,"failed");}
            else {assert_eq!(result.unwrap().name,name);assert_eq!(terminal,"completed");let measured=events.iter().filter(|event|event.stage=="copying").last().unwrap();assert_eq!(measured.received,expected);assert_eq!(measured.total,Some(expected));assert_eq!(fs::read(Path::new(&server.directory).join(name).join("region/r.0.0.mca")).unwrap(),vec![7u8;300_000]);}
            assert_eq!(Path::new(&server.directory).join(name).exists(),!cancel&&!corrupt);
            assert!(fs::read_dir(&server.directory).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().starts_with(".tch-world-")));
            assert_eq!(fs::read(source.join("region/r.0.0.mca")).unwrap(),vec![7u8;300_000]);valid_level_dat(&source).unwrap();
        }
        assert_eq!(list(&server).unwrap().iter().map(|world|world.name.as_str()).collect::<Vec<_>>(),vec!["Complete"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tracked_world_changes_cancel_before_mutation_and_retain_completed_safety_copies(){
        use std::sync::{Arc,Mutex};
        for delete in [false,true]{
            for mode in ["during_backup","after_backup","complete"]{
                let root=std::env::temp_dir().join(format!("tch-world-change-progress-{}",Uuid::new_v4()));
                fs::create_dir_all(&root).unwrap();let server=fixture(&root);let directory=PathBuf::from(&server.directory);
                make_world(&directory.join("Main"));make_world(&directory.join("Other"));
                fs::write(directory.join("Other/region/r.0.0.mca"),vec![6u8;300_000]).unwrap();
                properties::write(&directory.join("server.properties"),[("level-name".into(),"Main".into())].into()).unwrap();
                let expected=backup::size(&directory);
                let jobs:crate::file_tasks::Registry=Arc::new(Mutex::new(std::collections::HashMap::new()));
                let observed=jobs.clone();let events=Arc::new(Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();
                let emit=move|event:crate::model::Progress|{
                    let task=observed.lock().unwrap()[&event.server_id].clone();
                    assert_eq!(task.id,event.task_id);assert!(task.is_file_operation());
                    assert_eq!(event.kind,if delete{"world_delete"}else{"world_activate"});
                    if mode=="during_backup"&&event.stage=="copying"&&event.received>0{assert!(task.request_cancel());}
                    if mode=="after_backup"&&event.stage=="preparing"&&event.label.starts_with("Copia guardada;"){assert!(task.request_cancel());}
                    if event.stage=="publishing"{assert!(!task.request_cancel());}
                    capture.lock().unwrap().push(event);
                };
                let result=if delete{crate::file_tasks::delete_world(&root,&server,"Other",&jobs,emit).map(|_|())}else{crate::file_tasks::set_active_world(&root,&server,"Other",&jobs,emit).map(|_|())};
                assert!(jobs.lock().unwrap().is_empty());
                let events=events.lock().unwrap();let copies=storage::list_backups(&root,&server.id).unwrap();
                if mode=="complete"{
                    result.unwrap();assert_eq!(events.last().unwrap().stage,"completed");
                    assert_eq!(directory.join("Other").exists(),!delete);
                    assert_eq!(active_name(&server).unwrap(),if delete{"Main"}else{"Other"});
                }else{
                    assert_eq!(result.unwrap_err(),crate::download::CANCELLED);assert_eq!(events.last().unwrap().stage,"cancelled");
                    assert_eq!(active_name(&server).unwrap(),"Main");
                    assert_eq!(fs::read(directory.join("Other/region/r.0.0.mca")).unwrap(),vec![6u8;300_000]);
                }
                assert_eq!(copies.len(),usize::from(mode!="during_backup"));
                if mode!="during_backup"{
                    let measured=events.iter().filter(|event|event.stage=="copying").last().unwrap();
                    assert_eq!(measured.received,expected);assert_eq!(measured.total,Some(expected));
                    let copy=Path::new(&copies[0].path);assert_eq!(fs::read(copy.join("Other/region/r.0.0.mca")).unwrap(),vec![6u8;300_000]);
                    assert_eq!(properties::read(&copy.join("server.properties")).unwrap()["level-name"],"Main");
                }
                // The active world stays protected and selecting it again needs no backup.
                let active=active_name(&server).unwrap();let before=copies.len();
                assert!(crate::file_tasks::delete_world(&root,&server,&active,&jobs,|_|{}).is_err());
                assert!(crate::file_tasks::set_active_world(&root,&server,&active,&jobs,|_|{}).unwrap().is_none());
                assert_eq!(storage::list_backups(&root,&server.id).unwrap().len(),before);assert!(jobs.lock().unwrap().is_empty());
                fs::remove_dir_all(root).unwrap();
            }
        }
    }

    #[test]
    fn tracked_duplicates_and_exports_preserve_sources_and_cancelled_destinations() {
        use std::sync::{Arc,Mutex};
        let root=std::env::temp_dir().join(format!("tch-world-copy-progress-{}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();let server=fixture(&root);
        let source=PathBuf::from(&server.directory).join("Original");make_world(&source);
        fs::write(source.join("region/r.0.0.mca"),vec![9u8;300_000]).unwrap();
        let expected=backup::size(&source);
        for export in [false,true] {
            for mode in ["cancel","invalid","complete","exists"] {
                let parent=if export{root.join(format!("export-{mode}"))}else{PathBuf::from(&server.directory)};
                fs::create_dir_all(&parent).unwrap();
                let name=if export{"Original"}else{mode};
                let target=parent.join(name);
                if mode=="exists"{fs::create_dir_all(&target).unwrap();fs::write(target.join("keep"),b"unchanged").unwrap();}
                let jobs:crate::file_tasks::Registry=Arc::new(Mutex::new(std::collections::HashMap::new()));
                let observed=jobs.clone();let events=Arc::new(Mutex::new(Vec::<crate::model::Progress>::new()));let capture=events.clone();let directory=parent.clone();
                let emit=move|event:crate::model::Progress|{
                    let task=observed.lock().unwrap()[&event.server_id].clone();
                    assert_eq!(task.id,event.task_id);assert!(task.is_file_operation());
                    assert_eq!(event.kind,if export{"world_export"}else{"world_duplicate"});
                    if mode=="cancel"&&event.stage=="copying"&&event.received>0{assert!(task.request_cancel());}
                    if mode=="invalid"&&event.stage=="verifying"{
                        let stage=fs::read_dir(&directory).unwrap().map(|entry|entry.unwrap().path()).find(|path|path.file_name().unwrap().to_string_lossy().starts_with(".tch-world-")).unwrap();
                        fs::write(stage.join("level.dat"),b"invalid").unwrap();
                    }
                    if event.stage=="publishing"{assert!(!task.request_cancel());}
                    capture.lock().unwrap().push(event);
                };
                let result=if export{crate::file_tasks::export_world(&server,"Original",&parent,&jobs,emit).map(|_|())}else{crate::file_tasks::duplicate_world(&server,"Original",name,&jobs,emit).map(|_|())};
                assert!(jobs.lock().unwrap().is_empty());
                let events=events.lock().unwrap();let terminal=events.last().unwrap();
                if mode=="complete"{
                    result.unwrap();assert_eq!(terminal.stage,"completed");
                    let measured=events.iter().filter(|event|event.stage=="copying").last().unwrap();assert_eq!(measured.received,expected);assert_eq!(measured.total,Some(expected));
                    valid_level_dat(&target).unwrap();assert_eq!(fs::read(target.join("region/r.0.0.mca")).unwrap(),vec![9u8;300_000]);
                }else if mode=="cancel"{assert_eq!(result.unwrap_err(),crate::download::CANCELLED);assert_eq!(terminal.stage,"cancelled");assert!(!target.exists());}
                else{assert!(result.is_err());assert_eq!(terminal.stage,"failed");if mode=="exists"{assert_eq!(fs::read(target.join("keep")).unwrap(),b"unchanged");}else{assert!(!target.exists());}}
                assert!(fs::read_dir(&parent).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().starts_with(".tch-world-")));
                assert_eq!(fs::read(source.join("region/r.0.0.mca")).unwrap(),vec![9u8;300_000]);valid_level_dat(&source).unwrap();
            }
        }
        // Invalid export targets must never create a nested copy inside the source.
        let task=crate::download::DownloadTask::new(&server.id,"world_export",|_|{}).unwrap();
        assert!(export_tracked(&server,"Original",&source.join("region"),&task).is_err());
        assert!(!source.join("region/Original").exists());
        fs::remove_dir_all(root).unwrap();
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

    fn paper_fixture(root: &Path) -> Server {
        let mut server = fixture(root);
        server.platform = "paper".into();
        let directory = Path::new(&server.directory);
        for (name, bytes) in [("world", b"overworld".as_slice()), ("world_nether", b"nether blocks".as_slice()), ("world_the_end", b"end islands".as_slice())] {
            make_world(&directory.join(name));
            fs::write(directory.join(name).join("region/r.0.0.mca"), bytes).unwrap();
        }
        server
    }

    #[test]
    fn paper_families_list_duplicate_activate_export_and_delete_all_dimensions() {
        let root = std::env::temp_dir().join(format!("tch-paper-family-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let server = paper_fixture(&root);
        let directory = Path::new(&server.directory);
        let expected = ["world", "world_nether", "world_the_end"].iter().map(|name| backup::size(&directory.join(name))).sum::<u64>();
        let rows = list(&server).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "world"); assert!(rows[0].active);
        assert_eq!(rows[0].size_bytes, expected);
        assert_eq!(rows[0].dimensions, ["world_nether", "world_the_end"]);
        let copied = duplicate(&server, "world", "Adventure").unwrap();
        assert_eq!(copied.size_bytes, expected);
        assert_eq!(copied.dimensions, ["Adventure_nether", "Adventure_the_end"]);
        assert!(!copied.active);
        for (suffix, bytes) in [("", b"overworld".as_slice()), ("_nether", b"nether blocks".as_slice()), ("_the_end", b"end islands".as_slice())] {
            assert_eq!(fs::read(directory.join(format!("Adventure{suffix}/region/r.0.0.mca"))).unwrap(), bytes);
            assert_eq!(fs::read(directory.join(format!("world{suffix}/region/r.0.0.mca"))).unwrap(), bytes);
        }
        let exported = root.join("export"); fs::create_dir(&exported).unwrap();
        assert_eq!(export(&server, "world", &exported).unwrap(), exported.canonicalize().unwrap().join("world"));
        for suffix in ["", "_nether", "_the_end"] {
            assert_eq!(fs::read(exported.join(format!("world{suffix}/region/r.0.0.mca"))).unwrap(), fs::read(directory.join(format!("world{suffix}/region/r.0.0.mca"))).unwrap());
        }
        assert!(export(&server, "world", &directory.join("world_nether")).is_err());
        assert!(delete(&root, &server, "world").is_err());
        for dimension in ["world_nether", "world_the_end", "Adventure_nether", "Adventure_the_end"] {
            assert!(set_active(&root, &server, dimension).err().expect("paired dimension must be rejected").contains("Selecciona el mundo principal"));
            assert!(delete(&root, &server, dimension).is_err());
            assert!(duplicate(&server, dimension, "Isolated").is_err());
            assert!(export(&server, dimension, &exported).is_err());
        }
        assert!(storage::list_backups(&root, &server.id).unwrap().is_empty());
        let snapshot = set_active(&root, &server, "Adventure").unwrap().unwrap();
        assert_eq!(active_name(&server).unwrap(), "Adventure");
        for suffix in ["", "_nether", "_the_end"] { assert!(Path::new(&snapshot.path).join(format!("world{suffix}/level.dat")).is_file()); }
        // Protect the family even if an external editor previously selected its dimension directly.
        properties::write(&directory.join("server.properties"), [("level-name".into(), "Adventure_nether".into())].into()).unwrap();
        assert!(list(&server).unwrap().iter().find(|row| row.name == "Adventure").unwrap().active);
        assert!(delete(&root, &server, "Adventure").is_err());
        set_active(&root, &server, "Adventure").unwrap();
        let snapshot = delete(&root, &server, "world").unwrap();
        for suffix in ["", "_nether", "_the_end"] {
            assert!(!directory.join(format!("world{suffix}")).exists());
            assert!(Path::new(&snapshot.path).join(format!("world{suffix}/level.dat")).is_file());
            assert!(directory.join(format!("Adventure{suffix}/level.dat")).is_file());
        }
        assert_eq!(list(&server).unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paper_family_copy_preflights_all_names_and_cleans_failed_or_cancelled_stages() {
        use std::sync::{Arc, Mutex};
        let root = std::env::temp_dir().join(format!("tch-paper-family-copy-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap(); let server = paper_fixture(&root);
        let directory = PathBuf::from(&server.directory);
        for export in [false, true] {
            for mode in ["collision", "cancel", "invalid", "publication_failure", "complete"] {
                let parent = if export { root.join(format!("export-{mode}")) } else { directory.clone() };
                fs::create_dir_all(&parent).unwrap();
                let name = if export { "world" } else { mode };
                let target = parent.join(name);
                if mode == "collision" { fs::write(parent.join(format!("{name}_the_end")), b"unrelated").unwrap(); }
                let jobs: crate::file_tasks::Registry = Arc::new(Mutex::new(std::collections::HashMap::new()));
                let observed = jobs.clone(); let destination = parent.clone();
                let emit = move |event: crate::model::Progress| {
                    if mode == "cancel" && event.stage == "copying" && event.received > 0 { assert!(observed.lock().unwrap()[&event.server_id].request_cancel()); }
                    if mode == "invalid" && event.stage == "verifying" {
                        for entry in fs::read_dir(&destination).unwrap() {
                            let path = entry.unwrap().path();
                            if path.file_name().unwrap().to_string_lossy().starts_with(".tch-world-") { fs::write(path.join("level.dat"), b"invalid").unwrap(); }
                        }
                    }
                    if mode == "publication_failure" && event.stage == "publishing" {
                        // Remove the End's staged file after verification, forcing failure after the first two moves.
                        for entry in fs::read_dir(&destination).unwrap() {
                            let path = entry.unwrap().path();
                            if path.file_name().unwrap().to_string_lossy().starts_with(".tch-world-") && fs::read(path.join("region/r.0.0.mca")).is_ok_and(|bytes| bytes == b"end islands") {
                                fs::remove_dir_all(path).unwrap();
                            }
                        }
                    }
                };
                let result = if export { crate::file_tasks::export_world(&server, "world", &parent, &jobs, emit).map(|_| ()) }
                    else { crate::file_tasks::duplicate_world(&server, "world", name, &jobs, emit).map(|_| ()) };
                assert!(jobs.lock().unwrap().is_empty());
                if mode == "complete" { result.unwrap(); }
                else if mode == "cancel" { assert_eq!(result.unwrap_err(), crate::download::CANCELLED); }
                else { assert!(result.is_err()); }
                for suffix in ["", "_nether", "_the_end"] {
                    let path = parent.join(format!("{name}{suffix}"));
                    if mode == "collision" && suffix == "_the_end" { assert_eq!(fs::read(path).unwrap(), b"unrelated"); }
                    else { assert_eq!(path.exists(), mode == "complete"); }
                    assert!(directory.join(format!("world{suffix}/level.dat")).is_file());
                }
                assert!(fs::read_dir(&parent).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().starts_with(".tch-world-")));
                assert_eq!(target.exists(), mode == "complete");
            }
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paper_family_rejects_invalid_or_redirected_dimensions_without_mutation() {
        let root = std::env::temp_dir().join(format!("tch-paper-family-invalid-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap(); let server = paper_fixture(&root);
        let directory = Path::new(&server.directory);
        fs::write(directory.join("world_nether/level.dat"), b"invalid").unwrap();
        assert!(duplicate(&server, "world", "Copy").is_err());
        assert!(export(&server, "world", &root).is_err());
        assert!(set_active(&root, &server, "world").is_err());
        assert!(list(&server).unwrap_err().starts_with("No se pudo revisar la dimensión world_nether:"));
        assert!(!directory.join("Copy").exists());
        assert_eq!(fs::read(directory.join("world_the_end/region/r.0.0.mca")).unwrap(), b"end islands");
        fs::remove_dir_all(directory.join("world_nether")).unwrap();
        fs::write(directory.join("world_nether"), b"not a world directory").unwrap();
        assert!(duplicate(&server, "world", "Copy").is_err());
        fs::remove_file(directory.join("world_nether")).unwrap();
        #[cfg(windows)] {
            let outside = root.join("outside"); make_world(&outside);
            if std::os::windows::fs::symlink_dir(&outside, directory.join("world_nether")).is_ok() {
                assert!(duplicate(&server, "world", "Copy").is_err());
                assert!(set_active(&root, &server, "world_nether").is_err());
                fs::remove_dir(directory.join("world_nether")).unwrap();
                assert_eq!(fs::read(outside.join("region/r.0.0.mca")).unwrap(), b"world data");
            }
        }
        // With no split dimension, an unrelated destination suffix is still protected.
        fs::remove_dir_all(directory.join("world_the_end")).unwrap();
        make_world(&directory.join("Copy_nether"));
        assert!(duplicate(&server, "world", "Copy").is_err());
        assert!(!directory.join("Copy").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paper_destination_validation_rejects_aliases_and_late_reserved_suffix_collisions() {
        use std::sync::{Arc, Mutex};
        let root = std::env::temp_dir().join(format!("tch-paper-destination-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap(); let server = paper_fixture(&root);
        let directory = Path::new(&server.directory);
        for suffix in ["_nether", "_the_end"] { fs::remove_dir_all(directory.join(format!("world{suffix}"))).unwrap(); }
        assert!(duplicate(&server, "world", "world_nether").unwrap_err().contains("Selecciona el mundo principal"));
        assert!(!directory.join("world_nether").exists());
        let source = root.join("external"); make_world(&source);
        assert!(import(&server, &source, "world_nether").is_err());
        make_world(&directory.join("Taken_nether"));
        assert!(import(&server, &source, "Taken").is_err());
        assert!(!directory.join("Taken").exists());
        // A standalone source must not become a dimension of a different destination world.
        make_world(&directory.join("Adventure_nether"));
        let destination = root.join("alias-export"); fs::create_dir(&destination).unwrap();
        make_world(&destination.join("Adventure"));
        assert!(export(&server, "Adventure_nether", &destination).is_err());
        assert!(!destination.join("Adventure_nether").exists());
        // Reject redirected/invalid destination parents too, rather than merging into them.
        fs::write(destination.join("Invalid"), b"unrelated file").unwrap();
        make_world(&directory.join("Invalid_nether"));
        assert!(export(&server, "Invalid_nether", &destination).is_err());
        for operation in ["duplicate", "import", "export"] {
            let parent = if operation == "export" { let path = root.join("late-export"); fs::create_dir(&path).unwrap(); path } else { directory.to_path_buf() };
            let name = match operation { "duplicate" => "LateDuplicate", "import" => "LateImport", _ => "world" };
            let foreign = parent.join(format!("{name}_nether")); let inserted = foreign.clone();
            let jobs: crate::file_tasks::Registry = Arc::new(Mutex::new(std::collections::HashMap::new()));
            let emit = move |event: crate::model::Progress| {
                if event.stage == "publishing" { fs::write(&inserted, b"external change").unwrap(); }
            };
            let result = match operation {
                "duplicate" => crate::file_tasks::duplicate_world(&server, "world", name, &jobs, emit).map(|_| ()),
                "import" => crate::file_tasks::import_world(&server, &source, name, &jobs, emit).map(|_| ()),
                _ => crate::file_tasks::export_world(&server, "world", &parent, &jobs, emit).map(|_| ()),
            };
            assert!(result.is_err()); assert!(jobs.lock().unwrap().is_empty());
            assert!(!parent.join(name).exists());
            assert_eq!(fs::read(foreign).unwrap(), b"external change");
            assert!(fs::read_dir(&parent).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().starts_with(".tch-world-")));
        }
        assert_eq!(fs::read(source.join("region/r.0.0.mca")).unwrap(), b"world data");
        assert_eq!(fs::read(directory.join("world/region/r.0.0.mca")).unwrap(), b"overworld");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paper_64_character_primary_keeps_derived_dimensions_across_world_operations() {
        let root = std::env::temp_dir().join(format!("tch-paper-long-family-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap(); let mut server = fixture(&root); server.platform = "paper".into();
        let directory = Path::new(&server.directory);
        let name = "PrimaryWorld1234PrimaryWorld1234PrimaryWorld1234PrimaryWorld1234";
        let copy = "CopiedWorld12345CopiedWorld12345CopiedWorld12345CopiedWorld12345";
        assert_eq!(name.chars().count(), 64); assert_eq!(copy.chars().count(), 64);
        for (suffix, bytes) in [("", b"long overworld".as_slice()), ("_nether", b"long nether".as_slice()), ("_the_end", b"long end".as_slice())] {
            make_world(&directory.join(format!("{name}{suffix}")));
            fs::write(directory.join(format!("{name}{suffix}/region/r.0.0.mca")), bytes).unwrap();
        }
        let rows = list(&server).unwrap(); assert_eq!(rows.len(), 1); assert_eq!(rows[0].name, name);
        assert_eq!(rows[0].dimensions, [format!("{name}_nether"), format!("{name}_the_end")]);
        let copied = duplicate(&server, name, copy).unwrap();
        assert_eq!(copied.dimensions, [format!("{copy}_nether"), format!("{copy}_the_end")]);
        assert_eq!(copied.size_bytes, rows[0].size_bytes);
        assert_eq!(fs::read(directory.join(format!("{copy}_nether/region/r.0.0.mca"))).unwrap(), b"long nether");
        let destination = root.join("folder"); fs::create_dir(&destination).unwrap();
        export(&server, name, &destination).unwrap();
        assert_eq!(fs::read(destination.join(format!("{name}_the_end/region/r.0.0.mca"))).unwrap(), b"long end");
        let task = crate::download::DownloadTask::new(&server.id, "world_export", |_| {}).unwrap();
        let zip_path = root.join("long-family.zip");
        crate::world_bundle::export(&server, name, &zip_path, false, &task).unwrap();
        let mut zip = zip::ZipArchive::new(File::open(zip_path).unwrap()).unwrap();
        let mut bytes = Vec::new();
        zip.by_name(&format!("worlds/{name}_nether/region/r.0.0.mca")).unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"long nether"); drop(zip);
        set_active(&root, &server, copy).unwrap(); assert_eq!(active_name(&server).unwrap(), copy);
        assert!(set_active(&root, &server, &format!("{copy}_nether")).err().expect("paired dimension must be rejected").contains("Selecciona el mundo principal"));
        assert!(delete(&root, &server, &format!("{copy}_the_end")).is_err());
        assert!(delete(&root, &server, copy).is_err());
        // Derived folders may exceed 64 characters; user-created primary names still may not.
        assert!(duplicate(&server, name, &format!("{copy}X")).is_err());
        assert!(import(&server, &destination.join(name), &format!("{copy}X")).is_err());
        for invalid in ["../world_nether", "CON._nether", " invalid_nether"] { assert!(checked_world(&server, invalid).is_err()); }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn internal_dimension_engines_do_not_adopt_paper_style_siblings() {
        let root = std::env::temp_dir().join(format!("tch-internal-worlds-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap(); let mut server = paper_fixture(&root);
        let directory = Path::new(&server.directory);
        fs::create_dir_all(directory.join("world/DIM-1/region")).unwrap();
        fs::write(directory.join("world/DIM-1/region/nether.mca"), b"internal nether").unwrap();
        for platform in ["vanilla", "fabric", "neoforge"] {
            server.platform = platform.into();
            assert_eq!(list(&server).unwrap().len(), 3);
            let copy = duplicate(&server, "world", platform).unwrap();
            assert!(copy.dimensions.is_empty());
            assert_eq!(fs::read(directory.join(platform).join("DIM-1/region/nether.mca")).unwrap(), b"internal nether");
            assert!(!directory.join(format!("{platform}_nether")).exists());
            fs::remove_dir_all(directory.join(platform)).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
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
