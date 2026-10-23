use crate::{backup, download::DownloadTask, model::Server, world};
use serde::Serialize;
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub path: String,
    pub suggested_name: String,
    pub size_bytes: u64,
    pub dimensions: Vec<String>,
}

fn checked_source(path: &Path) -> Result<PathBuf, String> {
    for ancestor in path.ancestors().filter(|path| path.file_name().is_some()) {
        let meta = fs::symlink_metadata(ancestor).map_err(|e| e.to_string())?;
        if !meta.is_dir() || !world::plain(&meta) {
            return Err("La carpeta del mundo contiene enlaces o redirecciones".into());
        }
    }
    let source = path.canonicalize().map_err(|e| format!("No se pudo abrir la carpeta: {e}"))?;
    world::valid_level_dat(&source)?;
    Ok(source)
}

fn family(path: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let source = checked_source(path)?;
    let name = source.file_name().and_then(|name| name.to_str()).ok_or("Nombre de carpeta de mundo no válido")?;
    let parent = source.parent().ok_or("Selecciona una carpeta de mundo")?;
    for suffix in world::DIMENSION_SUFFIXES {
        if name.to_ascii_lowercase().ends_with(suffix) {
            let base = &name[..name.len() - suffix.len()];
            if world::valid_level_dat(&parent.join(base)).is_ok() {
                return Err(format!("Esa dimensión pertenece al mundo {base}. Selecciona el mundo principal"));
            }
        }
    }
    let mut parts = vec![(name.to_owned(), source.clone())];
    for suffix in world::DIMENSION_SUFFIXES {
        let sibling = format!("{name}{suffix}");
        let path = parent.join(&sibling);
        match fs::symlink_metadata(&path) {
            Ok(_) => parts.push((sibling, checked_source(&path)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(parts)
}

pub fn inspect(path: &Path) -> Result<Inspection, String> {
    let parts = family(path)?;
    Ok(Inspection {
        path: parts[0].1.to_string_lossy().into_owned(),
        suggested_name: parts[0].0.clone(),
        size_bytes: parts.iter().map(|(_, path)| backup::size(path)).sum(),
        dimensions: parts.iter().skip(1).map(|(name, _)| name.clone()).collect(),
    })
}

pub fn import(root: &Path, server: &Server, source: &Path, name: &str, dimensions: &[String], task: &DownloadTask) -> Result<world::WorldInfo, String> {
    task.cancellation.check()?;
    task.report("Comprobando la carpeta del mundo", "preparing");
    let parts = family(source)?;
    let actual: Vec<_> = parts.iter().skip(1).map(|(name, _)| name.clone()).collect();
    if actual != dimensions {
        return Err("Las dimensiones del mundo han cambiado. Vuelve a elegir la carpeta antes de importar".into());
    }
    if !dimensions.is_empty() && server.platform != "paper" {
        return Err("Este mundo tiene dimensiones separadas de Paper. Elige un servidor Paper; no se convierten automáticamente a otros tipos".into());
    }
    let parent = Path::new(&server.directory).canonicalize().map_err(|e| e.to_string())?;
    let active = crate::properties::read(&parent.join("server.properties"))?.get("level-name").map_or(name == "world", |active| active == name);
    let copied = world::copy_sources(root, server, &parts, &parts[0].0, name, &parent, Some(task))?;
    Ok(world::world_info(name.into(), &copied, active))
}
