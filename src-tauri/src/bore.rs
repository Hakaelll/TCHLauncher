//! Account-free Bore protocol client. Only explicit user requests contact bore.pub.
//! Protocol reference: ekzhang/bore v0.6.0 src/shared.rs and src/client.rs (MIT).
use crate::tunnel::{Endpoint,Error,Lease,ProviderInfo,Request,TunnelProvider};
use serde::Deserialize;
use std::{collections::HashMap,net::SocketAddr,sync::{Arc,Mutex,atomic::{AtomicBool,Ordering},mpsc},time::Duration};
use tokio::{io::{AsyncReadExt,AsyncWriteExt},net::TcpStream,sync::watch,task::JoinSet,time::timeout};

const LIMIT:usize=256;
const WAIT:Duration=Duration::from_secs(3);
#[derive(Deserialize)]
enum Message {Hello(u16),Heartbeat,Connection(String),Challenge(String),Error(String)}
async fn receive(stream:&mut TcpStream,bytes:&mut Vec<u8>)->Result<Message,Error>{
    loop {
        let byte=stream.read_u8().await.map_err(|_|Error::Temporary)?;
        if byte==0{let result=serde_json::from_slice(bytes).map_err(|_|Error::Temporary);bytes.clear();return result;}
        if bytes.len()==LIMIT{return Err(Error::Temporary);}
        bytes.push(byte);
    }
}
async fn send(stream:&mut TcpStream,value:serde_json::Value)->Result<(),Error>{
    let mut bytes=serde_json::to_vec(&value).map_err(|_|Error::Temporary)?;bytes.push(0);
    timeout(WAIT,stream.write_all(&bytes)).await.map_err(|_|Error::Temporary)?.map_err(|_|Error::Temporary)
}
struct Session {stop:watch::Sender<bool>,active:Arc<AtomicBool>,done:mpsc::Receiver<()>}
pub struct Provider {sessions:Mutex<HashMap<String,Session>>}
impl Default for Provider{fn default()->Self{Self{sessions:Mutex::new(HashMap::new())}}}
async fn relay(mut control:TcpStream,remote:SocketAddr,local:SocketAddr,mut stop:watch::Receiver<bool>){
    let mut jobs=JoinSet::new();let mut frame=Vec::new();
    loop {
        tokio::select! {
            _=stop.changed()=>break,
            result=receive(&mut control,&mut frame)=>match result {
                Ok(Message::Heartbeat)=>{},
                Ok(Message::Connection(id))=>{
                    if uuid::Uuid::parse_str(&id).is_err()||jobs.len()>=128 {break;}
                    jobs.spawn(async move {
                        let work=async {
                            let mut public=TcpStream::connect(remote).await.map_err(|_|Error::Temporary)?;
                            send(&mut public,serde_json::json!({"Accept":id})).await?;
                            let private=TcpStream::connect(local).await.map_err(|_|Error::Temporary)?;
                            Ok::<_,Error>((public,private))
                        };
                        if let Ok(Ok((mut public,mut private)))=timeout(WAIT,work).await {
                            let _=tokio::io::copy_bidirectional(&mut public,&mut private).await;
                        }
                    });
                },
                _=>break,
            },
            _=jobs.join_next(),if !jobs.is_empty()=>{},
        }
    }
    jobs.abort_all();while jobs.join_next().await.is_some(){}
}
impl Provider {
    fn connect_to(&self,request:&Request,remote:SocketAddr,host:&str)->Result<Lease,Error>{
        request.validate()?;
        let handle=uuid::Uuid::new_v4().to_string();let local=request.local_address;
        let (stop,wait)=watch::channel(false);let active=Arc::new(AtomicBool::new(false));let worker_active=active.clone();
        let (ready,result)=mpsc::sync_channel(1);let (finished,done)=mpsc::channel();
        std::thread::spawn(move||{
            if let Ok(runtime)=tokio::runtime::Builder::new_current_thread().enable_all().build(){
                runtime.block_on(async {
                    let initial=async {
                        let mut stream=TcpStream::connect(remote).await.map_err(|_|Error::Unavailable)?;
                        send(&mut stream,serde_json::json!({"Hello":0})).await?;
                        match receive(&mut stream,&mut Vec::new()).await? {
                            Message::Hello(port) if port>0=>Ok((stream,port)),
                            Message::Challenge(challenge)=>{let _=challenge;Err(Error::SignInRequired)},
                            Message::Error(message)=>{let _=message;Err(Error::Unavailable)},
                            _=>Err(Error::Temporary),
                        }
                    };
                    match timeout(WAIT,initial).await.unwrap_or(Err(Error::Unavailable)) {
                        Ok((stream,port))=>{
                            worker_active.store(true,Ordering::SeqCst);
                            if ready.send(Ok(port)).is_ok(){relay(stream,remote,local,wait).await;}
                        },
                        Err(error)=>{let _=ready.send(Err(error));},
                    }
                });
            }else{let _=ready.send(Err(Error::Unavailable));}
            worker_active.store(false,Ordering::SeqCst);let _=finished.send(());
        });
        let port=match result.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(port))=>port,
            Ok(Err(error))=>return Err(error),
            Err(_)=>{let _=stop.send(true);return Err(Error::Unavailable);},
        };
        let mut sessions=match self.sessions.lock(){Ok(sessions)=>sessions,Err(_)=>{let _=stop.send(true);return Err(Error::StateUnavailable);}};
        sessions.insert(handle.clone(),Session{stop,active,done});
        Ok(Lease{handle,endpoint:Endpoint{host:host.into(),port}})
    }
}
impl TunnelProvider for Provider {
    fn info(&self)->ProviderInfo{ProviderInfo{id:"bore".into(),name:"Bore · sin cuenta".into()}}
    fn connect(&self,request:&Request)->Result<Lease,Error>{
        // Bound DNS as well as the TCP handshake. Pin data connections to the
        // actual control peer; the remote service never supplies a local target.
        use std::net::ToSocketAddrs;
        let (send,recv)=mpsc::sync_channel(1);
        std::thread::spawn(move||{let _=send.send(("bore.pub",7835).to_socket_addrs().map(|values|values.filter(|address|address.is_ipv4()).take(4).collect::<Vec<_>>()));});
        let addresses=recv.recv_timeout(WAIT).map_err(|_|Error::Unavailable)?.map_err(|_|Error::Unavailable)?;
        let remote=addresses.into_iter().next().ok_or(Error::Unavailable)?;
        self.connect_to(request,remote,"bore.pub")
    }
    fn disconnect(&self,handle:&str)->Result<(),Error>{
        let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;
        let Some(session)=sessions.get_mut(handle)else{return Ok(());};
        let _=session.stop.send(true);
        session.done.recv_timeout(Duration::from_secs(5)).map_err(|_|Error::Temporary)?;
        sessions.remove(handle);Ok(())
    }
    fn is_active(&self,handle:&str)->bool{self.sessions.lock().ok().and_then(|sessions|sessions.get(handle).map(|session|session.active.load(Ordering::SeqCst))).unwrap_or(false)}
}
impl Drop for Provider{fn drop(&mut self){if let Ok(sessions)=self.sessions.get_mut(){for session in sessions.values(){let _=session.stop.send(true);}}}}

#[cfg(test)]mod tests{
    use super::*;
    use std::{io::{Read,Write},net::TcpListener};
    fn frame(stream:&mut std::net::TcpStream)->serde_json::Value{let mut value=vec![];loop{let mut byte=[0];stream.read_exact(&mut byte).unwrap();if byte[0]==0{break;}value.push(byte[0]);}serde_json::from_slice(&value).unwrap()}
    fn closed(stream:&mut std::net::TcpStream){let mut byte=[0];match stream.read(&mut byte){Ok(0)=>{},Err(error) if matches!(error.kind(),std::io::ErrorKind::ConnectionReset|std::io::ErrorKind::ConnectionAborted)=>{},result=>panic!("connection was not closed: {result:?}")}}
    #[test]fn local_relay_transfers_both_directions_and_closes_control_and_players(){
        let remote=TcpListener::bind("127.0.0.1:0").unwrap();let address=remote.local_addr().unwrap();
        let local=TcpListener::bind("127.0.0.1:0").unwrap();let local_address=local.local_addr().unwrap();
        let connection=uuid::Uuid::new_v4().to_string();let expected=connection.clone();let (notify,received)=mpsc::channel();
        let router=std::thread::spawn(move||{
            let (mut control,_)=remote.accept().unwrap();control.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            assert_eq!(frame(&mut control),serde_json::json!({"Hello":0}));
            control.write_all(b"{\"Hello\":30000}\0").unwrap();
            control.write_all(&[serde_json::to_vec(&serde_json::json!({"Connection":connection})).unwrap(),vec![0]].concat()).unwrap();
            let (mut stream,_)=remote.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            assert_eq!(frame(&mut stream),serde_json::json!({"Accept":expected}));
            stream.write_all(b"ping").unwrap();let mut reply=[0;4];stream.read_exact(&mut reply).unwrap();assert_eq!(&reply,b"pong");notify.send(()).unwrap();
            closed(&mut stream);closed(&mut control);
        });
        let minecraft=std::thread::spawn(move||{
            let (mut stream,_)=local.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();let mut request=[0;4];stream.read_exact(&mut request).unwrap();assert_eq!(&request,b"ping");stream.write_all(b"pong").unwrap();closed(&mut stream);
        });
        let provider=Provider::default();let request=Request{server_id:uuid::Uuid::new_v4().to_string(),local_address};
        let lease=provider.connect_to(&request,address,"localhost").unwrap();assert_eq!(lease.endpoint.port,30000);assert!(provider.is_active(&lease.handle));
        received.recv_timeout(Duration::from_secs(5)).unwrap();provider.disconnect(&lease.handle).unwrap();assert!(!provider.is_active(&lease.handle));provider.disconnect(&lease.handle).unwrap();minecraft.join().unwrap();router.join().unwrap();
    }
    #[test]fn rejects_invalid_handshakes_and_observes_remote_disconnect(){
        for reply in [b"{\"Hello\":0}\0".to_vec(),b"{\"Challenge\":\"challenge\"}\0".to_vec(),vec![b'x';258],b"{\"Hello\":30000}\0".to_vec()]{
            let remote=TcpListener::bind("127.0.0.1:0").unwrap();let address=remote.local_addr().unwrap();let accepted=reply==b"{\"Hello\":30000}\0";
            let worker=std::thread::spawn(move||{let(mut stream,_)=remote.accept().unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();let _=frame(&mut stream);let _=stream.write_all(&reply);});
            let provider=Provider::default();let request=Request{server_id:uuid::Uuid::new_v4().to_string(),local_address:"127.0.0.1:25565".parse().unwrap()};
            let result=provider.connect_to(&request,address,"localhost");worker.join().unwrap();
            if accepted{let lease=result.unwrap();let deadline=std::time::Instant::now()+Duration::from_secs(2);while provider.is_active(&lease.handle)&&std::time::Instant::now()<deadline{std::thread::sleep(Duration::from_millis(10));}assert!(!provider.is_active(&lease.handle));provider.disconnect(&lease.handle).unwrap();}else{assert!(result.is_err());}
        }
    }
    #[test]
    #[ignore="Requires the external bore.pub relay; exposes only a temporary test echo listener"]
    fn public_relay_forwards_a_temporary_echo_listener(){
        use std::net::ToSocketAddrs;
        let local=TcpListener::bind("127.0.0.1:0").unwrap();local.set_nonblocking(true).unwrap();let local_address=local.local_addr().unwrap();
        let echo=std::thread::spawn(move||{let deadline=std::time::Instant::now()+Duration::from_secs(15);loop{match local.accept(){Ok((mut stream,_))=>{stream.set_nonblocking(false).unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();let mut request=[0;4];stream.read_exact(&mut request).unwrap();assert_eq!(&request,b"ping");stream.write_all(b"pong").unwrap();break;},Err(error) if error.kind()==std::io::ErrorKind::WouldBlock&&std::time::Instant::now()<deadline=>std::thread::sleep(Duration::from_millis(20)),Err(error)=>panic!("echo listener: {error}")}}});
        let provider=Provider::default();let request=Request{server_id:uuid::Uuid::new_v4().to_string(),local_address};let lease=provider.connect(&request).unwrap();
        let address=(lease.endpoint.host.as_str(),lease.endpoint.port).to_socket_addrs().unwrap().find(|address|address.is_ipv4()).unwrap();
        let mut stream=std::net::TcpStream::connect_timeout(&address,Duration::from_secs(5)).unwrap();stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();stream.write_all(b"ping").unwrap();let mut reply=[0;4];stream.read_exact(&mut reply).unwrap();assert_eq!(&reply,b"pong");provider.disconnect(&lease.handle).unwrap();echo.join().unwrap();
    }
}
