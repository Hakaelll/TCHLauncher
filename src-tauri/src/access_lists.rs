use crate::{model::Server, players};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::{Read, Write}, path::Path};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Identity {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selection {
    Names { names: Vec<String> },
    Identities { players: Vec<Identity> },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub file_name: String,
    #[serde(flatten)]
    pub selection: Selection,
}

#[derive(Debug, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Outcome {
    Commands { submitted: usize },
    Saved { added: usize, #[serde(rename = "previousFile")] previous_file: Option<String> },
}

fn regular(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 { return false; }
    }
    metadata.is_file() && !metadata.file_type().is_symlink()
}

fn read_file(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if !regular(&metadata) || metadata.len() > limit {
        return Err(format!("{} debe ser un archivo normal de hasta {} MB", path.display(), limit / 1_000_000));
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let opened = file.metadata().map_err(|error| error.to_string())?;
    if !regular(&opened) || opened.len() > limit { return Err("El archivo ha cambiado o supera el límite permitido".into()); }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit || !regular(&fs::symlink_metadata(path).map_err(|error| error.to_string())?) {
        return Err("El archivo ha cambiado o supera el límite permitido".into());
    }
    Ok(Some(bytes))
}

fn identity_uuid(player: &Identity) -> Result<Uuid, String> {
    if !players::valid_player(&player.name) { return Err(format!("Nombre de Minecraft no válido: {}", player.name)); }
    let uuid = Uuid::parse_str(&player.uuid).map_err(|_| format!("UUID no válido para {}", player.name))?;
    if !uuid.hyphenated().to_string().eq_ignore_ascii_case(&player.uuid) { return Err(format!("El UUID de {} debe usar el formato de Minecraft con guiones", player.name)); }
    Ok(uuid)
}

fn validate_identities(input: Vec<Identity>) -> Result<Vec<Identity>, String> {
    if input.is_empty() || input.len() > 1000 { return Err("La lista debe contener entre 1 y 1000 jugadores".into()); }
    let mut seen = HashMap::new();
    let mut output = Vec::new();
    for mut player in input {
        let uuid = identity_uuid(&player)?;
        let name = player.name.to_ascii_lowercase();
        if let Some(previous) = seen.insert(uuid, name.clone()) {
            if previous != name { return Err(format!("El UUID de {} aparece con nombres distintos", player.name)); }
        } else {
            player.uuid = uuid.hyphenated().to_string();
            output.push(player);
        }
    }
    Ok(output)
}

pub fn read_import(path: &Path) -> Result<Import, String> {
    let bytes = read_file(path, 1_000_000)?.ok_or("El archivo seleccionado ya no existe")?;
    let selection = if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("json")) {
        let entries: Vec<Identity> = serde_json::from_slice(&bytes).map_err(|error| format!("El JSON debe contener jugadores con nombre y UUID: {error}"))?;
        Selection::Identities { players: validate_identities(entries)? }
    } else {
        let text = String::from_utf8(bytes).map_err(|_| "El archivo de texto debe usar UTF-8")?;
        let names = text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).map(str::to_string).collect::<Vec<_>>();
        Selection::Names { names: players::validate_names(&names)? }
    };
    Ok(Import {
        file_name: path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "Lista importada".into()),
        selection,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Removal { pub removed: bool, pub previous_file: Option<String> }

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum List { Whitelist, Operators, BannedPlayers }

impl List {
    fn stem(self) -> &'static str {
        match self { Self::Whitelist => "whitelist", Self::Operators => "ops", Self::BannedPlayers => "banned-players" }
    }
}

/// Caller holds the stopped-server reservation until this operation finishes.
pub fn merge(server: &Server, input: Vec<Identity>) -> Result<Outcome, String> {
    let incoming = validate_identities(input)?;
    let (added, previous_file) = update(server, List::Whitelist, |entries, existing| {
        let mut added = 0;
        for player in incoming {
            let uuid = identity_uuid(&player)?;
            if let std::collections::hash_map::Entry::Vacant(entry) = existing.entry(uuid) {
                entry.insert(player.name.to_ascii_lowercase());
                entries.push(serde_json::to_value(player).map_err(|error| error.to_string())?);
                added += 1;
            }
        }
        Ok(added)
    })?;
    Ok(Outcome::Saved { added, previous_file })
}

/// UUID selection avoids removing a different account with the same name.
pub fn remove(server: &Server, list: List, uuid: &str) -> Result<Removal, String> {
    let selected = Uuid::parse_str(uuid).map_err(|_| "UUID de jugador no válido")?;
    if !selected.hyphenated().to_string().eq_ignore_ascii_case(uuid) { return Err("UUID de jugador no válido".into()); }
    let canonical = selected.hyphenated().to_string();
    let (removed, previous_file) = update(server, list, |entries, _| {
        let before = entries.len();
        entries.retain(|entry| !entry["uuid"].as_str().is_some_and(|uuid| uuid.eq_ignore_ascii_case(&canonical)));
        Ok(before - entries.len())
    })?;
    Ok(Removal { removed: removed > 0, previous_file })
}

pub fn remove_ip(server: &Server, ip: &str) -> Result<Removal, String> {
    let selected=ip.parse::<std::net::IpAddr>().map_err(|_|"Dirección IP no válida")?;
    let (removed,previous_file)=update_file(server,"banned-ips",|entries| {
        for entry in entries.iter() {
            entry["ip"].as_str().and_then(|ip|ip.parse::<std::net::IpAddr>().ok())
                .ok_or("banned-ips.json contiene una dirección IP no válida")?;
        }
        let before=entries.len();
        entries.retain(|entry|entry["ip"].as_str().and_then(|ip|ip.parse::<std::net::IpAddr>().ok())!=Some(selected));
        Ok(before-entries.len())
    })?;
    Ok(Removal{removed:removed>0,previous_file})
}

fn update(server: &Server, list: List, change: impl FnOnce(&mut Vec<serde_json::Value>, &mut HashMap<Uuid, String>) -> Result<usize, String>) -> Result<(usize, Option<String>), String> {
    update_file(server,list.stem(),|entries| {
        let filename=format!("{}.json",list.stem());
        let mut existing = HashMap::new();
        for entry in entries.iter() {
            let player: Identity = serde_json::from_value(entry.clone()).map_err(|error| format!("{filename} contiene una identidad incompleta: {error}"))?;
            let uuid = identity_uuid(&player)?;
            if let Some(name) = existing.insert(uuid, player.name.to_ascii_lowercase()) {
                if name != player.name.to_ascii_lowercase() { return Err(format!("{filename} contiene un UUID con nombres distintos")); }
            }
        }
        change(entries,&mut existing)
    })
}

fn update_file(server: &Server, stem: &str, change: impl FnOnce(&mut Vec<serde_json::Value>) -> Result<usize, String>) -> Result<(usize, Option<String>), String> {
    let directory = Path::new(&server.directory);
    let filename = format!("{stem}.json");
    let path = directory.join(&filename);
    let original = read_file(&path, 5_000_000)?;
    let mut entries: Vec<serde_json::Value> = match original.as_deref() {
        Some(bytes) => serde_json::from_slice(bytes).map_err(|error| format!("{filename} no contiene una lista válida: {error}"))?,
        None => Vec::new(),
    };
    let changed = change(&mut entries)?;
    if changed == 0 { return Ok((0, None)); }
    let replacement = serde_json::to_vec_pretty(&entries).map_err(|error| error.to_string())?;
    if replacement.len() > 5_000_000 { return Err("La lista resultante supera el límite de 5 MB".into()); }
    let token = Uuid::new_v4();
    let temporary = directory.join(format!("{stem}-{token}.part"));
    let previous_file = original.as_ref().map(|_| format!("{stem}-before-{token}.json"));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|error| error.to_string())?;
        file.write_all(&replacement).and_then(|_| file.sync_all()).map_err(|error| error.to_string())?;
        drop(file);
        if let (Some(bytes), Some(name)) = (&original, &previous_file) {
            let mut safety = fs::OpenOptions::new().write(true).create_new(true).open(directory.join(name)).map_err(|error| error.to_string())?;
            safety.write_all(bytes).and_then(|_| safety.sync_all()).map_err(|error| error.to_string())?;
        }
        if read_file(&path, 5_000_000)? != original { return Err(format!("{filename} ha cambiado. Actualiza la lista antes de modificarla.")); }
        fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
        Ok((changed, previous_file))
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_preserves_identities_and_existing_files() {
        let directory = std::env::temp_dir().join(format!("tch-whitelist-{}", Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(directory.clone());
        let server = Server {
            id: "test".into(), name: "Test".into(), version: "1.21".into(), platform: "vanilla".into(),
            directory: directory.to_string_lossy().into_owned(), jar_path: None, java_path: None,
            memory_min_mb: 1024, memory_max_mb: 2048, storage_target_gb: 5,
            created_at: String::new(), state: "stopped".into(), image_data: None, pinned: false,
        };
        let first = "00000000-0000-4000-8000-000000000001";
        let second = "00000000-0000-4000-8000-000000000002";
        let third = "00000000-0000-4000-8000-000000000003";
        let selected = directory.join("selected.json");
        let source = format!(r#"[{{"name":"SameName","uuid":"{first}"}},{{"name":"samename","uuid":"{first}"}},{{"name":"SameName","uuid":"{second}"}}]"#);
        fs::write(&selected, &source).unwrap();
        let picked = read_import(&selected).unwrap();
        assert_eq!(picked.file_name, "selected.json");
        let Selection::Identities { players } = picked.selection else { panic!("JSON lost its identities") };
        assert_eq!(players.len(), 2);
        assert_eq!(players[0].uuid, first);
        assert_eq!(players[1].uuid, second);

        let target = directory.join("whitelist.json");
        let original = format!("[\n {{\"name\":\"OldName\", \"uuid\":\"{first}\", \"custom\":{{\"keep\":true}}}}\n]\n");
        fs::write(&target, &original).unwrap();
        let Outcome::Saved { added, previous_file } = merge(&server, players).unwrap() else { panic!("expected saved result") };
        assert_eq!(added, 1);
        let previous_file = previous_file.unwrap();
        assert!(previous_file.starts_with("whitelist-before-"));
        assert_eq!(fs::read(directory.join(&previous_file)).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&selected).unwrap(), source.as_bytes());
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
        assert_eq!(saved[0], serde_json::from_str::<serde_json::Value>(&original).unwrap()[0]);
        assert_eq!(saved[1]["name"], "SameName");
        assert_eq!(saved[1]["uuid"], second);
        let file_count = fs::read_dir(&directory).unwrap().count();
        let before_noop = fs::read(&target).unwrap();
        assert!(matches!(merge(&server, vec![Identity { name: "ChangedName".into(), uuid: first.into() }]).unwrap(), Outcome::Saved { added: 0, previous_file: None }));
        assert_eq!(fs::read_dir(&directory).unwrap().count(), file_count);
        assert_eq!(fs::read(&target).unwrap(), before_noop);

        fs::write(&target, b"[{ broken JSON").unwrap();
        assert!(merge(&server, vec![Identity { name: "NewPlayer".into(), uuid: third.into() }]).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"[{ broken JSON");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), file_count);
        for invalid in [r#"[{"name":"MissingUuid"}]"#.to_string(), format!(r#"[{{"name":"Alpha","uuid":"{first}"}},{{"name":"Beta","uuid":"{first}"}}]"#), r#"[{"name":"Alpha","uuid":"not-a-uuid"}]"#.to_string()] {
            fs::write(&selected, invalid).unwrap();
            assert!(read_import(&selected).is_err());
        }
        fs::write(&target, r#"[{"name":"OldName","uuid":"broken"}]"#).unwrap();
        let invalid_existing = fs::read(&target).unwrap();
        assert!(merge(&server, vec![Identity { name: "NewPlayer".into(), uuid: third.into() }]).is_err());
        assert_eq!(fs::read(&target).unwrap(), invalid_existing);

        let text = directory.join("players.txt");
        fs::write(&text, "# Players\nAlpha\n alpha \nBeta\n\n").unwrap();
        let Selection::Names { names } = read_import(&text).unwrap().selection else { panic!("expected name list") };
        assert_eq!(names, vec!["Alpha", "Beta"]);
        assert!(read_import(&directory).is_err());
        fs::write(&text, vec![b'a'; 1_000_001]).unwrap();
        assert!(read_import(&text).is_err());
        fs::remove_file(&target).unwrap();
        assert!(matches!(merge(&server, vec![Identity { name: "NewPlayer".into(), uuid: third.into() }]).unwrap(), Outcome::Saved { added: 1, previous_file: None }));
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&fs::read(&target).unwrap()).unwrap()[0]["uuid"], third);
        assert!(!fs::read_dir(&directory).unwrap().any(|entry| entry.unwrap().path().extension().is_some_and(|ext| ext == "part")));
        let same_name = format!(r#"[{{"name":"SameName","uuid":"{first}","keep":{{"permission":true}}}},{{"name":"SameName","uuid":"{second}"}},{{"name":"SameName","uuid":"{second}","duplicate":true}}]"#);
        fs::write(&target, &same_name).unwrap();
        let removed = remove(&server, List::Whitelist, second).unwrap();
        assert!(removed.removed);
        assert_eq!(fs::read(directory.join(removed.previous_file.unwrap())).unwrap(), same_name.as_bytes());
        let remaining: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
        assert_eq!(remaining, vec![serde_json::from_str::<Vec<serde_json::Value>>(&same_name).unwrap().remove(0)]);
        let count = fs::read_dir(&directory).unwrap().count();
        let unchanged = fs::read(&target).unwrap();
        let absent = remove(&server, List::Whitelist, second).unwrap();
        assert!(!absent.removed && absent.previous_file.is_none());
        assert_eq!(fs::read(&target).unwrap(), unchanged);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), count);
        assert!(remove(&server, List::Whitelist, "SameName").is_err());
        assert!(crate::players::command("whitelist_remove", Some("SameName"), None).is_err());
        fs::write(&target, b"[{broken JSON").unwrap();
        assert!(remove(&server, List::Whitelist, first).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"[{broken JSON");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), count);
        fs::remove_file(&target).unwrap();
        let absent = remove(&server, List::Whitelist, first).unwrap();
        assert!(!absent.removed && absent.previous_file.is_none());
        assert!(!target.exists());
        assert!(serde_json::from_str::<List>(r#""../../server.properties""#).is_err());
        for list in [List::Operators, List::BannedPlayers] {
            let path = directory.join(format!("{}.json", list.stem()));
            let original = format!(r#"[
  {{"name":"SameName","uuid":"{first}","level":3,"bypassesPlayerLimit":true,"reason":"keep this reason","expires":"forever","custom":{{"keep":true}}}},
  {{"name":"SameName","uuid":"{second}","level":4,"reason":"selected"}}
]"#);
            fs::write(&path, &original).unwrap();
            fs::write(&target, &same_name).unwrap();
            let removed = remove(&server, list, second).unwrap();
            assert!(removed.removed);
            assert_eq!(fs::read(directory.join(removed.previous_file.unwrap())).unwrap(), original.as_bytes());
            let remaining: Vec<serde_json::Value> = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(remaining, vec![serde_json::from_str::<Vec<serde_json::Value>>(&original).unwrap().remove(0)]);
            assert_eq!(fs::read(&target).unwrap(), same_name.as_bytes());
            let count = fs::read_dir(&directory).unwrap().count();
            let unchanged = fs::read(&path).unwrap();
            let absent = remove(&server, list, second).unwrap();
            assert!(!absent.removed && absent.previous_file.is_none());
            assert_eq!(fs::read(&path).unwrap(), unchanged);
            assert_eq!(fs::read_dir(&directory).unwrap().count(), count);
            fs::write(&path, b"[broken JSON").unwrap();
            assert!(remove(&server, list, first).is_err());
            assert_eq!(fs::read(&path).unwrap(), b"[broken JSON");
            assert_eq!(fs::read_dir(&directory).unwrap().count(), count);
            fs::remove_file(&path).unwrap();
            let absent = remove(&server, list, first).unwrap();
            assert!(!absent.removed && absent.previous_file.is_none());
            assert!(!path.exists());
        }
        let path=directory.join("banned-ips.json");
        let original=br#"[{"ip":"2001:DB8:0:0:0:0:0:1","reason":"selected"},{"ip":"203.0.113.9","created":"keep","extra":7}]"#;
        fs::write(&path,original).unwrap();
        let removed=remove_ip(&server,"2001:db8::1").unwrap();assert!(removed.removed);
        assert_eq!(fs::read(directory.join(removed.previous_file.unwrap())).unwrap(),original);
        let remaining=fs::read(&path).unwrap();
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&remaining).unwrap(),serde_json::json!([{"ip":"203.0.113.9","created":"keep","extra":7}]));
        let count=fs::read_dir(&directory).unwrap().count();
        let absent=remove_ip(&server,"2001:db8::1").unwrap();assert!(!absent.removed&&absent.previous_file.is_none());
        assert!(remove_ip(&server,"../ops.json").is_err());
        assert_eq!(fs::read(&path).unwrap(),remaining);assert_eq!(fs::read_dir(&directory).unwrap().count(),count);
        fs::write(&path,br#"[{"ip":"invalid"}]"#).unwrap();
        assert!(remove_ip(&server,"203.0.113.9").is_err());assert_eq!(fs::read(&path).unwrap(),br#"[{"ip":"invalid"}]"#);
        for legacy in ["deop", "pardon"] { assert!(crate::players::command(legacy, Some("SameName"), None).is_err()); }
    }
}
