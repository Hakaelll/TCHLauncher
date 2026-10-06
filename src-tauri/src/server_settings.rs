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
#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture() -> (Fixture, String) {
        let root =
            Fixture(std::env::temp_dir().join(format!("tch-settings-{}", uuid::Uuid::new_v4())));
        let server = storage::create(
            &root.0,
            crate::model::CreateServer {
                name: "Settings test".into(),
                version: "1.21.4".into(),
                platform: "vanilla".into(),
                memory_min_mb: 1024,
                memory_max_mb: 2048,
                max_players: 20,
                port: 25565,
                storage_target_gb: 20,
                image_path: None,
            },
        )
        .unwrap();
        (root, server.id)
    }
    fn journal(root: &Path, id: &str, after: &[u8]) -> Vec<u8> {
        let (target, _, _) = paths(root, id).unwrap();
        let before = bounded(&target).unwrap();
        database(root).unwrap().execute("INSERT INTO pending_server_settings(server_id,directory,before,after) VALUES (?1,?2,?3,?4)",params![id,target.parent().unwrap().to_string_lossy(),before,after]).unwrap();
        before
    }
    #[test]
    fn saves_capacity_and_properties_together_and_invalid_requests_change_neither() {
        let (root, id) = fixture();
        let (target, _, _) = paths(&root.0, &id).unwrap();
        let mut values = properties::read(&target).unwrap();
        values.insert("max-players".into(), "40".into());
        values.insert("motd".into(), "My world".into());
        save(&root.0, &id, values, 2048, 4096, 48).unwrap();
        let server = storage::get(&root.0, &id).unwrap();
        assert_eq!(
            (
                server.memory_min_mb,
                server.memory_max_mb,
                server.storage_target_gb
            ),
            (2048, 4096, 48)
        );
        assert_eq!(properties::read(&target).unwrap()["max-players"], "40");
        assert_eq!(properties::read(&target).unwrap()["motd"], "My world");
        let bytes = bounded(&target).unwrap();
        for (min, max, gb, port) in [
            (1, 4096, 48, "25565"),
            (2048, 1024, 48, "25565"),
            (2048, 4096, 0, "25565"),
            (2048, 4096, 48, "70000"),
        ] {
            let mut values = properties::read(&target).unwrap();
            values.insert("server-port".into(), port.into());
            assert!(save(&root.0, &id, values, min, max, gb).is_err());
            assert_eq!(bounded(&target).unwrap(), bytes);
            let server = storage::get(&root.0, &id).unwrap();
            assert_eq!(
                (
                    server.memory_min_mb,
                    server.memory_max_mb,
                    server.storage_target_gb
                ),
                (2048, 4096, 48)
            );
        }
        recover_all(&root.0).unwrap();
        assert_eq!(bounded(&target).unwrap(), bytes);
    }
    #[test]
    fn database_failure_after_file_publication_restores_original_bytes() {
        let (root, id) = fixture();
        let (target, stage, old) = paths(&root.0, &id).unwrap();
        let before = bounded(&target).unwrap();
        database(&root.0).unwrap().execute_batch("CREATE TRIGGER reject_settings BEFORE UPDATE OF memory_max_mb ON servers BEGIN SELECT RAISE(ABORT,'test update rejected'); END;").unwrap();
        let mut values = properties::read(&target).unwrap();
        values.insert("motd".into(), "Changed".into());
        assert!(save(&root.0, &id, values, 2048, 4096, 48)
            .unwrap_err()
            .contains("test update rejected"));
        assert_eq!(bounded(&target).unwrap(), before);
        let server = storage::get(&root.0, &id).unwrap();
        assert_eq!(
            (
                server.memory_min_mb,
                server.memory_max_mb,
                server.storage_target_gb
            ),
            (1024, 2048, 20)
        );
        assert!(!stage.exists() && !old.exists());
        assert_eq!(
            database(&root.0)
                .unwrap()
                .query_row("SELECT count(*) FROM pending_server_settings", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn interrupted_file_publication_and_restoration_are_recovered_before_next_save() {
        for phase in 0..4 {
            let (root, id) = fixture();
            let (target, stage, old) = paths(&root.0, &id).unwrap();
            let after = b"# changed\nserver-port=25566\n";
            let before = journal(&root.0, &id, after);
            match phase {
                0 => fs::write(&stage, after).unwrap(),
                1 => {
                    fs::write(&stage, after).unwrap();
                    fs::rename(&target, &old).unwrap();
                }
                2 => publish(&target, &stage, &old, after).unwrap(),
                _ => {
                    publish(&target, &stage, &old, after).unwrap();
                    cleanup(&old).unwrap();
                    fs::write(&stage, &before).unwrap();
                    fs::rename(&target, &old).unwrap();
                }
            }
            recover_all(&root.0).unwrap();
            recover_all(&root.0).unwrap();
            assert_eq!(bounded(&target).unwrap(), before);
            assert!(!stage.exists() && !old.exists());
            let server = storage::get(&root.0, &id).unwrap();
            assert_eq!(server.memory_max_mb, 2048);
        }
    }
    #[test]
    fn recovery_preserves_external_changes_and_rejects_a_changed_directory() {
        let (root, id) = fixture();
        let (target, stage, old) = paths(&root.0, &id).unwrap();
        journal(&root.0, &id, b"pending");
        fs::write(&target, b"external edit\n").unwrap();
        assert!(recover(&root.0, &id)
            .unwrap_err()
            .contains("fuera del launcher"));
        assert_eq!(bounded(&target).unwrap(), b"external edit\n");
        assert!(!stage.exists() && !old.exists());
        database(&root.0)
            .unwrap()
            .execute(
                "UPDATE pending_server_settings SET directory=?2 WHERE server_id=?1",
                params![id, root.0.to_string_lossy()],
            )
            .unwrap();
        assert!(recover(&root.0, &id).unwrap_err().contains("no válido"));
        assert_eq!(bounded(&target).unwrap(), b"external edit\n");
    }

    #[test]
    fn an_unavailable_folder_without_pending_work_does_not_block_recovery() {
        let (root, id) = fixture();
        let server = storage::get(&root.0, &id).unwrap();
        let directory = PathBuf::from(&server.directory);
        let moved = directory.with_extension("temporarily-unavailable");
        fs::rename(&directory, &moved).unwrap();
        recover_all(&root.0).unwrap();
        assert!(!directory.exists());
        assert!(save(&root.0, &id, BTreeMap::new(), 1024, 2048, 20).is_err());
        assert_eq!(storage::get(&root.0, &id).unwrap().memory_max_mb, 2048);
        fs::rename(&moved, &directory).unwrap();
    }

    #[test]
    fn missing_committed_properties_never_discards_the_remaining_copy() {
        let (root, id) = fixture();
        let (target, stage, old) = paths(&root.0, &id).unwrap();
        let bytes = bounded(&target).unwrap();
        fs::rename(&target, &old).unwrap();
        fs::write(&stage, b"partial replacement").unwrap();
        assert!(recover(&root.0, &id)
            .unwrap_err()
            .contains("sin registro pendiente"));
        assert_eq!(bounded(&old).unwrap(), bytes);
        assert_eq!(bounded(&stage).unwrap(), b"partial replacement");
        assert!(!target.exists());
    }

    #[test]
    fn read_only_properties_and_existing_recovery_files_are_not_replaced() {
        let (root, id) = fixture();
        let (target, stage, old) = paths(&root.0, &id).unwrap();
        let bytes = bounded(&target).unwrap();
        let permissions = fs::metadata(&target).unwrap().permissions();
        let mut read_only = permissions.clone();
        read_only.set_readonly(true);
        fs::set_permissions(&target, read_only).unwrap();
        let result = save(
            &root.0,
            &id,
            properties::read(&target).unwrap(),
            2048,
            4096,
            48,
        );
        fs::set_permissions(&target, permissions).unwrap();
        assert!(result.unwrap_err().contains("solo lectura"));
        assert_eq!(bounded(&target).unwrap(), bytes);
        assert_eq!(storage::get(&root.0, &id).unwrap().memory_max_mb, 2048);
        assert!(!stage.exists() && !old.exists());
        fs::write(&old, b"existing copy").unwrap();
        assert!(publish(&target, &stage, &old, b"new properties").is_err());
        assert_eq!(bounded(&target).unwrap(), bytes);
        assert_eq!(bounded(&old).unwrap(), b"existing copy");
        assert!(!stage.exists());
    }

    #[test]
    fn filesystem_rename_failure_keeps_original_and_can_resume_recovery() {
        let (root, id) = fixture();
        let (target, stage, old) = paths(&root.0, &id).unwrap();
        let before = journal(&root.0, &id, b"prepared");
        // A missing destination parent causes a real rename failure after staging.
        let blocked = target.with_extension("blocked").join("old");
        assert!(publish(&target, &stage, &blocked, b"prepared").is_err());
        assert_eq!(bounded(&target).unwrap(), before);
        assert!(stage.exists());
        assert!(!blocked.parent().unwrap().exists());
        assert_eq!(storage::get(&root.0, &id).unwrap().memory_max_mb, 2048);
        recover(&root.0, &id).unwrap();
        assert_eq!(bounded(&target).unwrap(), before);
        assert!(!stage.exists() && !old.exists());
        assert!(!blocked.parent().unwrap().exists());
    }
}
