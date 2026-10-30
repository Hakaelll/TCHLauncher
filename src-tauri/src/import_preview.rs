use serde::Serialize;
use std::{collections::BTreeMap, fs, io::Read, path::{Path, PathBuf}};

const MAX_FILE: u64 = 2_000_000;
const MAX_ENTRIES: usize = 5000;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub worlds: Vec<String>,
    pub mods: Vec<String>,
    pub plugins: Vec<String>,
    pub properties: BTreeMap<String, String>,
    pub permissions: Vec<PermissionFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionFile {
    pub filename: String,
    pub entries: Option<u32>,
}

fn plain(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() { return false; }
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return false; }
    }
    true
}

fn checked(path: &Path, parent: &Path, directory: bool) -> Result<fs::Metadata, ()> {
    let meta = fs::symlink_metadata(path).map_err(|_| ())?;
    if !plain(&meta) || (if directory { !meta.is_dir() } else { !meta.is_file() })
        || path.canonicalize().map_err(|_| ())?.parent() != Some(parent) { return Err(()); }
    Ok(meta)
}

fn warn(warnings: &mut Vec<String>, filename: &str) {
    if warnings.len() > 32 { return; }
    let name: String = filename.chars().take(128).map(|c| if c.is_control() { '�' } else { c }).collect();
    let warning = format!("{name}: vista previa incompleta; archivo no válido, inaccesible o fuera de los límites.");
    if !warnings.contains(&warning) {
        warnings.push(if warnings.len() == 32 { "Hay más avisos; la vista previa está incompleta.".into() } else { warning });
    }
}

fn children(folder: &Path, warnings: &mut Vec<String>, label: &str) -> Vec<PathBuf> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => { warn(warnings, label); return Vec::new(); }
    };
    let mut paths = Vec::new();
    for (index, entry) in entries.take(MAX_ENTRIES).enumerate() {
        match entry { Ok(entry) => paths.push(entry.path()), Err(_) => warn(warnings, label) }
        if index + 1 == MAX_ENTRIES { warn(warnings, label); }
    }
    paths.sort();
    paths
}

fn read(root: &Path, filename: &str) -> Result<Option<Vec<u8>>, ()> {
    let path = root.join(filename);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
        Ok(_) => (),
    }
    if checked(&path, root, false)?.len() > MAX_FILE { return Err(()); }
    let mut bytes = Vec::new();
    fs::File::open(path).map_err(|_| ())?.take(MAX_FILE + 1).read_to_end(&mut bytes).map_err(|_| ())?;
    if bytes.len() as u64 > MAX_FILE { return Err(()); }
    Ok(Some(bytes))
}

fn content(root: &Path, folder: &str, warnings: &mut Vec<String>) -> Vec<String> {
    let path = root.join(folder);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        _ => (),
    }
    if checked(&path, root, true).is_err() { warn(warnings, folder); return Vec::new(); }
    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => { warn(warnings, folder); return Vec::new(); }
    };
    children(&path, warnings, folder).into_iter().filter_map(|file| {
        let name = file.file_name()?.to_str()?.to_owned();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".jar") && !lower.ends_with(".jar.disabled") { return None; }
        if checked(&file, &path, false).is_err() { warn(warnings, &name); return None; }
        Some(name)
    }).collect()
}

fn property_value(key: &str, value: &str) -> Option<bool> {
    let valid = match key {
        "server-port" => value.parse::<u16>().is_ok_and(|n| n > 0),
        "max-players" | "view-distance" | "simulation-distance" => value.parse::<u32>().is_ok(),
        "gamemode" => ["survival", "creative", "adventure", "spectator", "0", "1", "2", "3"].contains(&value),
        "difficulty" => ["peaceful", "easy", "normal", "hard", "0", "1", "2", "3"].contains(&value),
        "online-mode" | "white-list" | "enforce-whitelist" => ["true", "false"].contains(&value),
        "level-name" => !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control),
        _ => return None,
    };
    Some(value.len() <= 128 && valid)
}

pub fn inspect(path: &Path) -> Result<crate::model::ImportInspection, String> {
    let source = crate::storage::inspect_launchers(path)?;
    let inventory = inventory(Path::new(&source.path));
    Ok(crate::model::ImportInspection { source, inventory })
}

fn inventory(root: &Path) -> Inventory {
    let mut inventory = Inventory::default();
    let mut world_checks = 0;
    for folder in children(root, &mut inventory.warnings, ".") {
        let name = folder.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let meta = match fs::symlink_metadata(&folder) {
            Ok(meta) => meta,
            Err(_) => { warn(&mut inventory.warnings, &name); continue; }
        };
        if !meta.is_dir() { continue; }
        if checked(&folder, root, true).is_err() { warn(&mut inventory.warnings, &name); continue; }
        match fs::symlink_metadata(folder.join("level.dat")) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Ok(_) => {
                if world_checks == 32 { warn(&mut inventory.warnings, "."); break; }
                world_checks += 1;
                if checked(&folder.join("level.dat"), &folder, false).is_ok()
                    && crate::world::valid_level_dat(&folder).is_ok() { inventory.worlds.push(name); }
                else { warn(&mut inventory.warnings, &name); }
            },
            _ => warn(&mut inventory.warnings, &name),
        }
    }
    inventory.mods = content(root, "mods", &mut inventory.warnings);
    inventory.plugins = content(root, "plugins", &mut inventory.warnings);
    match read(root, "server.properties") {
        Ok(Some(bytes)) => match String::from_utf8(bytes) {
            Ok(text) => for (key,value) in crate::properties::parse(&text) {
                match property_value(&key, &value) {
                    Some(true) => { inventory.properties.insert(key, value); }
                    Some(false) => warn(&mut inventory.warnings, "server.properties"),
                    None => (),
                }
            },
            Err(_) => warn(&mut inventory.warnings, "server.properties"),
        },
        _ => warn(&mut inventory.warnings, "server.properties"),
    }
    for filename in ["whitelist.json", "ops.json", "banned-players.json", "banned-ips.json"] {
        let entries = match read(root, filename) {
            Ok(None) => Some(0),
            Ok(Some(bytes)) => serde_json::from_slice::<serde_json::Value>(&bytes).ok()
                .and_then(|value| value.as_array().filter(|items| items.iter().all(|item| item.is_object())).map(|items| items.len() as u32)),
            Err(()) => None,
        };
        if entries.is_none() { warn(&mut inventory.warnings, filename); }
        inventory.permissions.push(PermissionFile { filename: filename.into(), entries });
    }
    inventory
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn previews_files_without_disclosing_private_data_or_changing_source() {
        let root = std::env::temp_dir().join(format!("tch-import-preview-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("world")).unwrap();
        fs::create_dir(root.join("fake-world")).unwrap();
        fs::create_dir(root.join(if cfg!(windows) { "Mods" } else { "mods" })).unwrap();
        fs::create_dir(root.join("plugins")).unwrap();
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gzip.write_all(&[10, 0, 0, 10, 0, 4, b'D', b'a', b't', b'a', 0, 0]).unwrap();
        let mut launcher = vec![0; 22];
        launcher[..4].copy_from_slice(b"PK\x05\x06");
        let files: Vec<(&str, Vec<u8>)> = vec![
            ("server.jar", launcher),
            ("world/level.dat", gzip.finish().unwrap()), ("fake-world/level.dat", b"invalid".to_vec()),
            ("mods/z.jar", vec![]), ("mods/a.jar.disabled", vec![]), ("mods/notes.txt", vec![]), ("plugins/p.jar", vec![]),
            ("server.properties", b"server-port=25565\nmax-players=20\nlevel-name=world\nrcon.password=PRIVATE_PASSWORD\nmotd=PRIVATE_MOTD\ngamemode=PRIVATE_INVALID\n".to_vec()),
            ("whitelist.json", br#"[{"name":"PRIVATE_NAME","uuid":"PRIVATE_UUID"}]"#.to_vec()),
            ("ops.json", b"PRIVATE_INVALID_JSON".to_vec()),
            ("banned-ips.json", br#"[{"ip":"PRIVATE_IP"}]"#.to_vec()),
        ];
        for (name, bytes) in &files { fs::write(root.join(name), bytes).unwrap(); }
        let root = root.canonicalize().unwrap();
        let inspection = inspect(&root).unwrap();
        let result = &inspection.inventory;
        assert_eq!(result.worlds, ["world"]);
        assert_eq!(result.mods, ["a.jar.disabled", "z.jar"]);
        assert_eq!(result.plugins, ["p.jar"]);
        assert_eq!(result.properties.len(), 3);
        assert_eq!(result.properties["server-port"], "25565");
        assert_eq!(result.permissions.iter().map(|item| item.entries).collect::<Vec<_>>(), [Some(1), None, Some(0), Some(1)]);
        assert!(result.warnings.iter().any(|warning| warning.starts_with("ops.json:")));
        assert!(result.warnings.iter().any(|warning| warning.starts_with("server.properties:")));
        assert!(!serde_json::to_string(&inspection).unwrap().contains("PRIVATE_"));
        let payload = serde_json::to_value(&inspection).unwrap();
        assert_eq!(payload["jarFiles"], serde_json::json!(["server.jar"]));
        assert!(payload.get("source").is_none());
        for (name, bytes) in files { assert_eq!(fs::read(root.join(name)).unwrap(), bytes); }
        assert_eq!(children(&root, &mut Vec::new(), "test").len(), 9);
        let mut warnings = Vec::new();
        for index in 0..100 { warn(&mut warnings, &format!("{index}.jar")); }
        assert_eq!(warnings.len(), 33);
        fs::remove_file(root.join("fake-world/level.dat")).unwrap();
        let level = fs::read(root.join("world/level.dat")).unwrap();
        for index in 0..33 {
            let folder = root.join(format!("world-{index:02}"));
            fs::create_dir(&folder).unwrap();
            fs::write(folder.join("level.dat"), &level).unwrap();
        }
        let limited = inspect(&root).unwrap().inventory;
        assert_eq!(limited.worlds.len(), 32);
        assert!(limited.warnings.iter().any(|warning| warning.starts_with(".:")));
        fs::remove_dir_all(root).unwrap();
    }
}
