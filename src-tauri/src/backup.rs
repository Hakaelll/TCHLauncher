use crate::model::Backup;
use crate::storage;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

pub fn size(path: &Path) -> u64 {
    WalkDir::new(path).into_iter().filter_map(Result::ok)
        .filter_map(|e| if e.file_type().is_file() { e.metadata().ok().map(|m| m.len()) } else { None })
        .sum()
}

pub(crate) fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    for entry in WalkDir::new(from).into_iter() {
        let entry = entry.map_err(|e| e.to_string())?;
        let relative = entry.path().strip_prefix(from).map_err(|e| e.to_string())?;
        let dest = to.join(relative);
        if entry.file_type().is_symlink() { return Err("Los enlaces simbólicos no se copian por seguridad".into()); }
        if entry.file_type().is_dir() { fs::create_dir_all(dest).map_err(|e| e.to_string())?; }
        else if entry.file_type().is_file() { fs::copy(entry.path(),dest).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

pub fn create(root: &Path, server_id: &str) -> Result<Backup, String> {
    let server = storage::get(root,server_id)?;
    let source = Path::new(&server.directory).canonicalize().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let base = root.join("backups").join(server_id);
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    if base.canonicalize().map_err(|e| e.to_string())?.starts_with(&source) {
        return Err("La carpeta de backups está dentro del servidor; elige otra ubicación para el servidor".into());
    }
    let stage = base.join(format!(".{id}.part"));
    let target = base.join(&id);
    let result = (|| {
        copy_tree(&source, &stage)?;
        fs::rename(&stage,&target).map_err(|e| e.to_string())?;
        let backup = Backup {id,server_id:server_id.into(),created_at:chrono::Utc::now().to_rfc3339(),size_bytes:size(&target),path:target.to_string_lossy().into()};
        storage::add_backup(root,&backup)?;
        Ok(backup)
    })();
    if result.is_err() { let _ = fs::remove_dir_all(stage); }
    result
}

pub fn restore(root: &Path, server_id: &str, backup_id: &str) -> Result<(), String> {
    let server = storage::get(root,server_id)?;
    let source = storage::backup_path(root,server_id,backup_id)?;
    if !source.is_dir() || !source.join("server.properties").is_file() { return Err("El backup está incompleto o no es válido".into()); }
    create(root,server_id)?;
    let target = Path::new(&server.directory);
    let stage = target.with_extension(format!("restore-{}",Uuid::new_v4()));
    copy_tree(&source,&stage)?;
    let previous = target.with_extension(format!("previous-{}",Uuid::new_v4()));
    fs::rename(target,&previous).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&stage,target) {
        let _ = fs::rename(&previous,target);
        return Err(format!("No se pudo activar el backup: {e}"));
    }
    fs::remove_dir_all(previous).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copies_nested_files() {
        let root = std::env::temp_dir().join(format!("tch-backup-test-{}",Uuid::new_v4()));
        let from = root.join("from"); let to = root.join("to");
        fs::create_dir_all(from.join("world/region")).unwrap();
        fs::write(from.join("world/region/r.0.0.mca"),b"abc").unwrap();
        copy_tree(&from,&to).unwrap();
        assert_eq!(fs::read(to.join("world/region/r.0.0.mca")).unwrap(),b"abc");
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn backup_and_restore_roundtrip() {
        let root = std::env::temp_dir().join(format!("tch-roundtrip-{}",Uuid::new_v4()));
        let server = storage::create(&root,crate::model::CreateServer {
            name:"Roundtrip".into(),version:"1.21.4".into(),platform:"vanilla".into(),
            memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None,
        }).unwrap();
        let world = Path::new(&server.directory).join("world");
        fs::create_dir_all(&world).unwrap();
        fs::write(world.join("level.dat"),b"original").unwrap();
        let saved = create(&root,&server.id).unwrap();
        fs::write(world.join("level.dat"),b"changed").unwrap();
        restore(&root,&server.id,&saved.id).unwrap();
        assert_eq!(fs::read(world.join("level.dat")).unwrap(),b"original");
        assert_eq!(storage::list_backups(&root,&server.id).unwrap().len(),2);
        let _ = fs::remove_dir_all(root);
    }
}
