//! Bounded IPv4 IGD discovery and durable ownership of temporary TCP mappings.
//! A router confirmation is not proof of reachability from the internet.
use crate::storage;
use reqwest::blocking::Client;
use reqwest::Url;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_XML: usize = 262_144;
const LEASE: u32 = 3600;
const SOAP_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";
const SERVICES: [&str; 3] = [
    "urn:schemas-upnp-org:service:WANIPConnection:2",
    "urn:schemas-upnp-org:service:WANIPConnection:1",
    "urn:schemas-upnp-org:service:WANPPPConnection:1",
];
fn now() -> i64 { chrono::Utc::now().timestamp_millis() }
fn xml(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&apos;")
}
fn parse_xml(text: &str) -> Result<roxmltree::Document<'_>, String> {
    if text.len() > MAX_XML { return Err("El router devolvió una respuesta demasiado grande".into()); }
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions {
        allow_dtd: false, nodes_limit: 4096, ..Default::default()
    }).map_err(|_| "La respuesta XML del router no era válida".into())
}
fn bounded_body(response: reqwest::blocking::Response) -> Result<String, String> {
    if response.content_length().is_some_and(|size| size > MAX_XML as u64) {
        return Err("El router devolvió una respuesta demasiado grande".into());
    }
    let mut bytes = Vec::new();
    response.take((MAX_XML + 1) as u64).read_to_end(&mut bytes)
        .map_err(|_| "No se pudo leer la respuesta del router".to_string())?;
    if bytes.len() > MAX_XML { return Err("El router devolvió una respuesta demasiado grande".into()); }
    String::from_utf8(bytes).map_err(|_| "La respuesta del router no usa una codificación compatible".into())
}
fn client() -> Result<Client, String> {
    Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_millis(1500)).timeout(Duration::from_millis(2500))
        .user_agent("TCHLauncher-UPnP").build()
        .map_err(|_| "No se pudo preparar la conexión con el router".into())
}
fn private_url(raw: &str, peer: Ipv4Addr) -> Result<Url, String> {
    if raw.len() > 1024 || !peer.is_private() { return Err("La dirección del router no es una dirección LAN IPv4 válida".into()); }
    let url = Url::parse(raw).map_err(|_| "La dirección UPnP no era válida".to_string())?;
    if url.scheme() != "http" || !url.username().is_empty() || url.password().is_some()
        || url.fragment().is_some() || url.host_str().and_then(|host| host.parse::<Ipv4Addr>().ok()) != Some(peer)
        || url.port_or_known_default() == Some(0) {
        return Err("El router anunció una dirección UPnP fuera de su propia dirección LAN".into());
    }
    Ok(url)
}
fn location(packet: &[u8], peer: Ipv4Addr) -> Option<Url> {
    if packet.len() >= 8192 { return None; }
    let text = std::str::from_utf8(packet).ok()?;
    let mut lines = text.lines();
    if !lines.next()?.trim().eq_ignore_ascii_case("HTTP/1.1 200 OK") { return None; }
    let mut found = None;
    for line in lines {
        if line.trim().is_empty() { break; }
        let Some((name, value)) = line.split_once(':') else { continue; };
        if name.eq_ignore_ascii_case("location") {
            if found.is_some() { return None; }
            found = Some(private_url(value.trim(), peer).ok()?);
        }
    }
    found
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Gateway {
    router_ip: Ipv4Addr,
    local_ip: Ipv4Addr,
    control_url: String,
    service_type: String,
    name: String,
}
impl Gateway {
    fn validate(&self) -> Result<(), String> {
        private_url(&self.control_url, self.router_ip)?;
        if !self.local_ip.is_private() || !SERVICES.contains(&self.service_type.as_str()) || self.name.len() > 256 {
            return Err("Los datos guardados del router no son válidos".into());
        }
        Ok(())
    }
    fn from_description(location: &Url, local_ip: Ipv4Addr, text: &str) -> Result<Self, String> {
        let router_ip = location.host_str().and_then(|host| host.parse().ok()).ok_or("Dirección de router no válida")?;
        private_url(location.as_str(), router_ip)?;
        let document = parse_xml(text)?;
        if !document.descendants().any(|node| node.has_tag_name("deviceType") && matches!(node.text(),
            Some("urn:schemas-upnp-org:device:InternetGatewayDevice:1" | "urn:schemas-upnp-org:device:InternetGatewayDevice:2"))) {
            return Err("El dispositivo anunciado no es un router UPnP compatible".into());
        }
        let base = match document.descendants().find(|node| node.has_tag_name("URLBase")).and_then(|node| node.text()) {
            Some(raw) if !raw.trim().is_empty() => private_url(raw.trim(), router_ip)?,
            _ => location.clone(),
        };
        if base.origin() != location.origin() { return Err("El router anunció una base UPnP de otro origen".into()); }
        for service in SERVICES {
            for node in document.descendants().filter(|node| node.has_tag_name("service")) {
                let child = |name| node.children().find(|child| child.has_tag_name(name)).and_then(|child| child.text());
                if child("serviceType") != Some(service) { continue; }
                let control = base.join(child("controlURL").ok_or("El router no indicó su dirección de control")?.trim())
                    .map_err(|_| "La dirección de control UPnP no era válida")?;
                private_url(control.as_str(), router_ip)?;
                if control.origin() != location.origin() { return Err("El router anunció un control UPnP de otro origen".into()); }
                let name = document.descendants().find(|node| node.has_tag_name("friendlyName")).and_then(|node| node.text())
                    .unwrap_or("Router UPnP").chars().take(64).collect();
                let gateway = Self { router_ip, local_ip, control_url: control.to_string(), service_type: service.into(), name };
                gateway.validate()?;
                return Ok(gateway);
            }
        }
        Err("El router no ofrece un servicio de apertura de puertos compatible".into())
    }
    fn action(&self, action: &str, arguments: &[(&str, String)]) -> Result<BTreeMap<String, String>, RouterError> {
        self.validate().map_err(RouterError::Message)?;
        self.action_transport(action, arguments)
    }
    fn action_transport(&self, action: &str, arguments: &[(&str, String)]) -> Result<BTreeMap<String, String>, RouterError> {
        let inner = arguments.iter().map(|(name, value)| format!("<{name}>{}</{name}>", xml(value))).collect::<String>();
        let body = format!("<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"{SOAP_NS}\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:{action} xmlns:u=\"{}\">{inner}</u:{action}></s:Body></s:Envelope>", self.service_type);
        let response = client().map_err(RouterError::Message)?.post(&self.control_url)
            .header("Content-Type", "text/xml; charset=\"utf-8\"")
            .header("SOAPAction", format!("\"{}#{action}\"", self.service_type))
            .body(body).send().map_err(|_| RouterError::Message("El router no respondió a tiempo. Comprueba la conexión y que UPnP esté habilitado".into()))?;
        let success = response.status().is_success();
        let body = bounded_body(response).map_err(RouterError::Message)?;
        parse_reply(&body, success, &self.service_type, action)
    }
    fn key(&self) -> String { format!("{}|{}", self.router_ip, self.control_url) }
}

#[derive(Debug)]
enum RouterError { Fault(u16), Message(String) }
impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Fault(714) => "No existe una apertura para ese puerto en el router",
            Self::Fault(718) => "Ese puerto externo ya pertenece a otra apertura del router",
            Self::Fault(725) => "Este router solo admite aperturas permanentes; TCHLauncher requiere una concesión temporal",
            Self::Fault(606) => "El router no permite a este equipo configurar UPnP",
            Self::Fault(401) => "El router no admite esta acción UPnP",
            Self::Fault(_) => "El router rechazó la configuración automática. Puedes configurar el puerto TCP manualmente",
            Self::Message(message) => message,
        };
        f.write_str(message)
    }
}
fn parse_reply(text: &str, success: bool, service: &str, action: &str) -> Result<BTreeMap<String, String>, RouterError> {
    let document = parse_xml(text).map_err(RouterError::Message)?;
    let envelope = document.root_element();
    if envelope.tag_name().name() != "Envelope" || envelope.tag_name().namespace() != Some(SOAP_NS) {
        return Err(RouterError::Message("El router no devolvió una respuesta SOAP válida".into()));
    }
    let body = envelope.children().find(|node| node.tag_name().name() == "Body" && node.tag_name().namespace() == Some(SOAP_NS))
        .ok_or_else(|| RouterError::Message("La respuesta SOAP del router está incompleta".into()))?;
    if let Some(fault) = body.children().find(|node| node.tag_name().name() == "Fault") {
        let code = fault.descendants().find(|node| node.has_tag_name("errorCode")).and_then(|node| node.text()).and_then(|code| code.trim().parse().ok());
        return Err(code.map(RouterError::Fault).unwrap_or_else(|| RouterError::Message("El router rechazó la acción sin un motivo compatible".into())));
    }
    if !success { return Err(RouterError::Message("El router rechazó la petición de configuración".into())); }
    let response_name = format!("{action}Response");
    let payload = body.children().find(|node| node.tag_name().name() == response_name && node.tag_name().namespace() == Some(service))
        .ok_or_else(|| RouterError::Message("La respuesta del router no corresponde a la acción solicitada".into()))?;
    let mut values = BTreeMap::new();
    for node in payload.children().filter(|node| node.is_element()) {
        if node.children().any(|child| child.is_element()) || values.insert(node.tag_name().name().into(), node.text().unwrap_or("").trim().into()).is_some() {
            return Err(RouterError::Message("La respuesta del router contiene campos ambiguos".into()));
        }
    }
    Ok(values)
}

#[derive(Clone, Debug)]
struct Entry { internal_ip: Ipv4Addr, internal_port: u16, description: String, enabled: bool, lease_seconds: u32 }
trait Control {
    fn mapping(&self, port: u16) -> Result<Option<Entry>, String>;
    fn add(&self, record: &Record) -> Result<(), String>;
    fn remove(&self, port: u16) -> Result<(), String>;
}
fn port_key(port: u16) -> Vec<(&'static str, String)> {
    vec![("NewRemoteHost", String::new()), ("NewExternalPort", port.to_string()), ("NewProtocol", "TCP".into())]
}
impl Control for Gateway {
    fn mapping(&self, port: u16) -> Result<Option<Entry>, String> {
        let values = match self.action("GetSpecificPortMappingEntry", &port_key(port)) {
            Ok(values) => values,
            Err(RouterError::Fault(714)) => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        let value = |key| values.get(key).ok_or("El router devolvió una apertura incompleta".to_string());
        let entry = Entry {
            internal_ip: value("NewInternalClient")?.parse().map_err(|_| "La apertura tiene una IP local no válida")?,
            internal_port: value("NewInternalPort")?.parse().map_err(|_| "La apertura tiene un puerto local no válido")?,
            enabled: match value("NewEnabled")?.as_str() { "1" | "true" => true, "0" | "false" => false, _ => return Err("La apertura tiene un estado no válido".into()) },
            description: value("NewPortMappingDescription")?.clone(),
            lease_seconds: value("NewLeaseDuration")?.parse().map_err(|_| "La apertura tiene una duración no válida")?,
        };
        Ok(Some(entry))
    }
    fn add(&self, record: &Record) -> Result<(), String> {
        let mut args = port_key(record.port);
        args.extend([("NewInternalPort", record.port.to_string()), ("NewInternalClient", record.gateway.local_ip.to_string()),
            ("NewEnabled", "1".into()), ("NewPortMappingDescription", record.description.clone()), ("NewLeaseDuration", LEASE.to_string())]);
        self.action("AddPortMapping", &args).map(|_| ()).map_err(|error| error.to_string())
    }
    fn remove(&self, port: u16) -> Result<(), String> {
        match self.action("DeletePortMapping", &port_key(port)) {
            Ok(_) | Err(RouterError::Fault(714)) => Ok(()), Err(error) => Err(error.to_string()),
        }
    }
}
impl Gateway {
    fn external_ip(&self) -> Result<Ipv4Addr, String> {
        self.action("GetExternalIPAddress", &[]).map_err(|error| error.to_string())?
            .get("NewExternalIPAddress").ok_or("El router no indicó su IP externa")?
            .parse().map_err(|_| "El router devolvió una IP externa no válida".into())
    }
}
fn discover(local_ip: Ipv4Addr) -> Result<Gateway, String> {
    if !local_ip.is_private() { return Err("No se detectó una dirección LAN IPv4 privada para configurar el router".into()); }
    let socket = UdpSocket::bind((local_ip, 0)).map_err(|_| "No se pudo iniciar la búsqueda del router")?;
    socket.set_multicast_ttl_v4(2).map_err(|_| "No se pudo preparar la búsqueda del router")?;
    for version in [1, 2] {
        let message = format!("M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 1\r\nST: urn:schemas-upnp-org:device:InternetGatewayDevice:{version}\r\n\r\n");
        socket.send_to(message.as_bytes(), (Ipv4Addr::new(239,255,255,250), 1900)).map_err(|_| "No se pudo enviar la búsqueda UPnP")?;
    }
    let deadline = Instant::now() + Duration::from_millis(2500);
    let mut packet = [0u8; 8192];
    let mut urls = Vec::new(); let mut seen = HashSet::new();
    while Instant::now() < deadline && urls.len() < 4 {
        socket.set_read_timeout(Some(deadline.saturating_duration_since(Instant::now()).max(Duration::from_millis(1))))
            .map_err(|_| "No se pudo configurar el tiempo de búsqueda UPnP")?;
        match socket.recv_from(&mut packet) {
            Ok((size, SocketAddr::V4(peer))) => if let Some(url) = location(&packet[..size], *peer.ip()) {
                if seen.insert(url.to_string()) { urls.push(url); }
            },
            Ok(_) => (),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => break,
            Err(_) => return Err("La búsqueda del router se interrumpió".into()),
        }
    }
    for url in urls {
        let Ok(response) = client()?.get(url.clone()).send() else { continue; };
        if !response.status().is_success() { continue; }
        let Ok(body) = bounded_body(response) else { continue; };
        if let Ok(gateway) = Gateway::from_description(&url, local_ip, &body) { return Ok(gateway); }
    }
    Err("No se encontró un router UPnP compatible. Puedes redirigir el puerto TCP manualmente hacia la IP local de este equipo".into())
}

pub fn public_ipv4(ip: Ipv4Addr) -> bool {
    let [a,b,c,_] = ip.octets();
    !(ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast()
        || ip.is_multicast() || ip.is_documentation() || a == 0 || a >= 240
        || a == 100 && (64..=127).contains(&b) || a == 198 && (b == 18 || b == 19)
        || a == 192 && b == 0 && c == 0 || a == 192 && b == 88 && c == 99)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase { Pending, Active, Closing, Attention }
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Record {
    server_id: String, gateway: Gateway, port: u16, description: String,
    phase: Phase, external_ip: Option<Ipv4Addr>, expires_at: Option<i64>,
    verified_at: Option<i64>, renew_at: i64, check_after: i64, last_error: Option<String>,
}
impl Record {
    fn validate(&self) -> Result<(), String> {
        self.gateway.validate()?;
        uuid::Uuid::parse_str(&self.server_id).map_err(|_| "La apertura guardada no identifica un servidor válido")?;
        if self.port == 0 || self.description.len() != 29 || !self.description.starts_with("TCH:")
            || !self.description[4..20].bytes().all(|b| b.is_ascii_hexdigit())
            || self.description[20..21] != *":" || self.description[21..] != self.server_id[..8]
            || self.expires_at.is_some_and(|at| at < 0) || self.last_error.as_ref().is_some_and(|error| error.len() > 4096) {
            return Err("La apertura guardada contiene datos no válidos".into());
        }
        Ok(())
    }
    fn owns(&self, entry: &Entry) -> bool {
        entry.description == self.description && entry.internal_ip == self.gateway.local_ip && entry.internal_port == self.port
    }
}
fn db(root: &Path) -> Result<rusqlite::Connection, String> {
    let db = storage::connect(root)?;
    db.busy_timeout(Duration::from_secs(2)).map_err(|error| error.to_string())?;
    // Deliberately no server FK: deletion must not discard a pending router cleanup.
    db.execute_batch("CREATE TABLE IF NOT EXISTS router_mappings (
        server_id TEXT PRIMARY KEY, router_key TEXT NOT NULL, port INTEGER NOT NULL,
        payload TEXT NOT NULL, UNIQUE(router_key,port));").map_err(|error| error.to_string())?;
    Ok(db)
}
fn records(root: &Path) -> Result<Vec<Record>, String> {
    let db = db(root)?;
    let mut statement = db.prepare("SELECT payload FROM router_mappings ORDER BY server_id").map_err(|error| error.to_string())?;
    let payloads = statement.query_map([], |row| row.get::<_, String>(0)).map_err(|error| error.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>().map_err(|error| error.to_string())?;
    payloads.into_iter().map(|payload| {
        if payload.len() > 8192 { return Err("Una apertura guardada es demasiado grande".into()); }
        let record: Record = serde_json::from_str(&payload).map_err(|_| "No se pudo leer una apertura guardada")?;
        record.validate()?; Ok(record)
    }).collect()
}
fn record(root: &Path, id: &str) -> Result<Option<Record>, String> { Ok(records(root)?.into_iter().find(|record| record.server_id == id)) }
fn insert(root: &Path, record: &Record) -> Result<(), String> {
    record.validate()?;
    db(root)?.execute("INSERT INTO router_mappings(server_id,router_key,port,payload) VALUES(?1,?2,?3,?4)",
        params![record.server_id,record.gateway.key(),record.port,serde_json::to_string(record).map_err(|error| error.to_string())?])
        .map(|_| ()).map_err(|_| "Ya existe una apertura registrada para este servidor o puerto del router".into())
}
fn save(root: &Path, record: &Record) -> Result<(), String> {
    record.validate()?;
    let changed = db(root)?.execute("UPDATE router_mappings SET payload=?2 WHERE server_id=?1 AND router_key=?3 AND port=?4",
        params![record.server_id,serde_json::to_string(record).map_err(|error| error.to_string())?,record.gateway.key(),record.port])
        .map_err(|error| error.to_string())?;
    if changed != 1 { return Err("La apertura registrada cambió durante la operación".into()); }
    Ok(())
}
fn forget(root: &Path, record: &Record) -> Result<(), String> {
    db(root)?.execute("DELETE FROM router_mappings WHERE server_id=?1 AND router_key=?2 AND port=?3",
        params![record.server_id,record.gateway.key(),record.port]).map(|_| ()).map_err(|error| error.to_string())
}
fn cleanup(root: &Path, record: &mut Record, control: &impl Control) -> Result<(), String> {
    record.phase = Phase::Closing; save(root, record)?;
    match control.mapping(record.port)? {
        None => forget(root, record),
        Some(entry) if !record.owns(&entry) => {
            forget(root, record)?;
            Err("El puerto pertenece ahora a otra apertura. TCHLauncher la ha conservado sin modificarla".into())
        }
        Some(_) => {
            control.remove(record.port)?;
            match control.mapping(record.port)? {
                None => forget(root, record),
                Some(entry) if !record.owns(&entry) => {
                    forget(root, record)?;
                    Err("El router sustituyó la apertura por otra distinta. No se ha modificado esa nueva apertura".into())
                }
                Some(_) => Err("El router todavía conserva la apertura; se volverá a intentar retirarla".into()),
            }
        }
    }
}
fn confirm(root: &Path, record: &mut Record, control: &impl Control, time: i64) -> Result<(), String> {
    let entry = control.mapping(record.port)?.ok_or("El router no confirmó la apertura solicitada")?;
    if !record.owns(&entry) { return Err("El router devolvió una apertura diferente; TCHLauncher no la modificará".into()); }
    if !entry.enabled || !(120..=LEASE).contains(&entry.lease_seconds) {
        return Err("El router no confirmó una concesión temporal activa de duración compatible".into());
    }
    record.phase = Phase::Active; record.verified_at = Some(time); record.last_error = None;
    record.expires_at = Some(time + i64::from(entry.lease_seconds) * 1000);
    record.renew_at = time + i64::from(entry.lease_seconds) * 500; record.check_after = time + 30_000;
    save(root, record)
}
fn create(root: &Path, mut record: Record, control: &impl Control, running: impl Fn() -> bool) -> Result<(), String> {
    if !running() { return Err("Inicia el servidor antes de abrir su puerto en el router".into()); }
    if control.mapping(record.port)?.is_some() { return Err("El puerto externo ya tiene una apertura. TCHLauncher no sobrescribe configuraciones existentes".into()); }
    insert(root, &record)?; // Intent is durable before any router mutation.
    let result = (|| {
        if !running() { return Err("El servidor dejó de estar en ejecución antes de abrir el puerto".into()); }
        control.add(&record)?;
        if !running() { return Err("El servidor se detuvo durante la configuración del router".into()); }
        confirm(root, &mut record, control, now())
    })();
    if let Err(error) = result {
        record.phase = Phase::Closing; record.last_error = Some(error.clone()); record.check_after = now() + 30_000;
        let saved = save(root, &record);
        if saved.is_ok() { let _ = cleanup(root, &mut record, control); }
        return Err(error);
    }
    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterInfo { name: String, ip: String, external_ip: Option<String>, public_address: Option<bool>, checked_at: i64 }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingInfo {
    phase: Phase, port: u16, local_address: String, external_address: Option<String>,
    expires_at: Option<i64>, verified_at: Option<i64>, last_error: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { router: Option<RouterInfo>, mapping: Option<MappingInfo>, lease_seconds: u32, remote_reachability: &'static str, discovery_error: Option<String> }
struct Detected { gateway: Gateway, info: RouterInfo, discovered: Instant }
#[derive(Default)]
pub struct Manager { gate: Mutex<()>, detected: Mutex<Option<Detected>>, discovery_error: Mutex<Option<String>> }
impl Manager {
    pub fn status(&self, root: &Path, id: &str) -> Result<Status, String> {
        let record = record(root, id)?;
        let cache = self.detected.lock().map_err(|error| error.to_string())?;
        let router = record.as_ref().map(|record| RouterInfo {
            name: record.gateway.name.clone(), ip: record.gateway.router_ip.to_string(),
            external_ip: record.external_ip.map(|ip| ip.to_string()), public_address: record.external_ip.map(public_ipv4),
            checked_at: record.verified_at.unwrap_or(0),
        }).or_else(|| cache.as_ref().map(|cache| cache.info.clone()));
        let mapping = record.map(|record| MappingInfo {
            phase: record.phase, port: record.port, local_address: format!("{}:{}",record.gateway.local_ip,record.port),
            external_address: record.external_ip.filter(|ip| public_ipv4(*ip)).map(|ip| format!("{ip}:{}",record.port)),
            expires_at: record.expires_at, verified_at: record.verified_at, last_error: record.last_error,
        });
        Ok(Status { router, mapping, lease_seconds: LEASE, remote_reachability: "not_checked",
            discovery_error: self.discovery_error.lock().map_err(|error| error.to_string())?.clone() })
    }
    pub fn detect(&self, root: &Path, id: &str, local_ip: Ipv4Addr) -> Result<Status, String> {
        let _gate = self.gate.lock().map_err(|error| error.to_string())?;
        match discover(local_ip) {
            Ok(gateway) => {
                let external_ip = gateway.external_ip().ok();
                let info = RouterInfo {name:gateway.name.clone(), ip:gateway.router_ip.to_string(),
                    external_ip:external_ip.map(|ip|ip.to_string()),public_address:external_ip.map(public_ipv4),checked_at:now()};
                *self.detected.lock().map_err(|error| error.to_string())? = Some(Detected { gateway, info, discovered:Instant::now() });
                *self.discovery_error.lock().map_err(|error| error.to_string())? = None;
            }
            Err(error) => {
                *self.detected.lock().map_err(|error| error.to_string())? = None;
                *self.discovery_error.lock().map_err(|error| error.to_string())? = Some(error);
            }
        }
        self.status(root,id)
    }
    pub fn open(&self, root: &Path, id: &str, port: u16, local_ip: Ipv4Addr, running: impl Fn() -> bool) -> Result<Status, String> {
        let _gate = self.gate.lock().map_err(|error| error.to_string())?;
        if record(root,id)?.is_some() { return Err("Retira primero la apertura registrada para este servidor".into()); }
        let cache = self.detected.lock().map_err(|error| error.to_string())?;
        let cache = cache.as_ref().filter(|cache| cache.discovered.elapsed() < Duration::from_secs(120) && cache.gateway.local_ip == local_ip)
            .ok_or("Detecta de nuevo el router antes de abrir el puerto")?;
        let gateway = cache.gateway.clone(); let external_ip = cache.info.external_ip.as_ref().and_then(|ip| ip.parse().ok());
        let description = format!("TCH:{}:{}", &uuid::Uuid::new_v4().simple().to_string()[..16], &id.get(..8).ok_or("Servidor no válido")?);
        let record = Record {server_id:id.into(),gateway:gateway.clone(),port,description,phase:Phase::Pending,
            external_ip,expires_at:None,verified_at:None,renew_at:0,check_after:0,last_error:None};
        drop(cache);
        create(root,record,&gateway,running)?;
        self.status(root,id)
    }
    pub fn close(&self, root: &Path, id: &str) -> Result<Status, String> {
        let _gate = self.gate.lock().map_err(|error| error.to_string())?;
        if let Some(mut record) = record(root,id)? {
            let gateway=record.gateway.clone();
            if let Err(error)=cleanup(root,&mut record,&gateway) {
                record.last_error=Some(error.clone());record.check_after=now()+30_000;
                if record_exists(root,id)? {save(root,&record)?;}
                return Err(error);
            }
        }
        self.status(root,id)
    }
    pub fn maintain(&self, root: &Path, running: impl Fn(&str)->bool) -> Result<Vec<String>,String> {
        let Ok(_gate)=self.gate.try_lock() else {return Ok(Vec::new());};
        let time=now();let mut changed=Vec::new();
        for mut record in records(root)? {
            let active=running(&record.server_id);
            if !active && record.phase!=Phase::Closing {record.phase=Phase::Closing;record.check_after=0;save(root,&record)?;}
            if record.check_after>time || record.phase==Phase::Attention && active {continue;}
            let gateway=record.gateway.clone();
            let result=if !active || matches!(record.phase,Phase::Pending|Phase::Closing) {
                cleanup(root,&mut record,&gateway)
            } else {
                (|| {
                    let entry=gateway.mapping(record.port)?.ok_or("La apertura ya no existe en el router. Revisa la conexión antes de volver a abrirla")?;
                    if !record.owns(&entry) {return Err("La apertura del router cambió de propietario. TCHLauncher no la modificará".into());}
                    if !entry.enabled || entry.lease_seconds==0 {return Err("El router dejó de confirmar una apertura temporal activa".into());}
                    if time>=record.renew_at {
                        if !running(&record.server_id) {return cleanup(root,&mut record,&gateway);}
                        gateway.add(&record)?;
                        confirm(root,&mut record,&gateway,now())?;
                        record.external_ip=gateway.external_ip().ok();
                    } else {record.verified_at=Some(time);record.last_error=None;}
                    record.check_after=now()+30_000;save(root,&record)
                })()
            };
            if let Err(error)=result {
                if record_exists(root,&record.server_id)? {
                    if record.phase==Phase::Active {record.phase=Phase::Attention;}
                    record.last_error=Some(error);record.check_after=now()+30_000;save(root,&record)?;
                }
            }
            changed.push(record.server_id);
        }
        Ok(changed)
    }
    pub fn close_all(&self, root:&Path) {
        let Ok(_gate)=self.gate.try_lock() else{return;};
        let deadline=Instant::now()+Duration::from_secs(4);
        let Ok(records)=records(root) else{return;};
        for mut record in records {
            record.phase=Phase::Closing;record.check_after=0;
            if save(root,&record).is_err(){continue;}
            if Instant::now()<deadline {let gateway=record.gateway.clone();let _=cleanup(root,&mut record,&gateway);}
        }
    }
}
fn record_exists(root:&Path,id:&str)->Result<bool,String>{
    db(root)?.query_row("SELECT 1 FROM router_mappings WHERE server_id=?1",[id],|_|Ok(())).optional()
        .map(|row|row.is_some()).map_err(|error|error.to_string())
}
