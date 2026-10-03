use crate::model::Server;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerIdentity {
    pub name: String,
    pub uuid: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownPlayer {
    pub name: String,
    pub uuid: Option<String>,
    pub expires_on: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub name: String,
    pub uuid: Option<String>,
    pub level: u8,
    pub bypasses_player_limit: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BannedPlayer {
    pub name: String,
    pub uuid: Option<String>,
    pub created: Option<String>,
    pub source: Option<String>,
    pub expires: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BannedIp {
    pub ip: String,
    pub created: Option<String>,
    pub source: Option<String>,
    pub expires: Option<String>,
    pub reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayersSnapshot {
    pub online: Vec<String>,
    pub known: Vec<KnownPlayer>,
    pub whitelisted: Vec<PlayerIdentity>,
    pub operators: Vec<Operator>,
    pub banned_players: Vec<BannedPlayer>,
    pub banned_ips: Vec<BannedIp>,
    pub whitelist_enabled: bool,
    pub enforce_whitelist: bool,
    pub max_players: u32,
    pub online_mode: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhitelistImport {
    pub file_name: String,
    pub names: Vec<String>,
}

fn read_list<T: DeserializeOwned>(directory: &Path, name: &str) -> Result<Vec<T>, String> {
    let path = directory.join(name);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("No se pudo leer {name}: {e}")),
    };
    if bytes.len() > 5_000_000 { return Err(format!("{name} supera el límite de 5 MB")); }
    serde_json::from_slice(&bytes).map_err(|e| format!("{name} no contiene una lista JSON válida: {e}"))
}

pub fn snapshot(server: &Server, mut online: Vec<String>) -> Result<PlayersSnapshot, String> {
    let directory = Path::new(&server.directory);
    let props = crate::properties::read(&directory.join("server.properties"))?;
    online.sort_unstable_by_key(|name| name.to_ascii_lowercase());
    Ok(PlayersSnapshot {
        online,
        known: read_list(directory, "usercache.json")?,
        whitelisted: read_list(directory, "whitelist.json")?,
        operators: read_list(directory, "ops.json")?,
        banned_players: read_list(directory, "banned-players.json")?,
        banned_ips: read_list(directory, "banned-ips.json")?,
        whitelist_enabled: props.get("white-list").is_some_and(|value| value == "true"),
        enforce_whitelist: props.get("enforce-whitelist").is_some_and(|value| value == "true"),
        max_players: props.get("max-players").and_then(|value| value.parse().ok()).unwrap_or(20),
        online_mode: props.get("online-mode").map_or(true,|value|value=="true"),
    })
}

fn valid_player(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub fn validate_names(names: &[String]) -> Result<Vec<String>, String> {
    if names.is_empty() || names.len() > 1000 { return Err("La lista debe contener entre 1 y 1000 jugadores".into()); }
    let mut unique = HashSet::new();
    let mut output = Vec::new();
    for name in names {
        let name = name.trim();
        if !valid_player(name) { return Err(format!("Nombre de Minecraft no válido: {name}")); }
        if unique.insert(name.to_ascii_lowercase()) { output.push(name.to_string()); }
    }
    Ok(output)
}

pub fn read_whitelist_import(path: &Path) -> Result<WhitelistImport, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("No se pudo abrir la lista: {e}"))?;
    if metadata.len() > 1_000_000 { return Err("El archivo debe ocupar menos de 1 MB".into()); }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() > 1_000_000 { return Err("El archivo debe ocupar menos de 1 MB".into()); }
    let names: Vec<String> = if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json")) {
        let entries: Vec<PlayerIdentity> = serde_json::from_slice(&bytes).map_err(|e| format!("El JSON no contiene una lista blanca válida: {e}"))?;
        entries.into_iter().map(|entry| entry.name).collect()
    } else {
        let text = String::from_utf8(bytes).map_err(|_| "El archivo de texto debe usar UTF-8")?;
        text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).map(str::to_string).collect()
    };
    Ok(WhitelistImport {
        file_name: path.file_name().map(|name| name.to_string_lossy().into()).unwrap_or_else(|| "Lista importada".into()),
        names: validate_names(&names)?,
    })
}

pub fn command(action: &str, target: Option<&str>, reason: Option<&str>) -> Result<String, String> {
    if matches!(action, "whitelist_on" | "whitelist_off") {
        return Ok(format!("whitelist {}", if action == "whitelist_on" { "on" } else { "off" }));
    }
    let target = target.ok_or("Introduce un jugador o una dirección IP")?.trim();
    let player_action = matches!(action, "whitelist_add" | "whitelist_remove" | "op" | "deop" | "ban" | "pardon" | "kick");
    let ip_action = matches!(action, "ban_ip" | "pardon_ip");
    if player_action && !valid_player(target) { return Err("El nombre debe tener entre 3 y 16 letras, números o guiones bajos".into()); }
    if ip_action && target.parse::<IpAddr>().is_err() { return Err("Introduce una dirección IPv4 o IPv6 válida".into()); }
    if !player_action && !ip_action { return Err("Acción de jugador no válida".into()); }
    let base = match action {
        "whitelist_add" => format!("whitelist add {target}"),
        "whitelist_remove" => format!("whitelist remove {target}"),
        "op" => format!("op {target}"),
        "deop" => format!("deop {target}"),
        "ban" => format!("ban {target}"),
        "pardon" => format!("pardon {target}"),
        "kick" => format!("kick {target}"),
        "ban_ip" => format!("ban-ip {target}"),
        "pardon_ip" => format!("pardon-ip {target}"),
        _ => unreachable!(),
    };
    if !matches!(action, "ban" | "ban_ip" | "kick") { return Ok(base); }
    let reason = reason.unwrap_or("").trim();
    if reason.is_empty() { return Ok(base); }
    if reason.len() > 120 || reason.chars().any(char::is_control) {
        return Err("El motivo debe tener como máximo 120 caracteres y no contener saltos de línea".into());
    }
    Ok(format!("{base} {reason}"))
}

pub fn log_player_change(line: &str) -> Option<(&str, bool)> {
    let message = line.rsplit_once("]: ").map(|(_, message)| message).unwrap_or(line);
    let (name, joined) = if let Some(name) = message.strip_suffix(" joined the game") { (name, true) }
        else if let Some(name) = message.strip_suffix(" left the game") { (name, false) }
        else { return None };
    valid_player(name).then_some((name, joined))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn parses_real_server_lists_and_online_state() {
        let directory = std::env::temp_dir().join(format!("tch-players-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("server.properties"), "max-players=12\nwhite-list=true\nenforce-whitelist=true\n").unwrap();
        fs::write(directory.join("usercache.json"), r#"[{"name":"Alex","uuid":"abc","expiresOn":"2026-10-01 00:00:00 +0000"}]"#).unwrap();
        fs::write(directory.join("whitelist.json"), r#"[{"name":"Alex","uuid":"abc"}]"#).unwrap();
        fs::write(directory.join("ops.json"), r#"[{"name":"Alex","uuid":"abc","level":4,"bypassesPlayerLimit":false}]"#).unwrap();
        fs::write(directory.join("banned-players.json"), r#"[{"name":"Steve","uuid":"def","created":"2026-01-01","source":"Console","expires":"forever","reason":"Griefing"}]"#).unwrap();
        let server = Server {pinned:false,id:"id".into(),name:"Test".into(),version:"1.21.4".into(),platform:"vanilla".into(),directory:directory.to_string_lossy().into(),jar_path:None,java_path:None,memory_min_mb:1024,memory_max_mb:2048,storage_target_gb:20,created_at:String::new(),state:"stopped".into(),image_data:None};
        let data = snapshot(&server,vec!["Alex".into()]).unwrap();
        assert_eq!(data.online, vec!["Alex"]);
        assert_eq!(data.known[0].uuid.as_deref(), Some("abc"));
        assert_eq!(data.operators[0].level, 4);
        assert_eq!(data.banned_players[0].reason.as_deref(), Some("Griefing"));
        assert!(data.whitelist_enabled && data.enforce_whitelist);
        assert_eq!(data.max_players, 12);
        assert!(data.online_mode);
        fs::write(directory.join("server.properties"),"online-mode=false\n").unwrap();
        assert!(!snapshot(&server,vec![]).unwrap().online_mode);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn validates_commands_and_log_events() {
        assert_eq!(command("whitelist_add",Some("Alex_1"),None).unwrap(),"whitelist add Alex_1");
        assert_eq!(command("ban_ip",Some("127.0.0.1"),Some("Spam")).unwrap(),"ban-ip 127.0.0.1 Spam");
        assert!(command("op",Some("Alex\nstop"),None).is_err());
        assert!(command("ban_ip",Some("not-ip"),None).is_err());
        assert!(command("ban",Some("Alex"),Some("x\nstop")).is_err());
        assert_eq!(log_player_change("[20:00:00] [Server thread/INFO]: Alex joined the game"),Some(("Alex",true)));
        assert_eq!(log_player_change("[20:01:00] [Server thread/INFO]: Alex left the game"),Some(("Alex",false)));
        assert_eq!(log_player_change("Alex joined the game; stop"),None);
    }

    #[test]
    fn imports_whitelist_text_and_json_without_duplicates() {
        let directory = std::env::temp_dir().join(format!("tch-whitelist-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        let text = directory.join("friends.txt");
        fs::write(&text, "# friends\nAlex\nsteve\nAlex\n").unwrap();
        assert_eq!(read_whitelist_import(&text).unwrap().names,vec!["Alex","steve"]);
        let json = directory.join("whitelist.json");
        fs::write(&json, r#"[{"uuid":"abc","name":"Alex"},{"uuid":"def","name":"Steve"}]"#).unwrap();
        assert_eq!(read_whitelist_import(&json).unwrap().names,vec!["Alex","Steve"]);
        fs::write(&text, "Alex\nstop;op Steve\n").unwrap();
        assert!(read_whitelist_import(&text).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
