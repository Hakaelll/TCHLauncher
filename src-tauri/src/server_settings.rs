//! Publish file properties and SQLite capacity as one recoverable settings operation.
use crate::{properties, storage};
use rusqlite::{params, OptionalExtension};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

fn database(root: &Path) -> Result<rusqlite::Connection, String> {
    let db = storage::connect(root)?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS pending_server_settings(server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,directory TEXT NOT NULL,before BLOB NOT NULL,after BLOB NOT NULL);").map_err(|e|e.to_string())?;
    Ok(db)
}
fn paths(root: &Path, id: &str) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    uuid::Uuid::parse_str(id).map_err(|_| "Identificador de servidor no válido")?;
    let server = storage::get(root, id)?;
    let directory = PathBuf::from(server.directory)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let target = directory.join("server.properties");
    let stage = directory.join(format!(".tch-settings-{id}.new"));
    let old = directory.join(format!(".tch-settings-{id}.old"));
    for path in [&target, &stage, &old] {
        if let Ok(meta) = fs::symlink_metadata(path) {
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 1_048_576 {
                return Err("El guardado no admite enlaces, directorios ni archivos de propiedades mayores de 1 MB".into());
            }
        }
    }
    Ok((target, stage, old))
}
fn bounded(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1_048_576 {
        return Err("El archivo de propiedades supera 1 MB".into());
    }
    Ok(bytes)
}
fn publish(target: &Path, stage: &Path, old: &Path, bytes: &[u8]) -> Result<(), String> {
    if target.exists() && old.exists() {
        return Err("Ya existe una copia de recuperación; no se sobrescribirá".into());
    }
    let permissions = if target.exists() {
        Some(
            fs::metadata(target)
                .map_err(|e| e.to_string())?
                .permissions(),
        )
    } else if old.exists() {
        Some(fs::metadata(old).map_err(|e| e.to_string())?.permissions())
    } else {
        None
    };
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(stage)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)
            .map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if target.exists() {
        fs::rename(target, old).map_err(|e| e.to_string())?;
    }
    fs::rename(stage, target).map_err(|e| e.to_string())
}
fn cleanup(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn recover(root: &Path, id: &str) -> Result<(), String> {
    let db = database(root)?;
    let pending = db
        .query_row(
            "SELECT directory,before,after FROM pending_server_settings WHERE server_id=?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;
    // Ordinary registered folders can be unavailable (for example an unplugged
    // external disk). Without pending work, recovery must not prevent app startup.
    if pending.is_none() {
        let server = storage::get(root, id)?;
        let directory = Path::new(&server.directory);
        if !directory.join(format!(".tch-settings-{id}.new")).exists()
            && !directory.join(format!(".tch-settings-{id}.old")).exists()
        {
            return Ok(());
        }
    }
    let (target, stage, old) = paths(root, id)?;
    let Some((directory, before, after)) = pending else {
        if !target.exists() && old.exists() {
            return Err("Falta server.properties y existe una copia anterior sin registro pendiente. Se conserva para revisión.".into());
        }
        cleanup(&stage)?;
        cleanup(&old)?;
        return Ok(());
    };
    if before.len() > 1_048_576
        || after.len() > 1_048_576
        || target.parent().unwrap() != Path::new(&directory)
    {
        return Err("Registro de recuperación de configuración no válido".into());
    }
    if target.exists() {
        let current = bounded(&target)?;
        if current != before && current != after {
            return Err("Las propiedades cambiaron fuera del launcher durante el guardado. Conservamos la recuperación pendiente para evitar sobrescribirlas.".into());
        }
    }
    if old.exists() {
        let bytes = bounded(&old)?;
        if bytes != before && bytes != after {
            return Err(
                "La copia de recuperación de propiedades cambió; no se sobrescribirá".into(),
            );
        }
    }
    // Restart recovery is idempotent even if it was interrupted during restoration.
    cleanup(&stage)?;
    if !target.exists() || bounded(&target)? != before {
        if target.exists() {
            cleanup(&old)?;
        }
        publish(&target, &stage, &old, &before)?;
    }
    db.execute(
        "DELETE FROM pending_server_settings WHERE server_id=?1",
        [id],
    )
    .map_err(|e| e.to_string())?;
    cleanup(&old)?;
    Ok(())
}
pub fn recover_all(root: &Path) -> Result<(), String> {
    for server in storage::list(root)? {
        recover(root, &server.id)?;
    }
    Ok(())
}

pub fn save(
    root: &Path,
    id: &str,
    values: BTreeMap<String, String>,
    min: u32,
    max: u32,
    gb: u32,
) -> Result<(), String> {
    if min < 512 || max > 65536 || min > max {
        return Err("Valores de memoria no válidos".into());
    }
    if !(1..=2048).contains(&gb) {
        return Err("El objetivo de espacio debe estar entre 1 y 2048 GB".into());
    }
    recover(root, id)?;
    let (target, stage, old) = paths(root, id)?;
    if fs::metadata(&target)
        .map_err(|e| e.to_string())?
        .permissions()
        .readonly()
    {
        return Err(
            "server.properties está en modo de solo lectura. Quita ese atributo antes de guardar."
                .into(),
        );
    }
    let before = bounded(&target)?;
    let after = properties::prepare(&target, values)?.into_bytes();
    if after.len() > 1_048_576 {
        return Err("El archivo de propiedades supera 1 MB".into());
    }
    let mut db = database(root)?;
    db.execute("INSERT INTO pending_server_settings(server_id,directory,before,after) VALUES (?1,?2,?3,?4)",params![id,target.parent().unwrap().to_string_lossy(),before,after]).map_err(|e|e.to_string())?;
    let result = (|| {
        // Reject external edits after preparation instead of replacing them.
        if bounded(&target)? != before {
            return Err("Las propiedades cambiaron mientras se preparaba el guardado".into());
        }
        publish(&target, &stage, &old, &after)?;
        let tx = db.transaction().map_err(|e| e.to_string())?;
        if tx.execute("UPDATE servers SET memory_min_mb=?2,memory_max_mb=?3,storage_target_gb=?4 WHERE id=?1",params![id,min,max,gb]).map_err(|e|e.to_string())?!=1{return Err("No se encontró el servidor".into());}
        tx.execute(
            "DELETE FROM pending_server_settings WHERE server_id=?1",
            [id],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    })();
    drop(db);
    if let Err(error) = result {
        return match recover(root, id) {
            Ok(()) => Err(error),
            Err(recovery) => Err(format!("{error}. Recuperación pendiente: {recovery}")),
        };
    }
    // Committed state must never be described as a failure just because cleanup failed.
    let _ = cleanup(&old);
    Ok(())
}
