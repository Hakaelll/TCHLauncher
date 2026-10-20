use crate::model::Server;
use crate::properties;
use serde::Serialize;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortState {
    Listening,
    Starting,
    NotResponding,
    Available,
    Occupied,
    Unknown,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    pub local_ip: Option<String>,
    pub configured_ip: Option<String>,
    pub port: u16,
    pub lan_address: Option<String>,
    pub local_address: String,
    pub port_state: PortState,
}

fn route_ipv4() -> Option<Ipv4Addr> {
    // UDP connect selects an interface but sends no packet. The documentation-only
    // destination avoids depending on an external service for LAN discovery.
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if !ip.is_unspecified() && !ip.is_loopback() => Some(ip),
        _ => None,
    }
}

fn address(ip: IpAddr, port: u16) -> String { SocketAddr::new(ip, port).to_string() }

fn local_target(binding:Option<IpAddr>)->IpAddr{
    match binding{Some(ip)if !ip.is_unspecified()=>ip,Some(IpAddr::V6(_))=>IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),_=>IpAddr::V4(Ipv4Addr::LOCALHOST)}
}
fn configured_port(values:&std::collections::BTreeMap<String,String>)->Result<u16,String>{
    values.get("server-port").map(String::as_str).unwrap_or("25565").parse::<u16>().ok().filter(|port|*port>0).ok_or_else(||"El puerto de server.properties no es válido".into())
}

fn probe_port(state: &str, port: u16, binding: Option<IpAddr>) -> PortState {
    if state == "running" || state == "starting" || state == "stopping" || state == "installing" {
        let target = local_target(binding);
        if TcpStream::connect_timeout(&SocketAddr::new(target, port), Duration::from_millis(350)).is_ok() {
            PortState::Listening
        } else if state == "starting" || state == "installing" {
            PortState::Starting
        } else { PortState::NotResponding }
    } else {
        let target = binding.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        match TcpListener::bind(SocketAddr::new(target, port)) {
            Ok(listener) => { drop(listener); PortState::Available }
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => PortState::Occupied,
            Err(_) => PortState::Unknown,
        }
    }
}

pub fn inspect(server: &Server, state: &str) -> Result<NetworkSnapshot, String> {
    let mut snapshot=configuration(server)?;
    let binding=snapshot.configured_ip.as_deref().and_then(|ip|ip.parse().ok());
    if snapshot.configured_ip.is_none()||binding.is_some(){snapshot.port_state=probe_port(state,snapshot.port,binding);}
    Ok(snapshot)
}

// Mapping maintenance needs routing/configuration, not a new Minecraft connection.
pub fn configured_lan_target(server:&Server)->Result<(Ipv4Addr,u16),String>{
    let snapshot=configuration(server)?;
    if snapshot.lan_address.is_none(){return Err("El servidor debe responder en una dirección LAN antes de abrir su puerto".into());}
    let ip=snapshot.local_ip.and_then(|ip|ip.parse::<Ipv4Addr>().ok()).filter(|ip|ip.is_private())
        .ok_or("No se detectó una dirección LAN IPv4 privada compatible")?;
    Ok((ip,snapshot.port))
}

fn configuration(server:&Server)->Result<NetworkSnapshot,String>{
    let path = Path::new(&server.directory).join("server.properties");
    let values = properties::read(&path)?;
    let port = configured_port(&values)?;
    let configured_ip = values.get("server-ip").map(|value| value.trim()).filter(|value| !value.is_empty()).map(str::to_string);
    let binding = configured_ip.as_deref().and_then(|value| value.parse::<IpAddr>().ok());
    let route = route_ipv4().map(IpAddr::V4);
    let local_ip = binding.filter(|ip| !ip.is_unspecified() && !ip.is_loopback()).or(route);
    let lan_address = if configured_ip.is_some() && binding.is_none() || binding.is_some_and(|ip| ip.is_loopback()) {
        None
    } else { local_ip.map(|ip| address(ip, port)) };
    let local_address = match (configured_ip.as_deref(), binding) {
        (_, Some(ip)) if !ip.is_unspecified() => address(ip, port),
        (Some(host), None) => format!("{host}:{port}"),
        _ => address(local_target(binding), port),
    };
    let port_state = PortState::Unknown;
    Ok(NetworkSnapshot {
        local_ip: local_ip.map(|ip| ip.to_string()),
        configured_ip,
        port,
        lan_address,
        local_address,
        port_state,
    })
}

pub fn public_ip() -> Result<String, String> {
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5))
        .build().map_err(|_| "No se pudo preparar la consulta de IP pública".to_string())?;
    let response = client.get("https://api.ipify.org").send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|_| "No se pudo consultar la IP pública. Comprueba tu conexión a internet".to_string())?;
    let mut text = String::new();
    response.take(128).read_to_string(&mut text)
        .map_err(|_| "La respuesta de IP pública no se pudo leer".to_string())?;
    let ip = text.trim().parse::<IpAddr>()
        .map_err(|_| "La respuesta de IP pública no era válida".to_string())?;
    Ok(ip.to_string())
}

pub fn check_response(server:&Server,state:&str)->Result<crate::server_ping::Response,String>{
    if !matches!(state,"running"|"starting"){return Err("Inicia el servidor antes de comprobar su respuesta Minecraft".into());}
    let values=properties::read(&Path::new(&server.directory).join("server.properties"))?;
    if values.get("enable-status").is_some_and(|value|value=="false"){return Err("La consulta Minecraft está desactivada (enable-status=false). Puedes activarla en Configuración y reiniciar el servidor.".into());}
    let binding=values.get("server-ip").map(|value|value.trim()).filter(|value|!value.is_empty()).map(|value|value.parse::<IpAddr>()).transpose().map_err(|_|"La dirección configurada no es una IP; no se pudo comprobar Minecraft localmente")?;
    let address=SocketAddr::new(local_target(binding),configured_port(&values)?);
    crate::server_ping::query(address)
}
