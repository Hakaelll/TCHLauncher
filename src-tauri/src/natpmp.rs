//! Bounded NAT-PMP IPv4/TCP transport (RFC 6886).
//! A gateway response does not prove internet reachability.
use std::net::{Ipv4Addr,SocketAddr,UdpSocket};
use std::time::{Duration,Instant};

#[derive(Clone,Debug,PartialEq)]
pub struct Address{pub epoch:u32,pub external_ip:Ipv4Addr}
#[derive(Clone,Debug,PartialEq)]
pub struct Mapping{pub epoch:u32,pub internal_port:u16,pub external_port:u16,pub lifetime_seconds:u32}

fn result(packet:&[u8],opcode:u8,length:usize)->Result<(),String>{
    if packet.len()!=length||packet[0]!=0||packet[1]!=128+opcode{return Err("El router devolvió una respuesta NAT-PMP no válida".into());}
    match u16::from_be_bytes([packet[2],packet[3]]){
        0=>Ok(()),
        1=>Err("El router no admite esta versión de NAT-PMP".into()),
        2=>Err("El router no permite a este equipo abrir puertos mediante NAT-PMP".into()),
        3=>Err("El router no pudo establecer una conexión con su red externa".into()),
        4=>Err("El router no tiene puertos disponibles para esta apertura".into()),
        5=>Err("El router no admite la apertura TCP mediante NAT-PMP".into()),
        _=>Err("El router rechazó la solicitud NAT-PMP".into()),
    }
}
fn number(packet:&[u8],offset:usize)->u32{u32::from_be_bytes(packet[offset..offset+4].try_into().unwrap())}
fn parse_address(packet:&[u8])->Result<Address,String>{
    result(packet,0,12)?;
    let external_ip=Ipv4Addr::new(packet[8],packet[9],packet[10],packet[11]);
    if external_ip.is_unspecified()||external_ip.is_multicast()||external_ip.is_broadcast(){return Err("El router no indicó una dirección externa válida".into());}
    Ok(Address{epoch:number(packet,4),external_ip})
}
fn request(internal:u16,external:u16,lifetime:u32)->Result<[u8;12],String>{
    if internal==0||lifetime>7200{return Err("La solicitud de apertura temporal no es válida".into());}
    // An internal port of zero would delete every TCP mapping owned by this IP.
    let mut packet=[0u8;12];packet[1]=2;packet[4..6].copy_from_slice(&internal.to_be_bytes());
    packet[6..8].copy_from_slice(&(if lifetime==0{0}else{external}).to_be_bytes());
    packet[8..12].copy_from_slice(&lifetime.to_be_bytes());Ok(packet)
}
fn parse_mapping(packet:&[u8],internal:u16,deleting:bool)->Result<Mapping,String>{
    result(packet,2,16)?;
    let mapping=Mapping{epoch:number(packet,4),internal_port:u16::from_be_bytes([packet[8],packet[9]]),external_port:u16::from_be_bytes([packet[10],packet[11]]),lifetime_seconds:number(packet,12)};
    if mapping.internal_port!=internal||if deleting{mapping.external_port!=0||mapping.lifetime_seconds!=0}else{mapping.external_port==0||mapping.lifetime_seconds==0||mapping.lifetime_seconds>7200}{return Err("El router confirmó una apertura distinta o no temporal".into());}
    Ok(mapping)
}

pub struct Client{local:Ipv4Addr,gateway:SocketAddr}

pub fn default_gateway(local:Ipv4Addr)->Result<Ipv4Addr,String>{
    if !local.is_private(){return Err("No se detectó una dirección LAN IPv4 compatible".into());}
    #[cfg(windows)]
    let candidate={
        use std::io::Read;
        use std::process::{Command,Stdio};
        use std::os::windows::process::CommandExt;
        // The interpolated value is an already parsed IPv4, never arbitrary shell text.
        let script=format!("$ErrorActionPreference='Stop'; $i=Get-NetIPAddress -AddressFamily IPv4 | Where-Object IPAddress -EQ '{local}' | Select-Object -First 1 -ExpandProperty InterfaceIndex; Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' -InterfaceIndex $i | Sort-Object RouteMetric | Select-Object -First 1 -ExpandProperty NextHop");
        let mut child=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command",&script]).creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|"No se pudo consultar la puerta de enlace del equipo")?;
        let stdout=child.stdout.take().ok_or("No se pudo leer la puerta de enlace")?;
        let reader=std::thread::spawn(move||{let mut bytes=Vec::new();stdout.take(16385).read_to_end(&mut bytes).map(|_|bytes)});
        let deadline=Instant::now()+Duration::from_secs(5);
        loop{
            match child.try_wait(){
                Ok(Some(status)) if status.success()=>break,
                Ok(Some(_))=>{let _=reader.join();return Err("No se pudo detectar el router de esta interfaz. Indica su IPv4 manualmente".into());},
                Ok(None) if Instant::now()<deadline=>std::thread::sleep(Duration::from_millis(20)),
                _=>{let _=child.kill();let _=child.wait();let _=reader.join();return Err("La detección del router no respondió a tiempo. Indica su IPv4 manualmente".into());},
            }
        }
        let bytes=reader.join().map_err(|_|"No se pudo leer la puerta de enlace")?.map_err(|_|"No se pudo leer la puerta de enlace")?;
        if bytes.len()>16384{return Err("La respuesta de red era demasiado grande".into());}
        String::from_utf8(bytes).map_err(|_|"La puerta de enlace no tiene un formato válido")?.trim().parse::<Ipv4Addr>().map_err(|_|"No se encontró una puerta de enlace IPv4")?
    };
    #[cfg(target_os="linux")]
    let candidate={
        let routes=std::fs::read_to_string("/proc/net/route").map_err(|_|"No se pudo leer la puerta de enlace")?;
        routes.lines().skip(1).filter_map(|line|{let fields:Vec<_>=line.split_whitespace().collect();if fields.len()<8||fields[1]!="00000000"{return None;}let flags=u16::from_str_radix(fields[3],16).ok()?;if flags&3!=3{return None;}let gateway=u32::from_str_radix(fields[2],16).ok()?;Some((fields[6].parse::<u32>().ok()?,Ipv4Addr::from(gateway.to_le_bytes()))) }).min_by_key(|(metric,_)|*metric).map(|(_,ip)|ip).ok_or("No se encontró una puerta de enlace IPv4")?
    };
    #[cfg(not(any(windows,target_os="linux")))]
    let candidate:Ipv4Addr=return Err("Indica la IPv4 del router manualmente en esta plataforma".into());
    Client::new(local,candidate)?;
    let socket=UdpSocket::bind((Ipv4Addr::UNSPECIFIED,0)).map_err(|_|"No se pudo comprobar la interfaz del router")?;
    socket.connect((candidate,5351)).map_err(|_|"No se pudo comprobar la interfaz del router")?;
    if socket.local_addr().map_err(|_|"No se pudo comprobar la interfaz del router")?.ip()!=std::net::IpAddr::V4(local){return Err("El router detectado pertenece a otra interfaz. Indica la IPv4 correspondiente a este servidor".into());}
    Ok(candidate)
}
impl Client{
    pub fn new(local:Ipv4Addr,gateway:Ipv4Addr)->Result<Self,String>{
        if !local.is_private()||!gateway.is_private()||local==gateway{return Err("NAT-PMP necesita la dirección IPv4 local y la del router de esta red".into());}
        Ok(Self{local,gateway:SocketAddr::from((gateway,5351))})
    }
    fn exchange<T>(&self,request:&[u8],parse:impl Fn(&[u8])->Result<T,String>)->Result<T,String>{
        let socket=UdpSocket::bind((self.local,0)).map_err(|_|"No se pudo abrir la conexión local con el router")?;
        socket.connect(self.gateway).map_err(|_|"No se pudo contactar con el router NAT-PMP")?;
        let mut packet=[0u8;64];
        for milliseconds in [250,500,1000]{
            socket.send(request).map_err(|_|"No se pudo enviar la solicitud NAT-PMP")?;
            let deadline=Instant::now()+Duration::from_millis(milliseconds);
            loop{
                let remaining=deadline.saturating_duration_since(Instant::now());if remaining.is_zero(){break;}
                socket.set_read_timeout(Some(remaining)).map_err(|_|"No se pudo limitar la espera del router")?;
                match socket.recv(&mut packet){
                    Ok(length)=>{
                        // Ignore unrelated protocol messages; connected UDP rejects other peers.
                        if length<2||packet[1]!=request[1]+128{continue;}
                        return parse(&packet[..length]);
                    },
                    Err(error) if matches!(error.kind(),std::io::ErrorKind::TimedOut|std::io::ErrorKind::WouldBlock)=>break,
                    Err(_)=>return Err("El router no respondió a la solicitud NAT-PMP".into()),
                }
            }
        }
        Err("No se encontró soporte NAT-PMP en este router. Para jugar desde internet puedes configurar una apertura manual".into())
    }
    pub fn address(&self)->Result<Address,String>{self.exchange(&[0,0],parse_address)}
    pub fn open(&self,internal:u16,suggested_external:u16,lifetime:u32)->Result<Mapping,String>{
        if lifetime==0{return Err("La apertura necesita una duración temporal mayor que cero".into());}
        let packet=request(internal,suggested_external,lifetime)?;
        self.exchange(&packet,|reply|parse_mapping(reply,internal,false))
    }
    pub fn close(&self,internal:u16)->Result<(),String>{
        self.exchange(&request(internal,0,0)?,|reply|parse_mapping(reply,internal,true)).map(|_|())
    }
}
