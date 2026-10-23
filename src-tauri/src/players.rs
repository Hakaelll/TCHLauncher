use crate::model::Server;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use std::collections::HashSet;
use std::io::Read;

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
    pub authentication_checks: Vec<crate::authentication::Check>,
}

fn read_list<T: DeserializeOwned>(directory: &Path, name: &str) -> Result<Vec<T>, String> {
    let path = directory.join(name);
    let preliminary=match fs::metadata(&path){Ok(metadata)=>metadata,Err(error)if error.kind()==std::io::ErrorKind::NotFound=>return Ok(Vec::new()),Err(error)=>return Err(format!("No se pudo leer {name}: {error}"))};
    if !preliminary.is_file(){return Err(format!("{name} debe ser un archivo normal"));}
    if preliminary.len()>5_000_000{return Err(format!("{name} supera el límite de 5 MB"));}
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("No se pudo leer {name}: {e}")),
    };
    let metadata=file.metadata().map_err(|e|format!("No se pudo leer {name}: {e}"))?;
    if !metadata.is_file(){return Err(format!("{name} debe ser un archivo normal"));}
    if metadata.len()>5_000_000{return Err(format!("{name} supera el límite de 5 MB"));}
    let mut bytes=Vec::new();
    file.take(5_000_001).read_to_end(&mut bytes).map_err(|e|format!("No se pudo leer {name}: {e}"))?;
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
        authentication_checks: crate::authentication::inspect(server),
    })
}

pub(crate) fn valid_player(name: &str) -> bool {
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

pub fn command(action: &str, target: Option<&str>, reason: Option<&str>) -> Result<String, String> {
    if matches!(action, "whitelist_on" | "whitelist_off") {
        return Ok(format!("whitelist {}", if action == "whitelist_on" { "on" } else { "off" }));
    }
    let target = target.ok_or("Introduce un jugador o una dirección IP")?.trim();
    let player_action = matches!(action, "whitelist_add" | "op" | "deop" | "ban" | "pardon" | "kick");
    let ip_action = matches!(action, "ban_ip" | "pardon_ip");
    if player_action && !valid_player(target) { return Err("El nombre debe tener entre 3 y 16 letras, números o guiones bajos".into()); }
    if ip_action && target.parse::<IpAddr>().is_err() { return Err("Introduce una dirección IPv4 o IPv6 válida".into()); }
    if !player_action && !ip_action { return Err("Acción de jugador no válida".into()); }
    let base = match action {
        "whitelist_add" => format!("whitelist add {target}"),
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
    if reason.chars().count() > 120 || reason.chars().any(char::is_control) {
        return Err("El motivo debe tener como máximo 120 caracteres y no contener saltos de línea".into());
    }
    Ok(format!("{base} {reason}"))
}

pub(crate) fn message_player_change(message: &str) -> Option<(&str, bool)> {
    let (name, joined) = if let Some(name) = message.strip_suffix(" joined the game") { (name, true) }
        else if let Some(name) = message.strip_suffix(" left the game") { (name, false) }
        else { return None };
    valid_player(name).then_some((name, joined))
}

pub(crate) fn apply_log_event(online:&mut HashSet<String>,event:&crate::model::LogEvent){
    let Some((name,joined))=message_player_change(&event.message)else{return;};
    if event.level!="player"||event.event.as_deref()!=Some(if joined{"join"}else{"leave"}){return;}
    if joined{online.insert(name.into());}else{online.remove(name);}
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct OperatorEdit {
    pub uuid:String,
    pub level:u8,
    pub bypasses_player_limit:bool,
    pub expected_level:u8,
    pub expected_bypasses_player_limit:bool,
}

/// Caller must own the stopped-server reservation. Only edit an existing UUID;
/// never resolve a name into a different identity or grant OP implicitly.
pub fn save_operator(server:&Server,edit:&OperatorEdit)->Result<String,String>{
    use std::io::{Read,Write};
    let identity=uuid::Uuid::parse_str(&edit.uuid).map_err(|_|"El UUID del operador no es válido")?;
    if edit.level>4{return Err("El nivel de operador debe estar entre 0 y 4".into());}
    let path=Path::new(&server.directory).join("ops.json");
    let read=||->Result<Vec<u8>,String>{
        let meta=fs::symlink_metadata(&path).map_err(|e|format!("No se pudo leer ops.json: {e}"))?;
        if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>5_000_000{return Err("ops.json debe ser un archivo normal de hasta 5 MB".into());}
        let mut bytes=Vec::new();fs::File::open(&path).map_err(|e|e.to_string())?.take(5_000_001).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
        if bytes.len()>5_000_000{return Err("ops.json supera el límite de 5 MB".into());}Ok(bytes)
    };
    let original=read()?;
    let operators:Vec<Operator>=serde_json::from_slice(&original).map_err(|e|format!("ops.json no contiene una lista válida: {e}"))?;
    let matches:Vec<_>=operators.iter().enumerate().filter(|(_,op)|op.uuid.as_deref().and_then(|id|uuid::Uuid::parse_str(id).ok())==Some(identity)).collect();
    if matches.len()!=1{return Err("El operador no existe o su UUID aparece repetido. Actualiza la lista.".into());}
    let (index,operator)=matches[0];
    if operator.level!=edit.expected_level||operator.bypasses_player_limit!=edit.expected_bypasses_player_limit{return Err("Los permisos han cambiado. Actualiza la lista antes de guardar.".into());}
    let mut entries:serde_json::Value=serde_json::from_slice(&original).map_err(|e|e.to_string())?;
    entries[index]["level"]=serde_json::json!(edit.level);
    entries[index]["bypassesPlayerLimit"]=serde_json::json!(edit.bypasses_player_limit);
    let replacement=serde_json::to_vec_pretty(&entries).map_err(|e|e.to_string())?;
    let directory=Path::new(&server.directory);
    let token=uuid::Uuid::new_v4();
    let previous=format!("ops-before-{token}.json");
    let temporary=directory.join(format!("ops-{token}.part"));
    let result=(||{
        let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&temporary).map_err(|e|e.to_string())?;
        file.write_all(&replacement).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;drop(file);
        if read()?!=original{return Err("ops.json ha cambiado. Actualiza la lista antes de guardar.".into());}
        let mut safety=fs::OpenOptions::new().create_new(true).write(true).open(directory.join(&previous)).map_err(|e|e.to_string())?;
        safety.write_all(&original).and_then(|_|safety.sync_all()).map_err(|e|e.to_string())?;drop(safety);
        crate::download::replace_file(&temporary,&path)?;
        Ok(previous)
    })();
    if result.is_err(){let _=fs::remove_file(&temporary);}result
}
