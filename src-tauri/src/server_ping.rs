//! Java Server List Ping, using only the registered server's configured endpoint.
use serde::Serialize;
use std::{io::{Read,Write},net::{SocketAddr,TcpStream},time::{Duration,Instant}};

const MAX_PACKET:usize=131_072;
const TIMEOUT:Duration=Duration::from_secs(3);
#[derive(Serialize,Debug)]
#[serde(rename_all="camelCase")]
pub struct Response{
    pub address:String,
    pub sampled_at:i64,
    pub latency_ms:f64,
    pub version:Option<String>,
    pub online:Option<u32>,
    pub maximum:Option<u32>,
}
fn varint(mut value:u32,out:&mut Vec<u8>){loop{let byte=(value&127)as u8;value>>=7;out.push(byte|if value!=0{128}else{0});if value==0{break;}}}
fn integer(read:&mut impl Read)->Result<u32,String>{
    let mut result=0u32;
    for shift in (0..35).step_by(7){let mut byte=[0];read.read_exact(&mut byte).map_err(io_error)?;if shift==28&&byte[0]&0xf0!=0{return Err("Entero de respuesta Minecraft no válido".into());}result|=u32::from(byte[0]&127)<<shift;if byte[0]&128==0{return Ok(result);}}
    Err("Entero de respuesta Minecraft demasiado largo".into())
}
fn io_error(error:std::io::Error)->String{
    match error.kind(){std::io::ErrorKind::TimedOut|std::io::ErrorKind::WouldBlock=>"Minecraft no respondió a tiempo. Comprueba que haya terminado de iniciar y que enable-status esté activado.".into(),_=>format!("No se pudo consultar Minecraft: {error}")}
}
struct Connection{stream:TcpStream,deadline:Instant}
impl Read for Connection{
    fn read(&mut self,bytes:&mut[u8])->std::io::Result<usize>{
        let remaining=self.deadline.checked_duration_since(Instant::now()).filter(|time|!time.is_zero()).ok_or_else(||std::io::Error::from(std::io::ErrorKind::TimedOut))?;
        self.stream.set_read_timeout(Some(remaining))?;self.stream.read(bytes)
    }
}
impl Connection{
    fn send(&mut self,payload:&[u8])->Result<(),String>{
        let mut packet=Vec::new();varint(payload.len()as u32,&mut packet);packet.extend_from_slice(payload);
        while !packet.is_empty(){
            let remaining=self.deadline.checked_duration_since(Instant::now()).filter(|time|!time.is_zero()).ok_or_else(||io_error(std::io::ErrorKind::TimedOut.into()))?;
            self.stream.set_write_timeout(Some(remaining)).map_err(io_error)?;
            let count=self.stream.write(&packet).map_err(io_error)?;
            if count==0{return Err("Minecraft cerró la conexión".into());}packet.drain(..count);
        }Ok(())
    }
    fn packet(&mut self)->Result<Vec<u8>,String>{
        let length=integer(self)?as usize;
        if length==0||length>MAX_PACKET{return Err("Tamaño de respuesta Minecraft no válido".into());}
        let mut packet=vec![0;length];self.read_exact(&mut packet).map_err(io_error)?;Ok(packet)
    }
}
fn status(packet:&[u8])->Result<(Option<String>,Option<u32>,Option<u32>),String>{
    let mut cursor=std::io::Cursor::new(packet);
    if integer(&mut cursor)?!=0{return Err("Respuesta de estado Minecraft no válida".into());}
    let length=integer(&mut cursor)?as usize;
    let offset=cursor.position()as usize;
    if length==0||length>MAX_PACKET||offset.checked_add(length)!=Some(packet.len()){return Err("Longitud del estado Minecraft no válida".into());}
    let value:serde_json::Value=serde_json::from_slice(&packet[offset..]).map_err(|_|"Minecraft devolvió un estado JSON no válido")?;
    if !value.is_object(){return Err("Minecraft no devolvió un objeto de estado".into());}
    let version=value.get("version").and_then(|v|v.get("name")).and_then(|v|v.as_str()).filter(|v|v.chars().count()<=256).map(str::to_owned);
    let count=|key:&str|value.get("players").and_then(|v|v.get(key)).and_then(|v|v.as_u64()).and_then(|v|u32::try_from(v).ok());
    Ok((version,count("online"),count("max")))
}
pub fn query(address:SocketAddr)->Result<Response,String>{query_with_timeout(address,TIMEOUT)}
fn query_with_timeout(address:SocketAddr,timeout:Duration)->Result<Response,String>{
    let deadline=Instant::now()+timeout;
    let stream=TcpStream::connect_timeout(&address,timeout).map_err(io_error)?;
    stream.set_nodelay(true).map_err(io_error)?;
    let mut connection=Connection{stream,deadline};
    let host=address.ip().to_string();
    let mut handshake=vec![0];varint(u32::MAX,&mut handshake);varint(host.len()as u32,&mut handshake);handshake.extend_from_slice(host.as_bytes());handshake.extend_from_slice(&address.port().to_be_bytes());handshake.push(1);
    connection.send(&handshake)?;connection.send(&[0])?;
    let token=chrono::Utc::now().timestamp_millis().to_be_bytes();
    let mut ping=vec![1];ping.extend_from_slice(&token);
    // Send the ping before waiting: some servers defer status until it arrives.
    let started=Instant::now();connection.send(&ping)?;
    let (version,online,maximum)=status(&connection.packet()?)?;
    let pong=connection.packet()?;
    if pong.len()!=9||pong[0]!=1||pong[1..]!=token{return Err("Minecraft devolvió una respuesta de latencia no válida".into());}
    Ok(Response{address:address.to_string(),sampled_at:chrono::Utc::now().timestamp_millis(),latency_ms:started.elapsed().as_secs_f64()*1000.0,version,online,maximum})
}

#[cfg(test)]mod tests{
    use super::*;
    use std::net::TcpListener;
    fn response(json:&str)->Vec<u8>{let mut payload=vec![0];varint(json.len()as u32,&mut payload);payload.extend_from_slice(json.as_bytes());payload}
    #[test]fn rejects_malformed_status_without_inventing_counts(){
        let json=r#"{"version":{"name":"Paper"},"players":{"online":0,"max":20}}"#;
        assert_eq!(status(&response(json)).unwrap(),(Some("Paper".into()),Some(0),Some(20)));
        assert_eq!(status(&response(r#"{"players":{"online":-1,"max":4294967296}}"#)).unwrap(),(None,None,None));
        for json in ["[]","null","garbage"]{assert!(status(&response(json)).is_err());}
        let mut trailing=response(json);trailing.push(0);assert!(status(&trailing).is_err());
        assert!(integer(&mut &[255u8,255,255,255,127][..]).is_err());
        assert!(integer(&mut &[128u8;5][..]).is_err());
    }
    #[test]fn actual_tcp_exchange_checks_handshake_status_and_matching_pong(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
        let peer=std::thread::spawn(move||{
            let (stream,_)=listener.accept().unwrap();let mut connection=Connection{stream,deadline:Instant::now()+TIMEOUT};
            let handshake=connection.packet().unwrap();let mut cursor=std::io::Cursor::new(handshake);
            assert_eq!(integer(&mut cursor).unwrap(),0);assert_eq!(integer(&mut cursor).unwrap(),u32::MAX);
            let length=integer(&mut cursor).unwrap()as usize;let mut host=vec![0;length];cursor.read_exact(&mut host).unwrap();assert_eq!(host,b"127.0.0.1");
            let mut port=[0;2];cursor.read_exact(&mut port).unwrap();assert_eq!(u16::from_be_bytes(port),address.port());let mut state=[0];cursor.read_exact(&mut state).unwrap();assert_eq!(state,[1]);assert_eq!(cursor.position()as usize,cursor.get_ref().len());
            assert_eq!(connection.packet().unwrap(),[0]);
            let ping=connection.packet().unwrap();assert_eq!(ping.len(),9);assert_eq!(ping[0],1);
            connection.send(&response(r#"{"version":{"name":"Local fixture"},"players":{"online":2,"max":10}}"#)).unwrap();connection.send(&ping).unwrap();
        });
        let value=query(address).unwrap();peer.join().unwrap();assert_eq!(value.online,Some(2));assert_eq!(value.maximum,Some(10));assert_eq!(value.version.as_deref(),Some("Local fixture"));assert!(value.latency_ms.is_finite()&&value.latency_ms>=0.0);
    }
    #[test]fn wrong_pong_oversized_frames_and_silent_peers_fail(){
        for mode in 0..3{
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();
            let peer=std::thread::spawn(move||{
                let (stream,_)=listener.accept().unwrap();let mut connection=Connection{stream,deadline:Instant::now()+TIMEOUT};
                connection.packet().unwrap();connection.packet().unwrap();let mut ping=connection.packet().unwrap();
                if mode==0{connection.send(&response("{}")).unwrap();ping[8]^=1;connection.send(&ping).unwrap();}
                else if mode==1{let mut oversized=Vec::new();varint((MAX_PACKET+1)as u32,&mut oversized);connection.stream.write_all(&oversized).unwrap();}
                else{std::thread::sleep(Duration::from_millis(400));}
            });
            assert!(query_with_timeout(address,Duration::from_millis(200)).is_err());peer.join().unwrap();
        }
    }
}
