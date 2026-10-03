//! Optional TCP tunnel-provider contract. Bore is available only on explicit request.
//! Providers perform bounded blocking work off the UI thread. A failed connect must
//! clean up any remote resources it created before returning an error. Opaque lease
//! handles and authentication data must never appear in serialized status or logs.
use serde::Serialize;
use std::{collections::HashMap,net::{IpAddr,SocketAddr},sync::{Arc,Mutex}};

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct ProviderInfo {pub id:String,pub name:String}
#[derive(Clone,Debug)]
pub struct Request {pub server_id:String,pub local_address:SocketAddr}
impl Request {
    pub fn validate(&self)->Result<(),Error>{
        uuid::Uuid::parse_str(&self.server_id).map_err(|_|Error::InvalidRequest)?;
        let local=match self.local_address.ip(){IpAddr::V4(ip)=>ip.is_loopback()||ip.is_private(),IpAddr::V6(ip)=>ip.is_loopback()||ip.is_unique_local()||ip.is_unicast_link_local()};
        if !local||self.local_address.port()==0{return Err(Error::InvalidRequest);}
        Ok(())
    }
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct Endpoint {pub host:String,pub port:u16}
impl Endpoint {
    pub fn validate(&self)->Result<(),Error>{
        if self.port==0||self.host.len()>253||self.host.is_empty(){return Err(Error::InvalidEndpoint);}
        if self.host.parse::<IpAddr>().is_ok(){return Ok(());}
        if !self.host.is_ascii()||self.host.split('.').any(|label|label.is_empty()||label.len()>63||label.starts_with('-')||label.ends_with('-')||!label.bytes().all(|byte|byte.is_ascii_alphanumeric()||byte==b'-')){return Err(Error::InvalidEndpoint);}
        Ok(())
    }
}
#[derive(Clone,Debug)]
pub struct Lease {pub handle:String,pub endpoint:Endpoint}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Error {ConfirmationRequired,InvalidRequest,InvalidEndpoint,UnknownProvider,Busy,Unavailable,SignInRequired,Temporary,Stopped,CleanupRequired,StateUnavailable}

pub trait TunnelProvider:Send+Sync {
    fn info(&self)->ProviderInfo;
    fn connect(&self,request:&Request)->Result<Lease,Error>;
    /// Must be idempotent: an already absent lease is a successful disconnection.
    fn disconnect(&self,handle:&str)->Result<(),Error>;
    fn is_active(&self,_handle:&str)->bool{true}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all="snake_case")]
pub enum Phase {Creating,Active,Closing,Attention}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status {pub server_id:String,pub provider_id:String,pub phase:Phase,pub endpoint:Option<Endpoint>}
struct Session {local_address:SocketAddr,generation:String,provider_id:String,provider:Arc<dyn TunnelProvider>,phase:Phase,lease:Option<Lease>,disconnecting:bool}
pub struct Manager {providers:HashMap<String,Arc<dyn TunnelProvider>>,sessions:Mutex<HashMap<String,Session>>}
impl Manager {
    pub fn new(providers:Vec<Arc<dyn TunnelProvider>>)->Result<Self,Error>{
        let mut registered=HashMap::new();
        for provider in providers {
            let info=provider.info();
            if info.id.is_empty()||info.id.len()>32||!info.id.bytes().all(|byte|byte.is_ascii_lowercase()||byte.is_ascii_digit()||byte==b'-')||info.name.is_empty()||info.name.len()>80||info.name.chars().any(char::is_control)||registered.contains_key(&info.id){return Err(Error::InvalidRequest);}
            registered.insert(info.id,provider);
        }
        Ok(Self{providers:registered,sessions:Mutex::new(HashMap::new())})
    }
    pub fn providers(&self)->Vec<ProviderInfo>{let mut result:Vec<_>=self.providers.values().map(|provider|provider.info()).collect();result.sort_by(|a,b|a.id.cmp(&b.id));result}
    pub fn status(&self,server_id:&str)->Result<Option<Status>,Error>{
        {let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;if let Some(session)=sessions.get_mut(server_id){if session.phase==Phase::Active&&session.lease.as_ref().is_some_and(|lease|!session.provider.is_active(&lease.handle)){session.phase=Phase::Attention;}}}
        Ok(self.sessions.lock().map_err(|_|Error::StateUnavailable)?.get(server_id).map(|session|Status{server_id:server_id.into(),provider_id:session.provider_id.clone(),phase:session.phase,endpoint:if session.phase==Phase::Active{session.lease.as_ref().map(|lease|lease.endpoint.clone())}else{None}}))
    }
    pub fn connect(&self,provider_id:&str,request:&Request,confirmed:bool,running:impl Fn()->bool)->Result<Status,Error>{
        if !confirmed{return Err(Error::ConfirmationRequired);}
        request.validate()?;if !running(){return Err(Error::Stopped);}
        let provider=self.providers.get(provider_id).ok_or(Error::UnknownProvider)?.clone();
        {let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;if sessions.contains_key(&request.server_id){return Err(Error::Busy);}sessions.insert(request.server_id.clone(),Session{local_address:request.local_address,generation:uuid::Uuid::new_v4().to_string(),provider_id:provider_id.into(),provider:provider.clone(),phase:Phase::Creating,lease:None,disconnecting:false});}
        let lease=match provider.connect(request){Ok(lease)=>lease,Err(error)=>{self.sessions.lock().map_err(|_|Error::StateUnavailable)?.remove(&request.server_id);return Err(error);}};
        let valid=lease.endpoint.validate().is_ok()&&!lease.handle.is_empty()&&lease.handle.len()<=4096&&!lease.handle.chars().any(char::is_control);
        let still_running=running();
        let close={let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;let session=sessions.get_mut(&request.server_id).ok_or(Error::StateUnavailable)?;let close=!valid||!still_running||session.phase==Phase::Closing;session.lease=Some(lease);session.phase=if close{Phase::Closing}else{Phase::Active};close};
        if close {self.disconnect(&request.server_id)?;return Err(if !valid{Error::InvalidEndpoint}else{Error::Stopped});}
        self.status(&request.server_id)?.ok_or(Error::Stopped)
    }
    pub fn disconnect(&self,server_id:&str)->Result<(),Error>{
        let work={let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;let Some(session)=sessions.get_mut(server_id)else{return Ok(());};if session.disconnecting{return Err(Error::Busy);}session.phase=Phase::Closing;let work=session.lease.as_ref().map(|lease|(session.provider.clone(),lease.handle.clone(),session.generation.clone()));session.disconnecting=work.is_some();work};
        // Creation still owns the session and must close any lease it obtains.
        let Some((provider,handle,generation))=work else{return Err(Error::Busy);};
        let result=provider.disconnect(&handle);
        let mut sessions=self.sessions.lock().map_err(|_|Error::StateUnavailable)?;
        if sessions.get(server_id).is_some_and(|session|session.generation==generation){if result.is_ok(){sessions.remove(server_id);}else if let Some(session)=sessions.get_mut(server_id){session.phase=Phase::Attention;session.disconnecting=false;}}
        result.map_err(|_|Error::CleanupRequired)
    }
    pub fn maintain(&self,valid:impl Fn(&str,SocketAddr)->bool)->Result<(),Error>{
        let ids:Vec<_>=self.sessions.lock().map_err(|_|Error::StateUnavailable)?.iter().filter(|(id,session)|session.phase==Phase::Attention||session.lease.as_ref().is_some_and(|lease|!session.provider.is_active(&lease.handle))||!valid(id,session.local_address)).map(|(id,_)|id.clone()).collect();
        for id in ids{let _=self.disconnect(&id);}Ok(())
    }
    pub fn disconnect_all(&self)->Result<(),Error>{
        let ids:Vec<_>=self.sessions.lock().map_err(|_|Error::StateUnavailable)?.keys().cloned().collect();
        let mut failed=false;for id in ids{if self.disconnect(&id).is_err(){failed=true;}}
        if failed{Err(Error::CleanupRequired)}else{Ok(())}
    }
}

#[cfg(test)] mod tests{
    use super::*;
    use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
    struct Provider {opened:AtomicUsize,closed:AtomicUsize,fail_close:AtomicBool}
    impl Provider {fn new()->Self{Self{opened:AtomicUsize::new(0),closed:AtomicUsize::new(0),fail_close:AtomicBool::new(false)}}}
    impl TunnelProvider for Provider {
        fn info(&self)->ProviderInfo{ProviderInfo{id:"test-provider".into(),name:"Local test provider".into()}}
        fn connect(&self,_:&Request)->Result<Lease,Error>{self.opened.fetch_add(1,Ordering::SeqCst);Ok(Lease{handle:"private-test-handle".into(),endpoint:Endpoint{host:"example.test".into(),port:25565}})}
        fn disconnect(&self,handle:&str)->Result<(),Error>{assert_eq!(handle,"private-test-handle");if self.fail_close.load(Ordering::SeqCst){return Err(Error::Temporary);}self.closed.fetch_add(1,Ordering::SeqCst);Ok(())}
    }
    fn request()->Request{Request{server_id:uuid::Uuid::new_v4().to_string(),local_address:"127.0.0.1:25565".parse().unwrap()}}
    #[test] fn optional_provider_requires_confirmation_and_never_serializes_lease_handles(){
        let empty=Manager::new(vec![]).unwrap();assert!(empty.providers().is_empty());assert!(empty.disconnect_all().is_ok());
        let provider=Arc::new(Provider::new());let manager=Manager::new(vec![provider.clone()]).unwrap();let request=request();
        assert_eq!(manager.connect("test-provider",&request,false,||true).unwrap_err(),Error::ConfirmationRequired);assert_eq!(provider.opened.load(Ordering::SeqCst),0);
        let status=manager.connect("test-provider",&request,true,||true).unwrap();assert_eq!(status.phase,Phase::Active);let json=serde_json::to_string(&status).unwrap();assert!(!json.contains("private-test-handle"));
        assert_eq!(manager.connect("test-provider",&request,true,||true).unwrap_err(),Error::Busy);
        manager.disconnect_all().unwrap();assert!(manager.status(&request.server_id).unwrap().is_none());assert_eq!(provider.closed.load(Ordering::SeqCst),1);manager.disconnect_all().unwrap();
    }
    #[test] fn stopped_during_creation_closes_owned_lease_and_failed_cleanup_can_be_retried(){
        let provider=Arc::new(Provider::new());let manager=Manager::new(vec![provider.clone()]).unwrap();let request=request();
        assert_eq!(manager.connect("test-provider",&request,true,||provider.opened.load(Ordering::SeqCst)==0).unwrap_err(),Error::Stopped);assert_eq!(provider.closed.load(Ordering::SeqCst),1);assert!(manager.status(&request.server_id).unwrap().is_none());
        manager.connect("test-provider",&request,true,||true).unwrap();provider.fail_close.store(true,Ordering::SeqCst);assert_eq!(manager.disconnect(&request.server_id),Err(Error::CleanupRequired));assert_eq!(manager.status(&request.server_id).unwrap().unwrap().phase,Phase::Attention);assert_eq!(manager.connect("test-provider",&request,true,||true).unwrap_err(),Error::Busy);
        provider.fail_close.store(false,Ordering::SeqCst);manager.disconnect(&request.server_id).unwrap();assert!(manager.status(&request.server_id).unwrap().is_none());
    }
    #[test] fn validates_targets_provider_identity_and_public_endpoint_shape(){
        let mut request=request();request.local_address="192.0.2.1:25565".parse().unwrap();assert_eq!(request.validate(),Err(Error::InvalidRequest));request.local_address="127.0.0.1:0".parse().unwrap();assert_eq!(request.validate(),Err(Error::InvalidRequest));
        for host in ["https://example.test","user@example.test","bad..test","-bad.test","a/b"]{assert_eq!(Endpoint{host:host.into(),port:25565}.validate(),Err(Error::InvalidEndpoint));}
        for host in ["example.test","203.0.113.1","2001:db8::1"]{Endpoint{host:host.into(),port:25565}.validate().unwrap();}
        let provider=Arc::new(Provider::new());assert!(Manager::new(vec![provider.clone(),provider]).is_err());
    }
    #[test] fn overlapping_cleanup_keeps_ownership_until_the_provider_finishes(){
        use std::{sync::mpsc,time::Duration};
        struct GateProvider {base:Provider,started:mpsc::Sender<()>,release:Mutex<mpsc::Receiver<()>>}
        impl TunnelProvider for GateProvider{
            fn info(&self)->ProviderInfo{self.base.info()}
            fn connect(&self,request:&Request)->Result<Lease,Error>{self.base.connect(request)}
            fn disconnect(&self,handle:&str)->Result<(),Error>{self.started.send(()).map_err(|_|Error::Temporary)?;self.release.lock().unwrap().recv_timeout(Duration::from_secs(5)).map_err(|_|Error::Temporary)?;self.base.disconnect(handle)}
        }
        let (started,ready)=mpsc::channel();let (release,wait)=mpsc::channel();
        let provider=Arc::new(GateProvider{base:Provider::new(),started,release:Mutex::new(wait)});
        let manager=Arc::new(Manager::new(vec![provider.clone()]).unwrap());let request=request();manager.connect("test-provider",&request,true,||true).unwrap();
        let worker=manager.clone();let id=request.server_id.clone();let cleanup=std::thread::spawn(move||worker.disconnect(&id));
        ready.recv_timeout(Duration::from_secs(2)).unwrap();assert_eq!(manager.disconnect(&request.server_id),Err(Error::Busy));assert_eq!(manager.connect("test-provider",&request,true,||true).unwrap_err(),Error::Busy);assert_eq!(manager.status(&request.server_id).unwrap().unwrap().phase,Phase::Closing);
        release.send(()).unwrap();cleanup.join().unwrap().unwrap();assert!(manager.status(&request.server_id).unwrap().is_none());assert_eq!(provider.base.closed.load(Ordering::SeqCst),1);
    }
    #[test]fn maintenance_closes_changed_targets_and_disconnected_workers(){
        struct LiveProvider{base:Provider,active:AtomicBool}
        impl TunnelProvider for LiveProvider{
            fn info(&self)->ProviderInfo{self.base.info()}
            fn connect(&self,request:&Request)->Result<Lease,Error>{self.active.store(true,Ordering::SeqCst);self.base.connect(request)}
            fn disconnect(&self,handle:&str)->Result<(),Error>{self.active.store(false,Ordering::SeqCst);self.base.disconnect(handle)}
            fn is_active(&self,_:&str)->bool{self.active.load(Ordering::SeqCst)}
        }
        let provider=Arc::new(LiveProvider{base:Provider::new(),active:AtomicBool::new(true)});let manager=Manager::new(vec![provider.clone()]).unwrap();let request=request();
        manager.connect("test-provider",&request,true,||true).unwrap();manager.maintain(|id,address|id==request.server_id&&address==request.local_address).unwrap();assert_eq!(manager.status(&request.server_id).unwrap().unwrap().phase,Phase::Active);
        manager.maintain(|_,_|false).unwrap();assert!(manager.status(&request.server_id).unwrap().is_none());
        manager.connect("test-provider",&request,true,||true).unwrap();provider.active.store(false,Ordering::SeqCst);let status=manager.status(&request.server_id).unwrap().unwrap();assert_eq!(status.phase,Phase::Attention);assert!(status.endpoint.is_none());
        manager.maintain(|_,_|true).unwrap();assert!(manager.status(&request.server_id).unwrap().is_none());assert_eq!(provider.base.closed.load(Ordering::SeqCst),2);
    }
}
