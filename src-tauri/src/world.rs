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

pub(crate) fn is_world(server:&Server,name:&str)->bool {checked_world(server,name).is_ok()}

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

#[cfg(test)]
fn copy_as(source: &Path, root: &Path, name: &str) -> Result<PathBuf, String> {
    copy_as_inner(source, root, name, None)
}

fn copy_as_inner(source: &Path, root: &Path, name: &str, task: Option<&crate::download::DownloadTask>) -> Result<PathBuf, String> {
    valid_name(name)?;
    let target = root.join(name);
    if target.exists() { return Err("Ya existe un archivo o mundo con ese nombre".into()); }
    let stage = root.join(format!(".tch-world-{}.part", Uuid::new_v4()));
    let result = (|| {
        let action=match task.map(|task|task.kind()){Some("world_duplicate")=>"Duplicando archivos del mundo",Some("world_export")=>"Exportando archivos del mundo",_=>"Importando archivos del mundo"};
        if let Some(task)=task { backup::copy_tree_tracked(source,&stage,task,action)?; }
        else { backup::copy_tree(source, &stage)?; }
        if let Some(task)=task { task.cancellation.check()?; task.report("Comprobando la copia del mundo","verifying"); }
        valid_level_dat(&stage)?;
        if let Some(task)=task { task.begin_publication("Guardando la copia del mundo")?; }
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
    let target = copy_as_inner(source, &root, name, task)?;
    Ok(world_info(name.into(), &target, active))
}

#[cfg(test)]
pub fn duplicate(server: &Server, name: &str, new_name: &str) -> Result<WorldInfo, String> {
    let source = checked_world(server, name)?;
    let target = copy_as(&source, &server_root(server)?, new_name)?;
    Ok(world_info(new_name.into(), &target, false))
}

pub fn duplicate_tracked(server:&Server,name:&str,new_name:&str,task:&crate::download::DownloadTask)->Result<WorldInfo,String>{
    task.cancellation.check()?;task.report("Comprobando el mundo original","preparing");
    let source=checked_world(server,name)?;
    let target=copy_as_inner(&source,&server_root(server)?,new_name,Some(task))?;
    Ok(world_info(new_name.into(),&target,false))
}

#[cfg(test)]
pub fn export(server: &Server, name: &str, parent: &Path) -> Result<PathBuf, String> {
    let source = checked_world(server, name)?;
    let parent = parent.canonicalize().map_err(|e| format!("No se pudo abrir el destino: {e}"))?;
    if !parent.is_dir() || parent.starts_with(&source) {
        return Err("Elige una carpeta de destino fuera del mundo".into());
    }
    copy_as(&source, &parent, name)
}

pub fn export_tracked(server:&Server,name:&str,parent:&Path,task:&crate::download::DownloadTask)->Result<PathBuf,String>{
    task.cancellation.check()?;task.report("Comprobando el destino del mundo","preparing");
    let source=checked_world(server,name)?;
    let parent=parent.canonicalize().map_err(|e|format!("No se pudo abrir el destino: {e}"))?;
    if !parent.is_dir()||parent.starts_with(&source){return Err("Elige una carpeta de destino fuera del mundo".into());}
    copy_as_inner(&source,&parent,name,Some(task))
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
    checked_world(server, name)?;
    if active_name(server)? == name { return Ok(None); }
    let snapshot = if let Some(task)=task{backup::create_tracked(root,&server.id,task)?}else{backup::create(root,&server.id)?};
    if let Some(task)=task{
        task.report("Copia guardada; preparando el cambio de mundo","preparing");
        task.cancellation.check()?;
    }
    checked_world(server,name)?;
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
    checked_world(server, name)?;
    if active_name(server)? == name { return Err("No se puede borrar el mundo principal. Activa otro primero".into()); }
    let snapshot = if let Some(task)=task{backup::create_tracked(root,&server.id,task)?}else{backup::create(root,&server.id)?};
    if let Some(task)=task{
        task.report("Copia guardada; preparando la eliminación del mundo","preparing");
        task.cancellation.check()?;
    }
    let target=checked_world(server,name)?;
    if active_name(server)?==name{return Err("No se puede borrar el mundo principal. Activa otro primero".into());}
    if let Some(task)=task{task.begin_publication("Eliminando el mundo; la copia está guardada")?;}
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
