use std::time::{Duration,Instant};
use serde::Serialize;

pub struct Target {pub id:String,pub label:String}
pub enum StopState {Active,Clean,Failed(String)}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Failure {pub server_id:String,pub server_name:String,pub cause:FailureCause}
#[derive(Serialize)]
#[serde(tag="kind",rename_all="snake_case")]
pub enum FailureCause {Stop{message:String},Timeout,Backup{message:String}}
impl Target {
    fn failure(&self,cause:FailureCause)->Failure {Failure{server_id:self.id.clone(),server_name:self.label.clone(),cause}}
}

// Request every graceful stop before waiting. One failed server must not prevent
// other owned processes from stopping or their configured safety copies running.
pub fn stop_all(
    targets:&[Target],
    mut request:impl FnMut(&str)->Result<(),String>,
    mut observe:impl FnMut(&str)->StopState,
    mut backup:impl FnMut(&str)->Result<(),String>,
    timeout:Duration,
)->Vec<Failure>{
    let mut pending=Vec::new();let mut clean=Vec::new();let mut errors=Vec::new();
    for target in targets{
        match request(&target.id){Ok(())=>pending.push(target),Err(message)=>errors.push(target.failure(FailureCause::Stop{message}))}
    }
    let deadline=Instant::now()+timeout;
    while !pending.is_empty(){
        pending.retain(|target|match observe(&target.id){
            StopState::Clean=>{clean.push(*target);false},
            StopState::Failed(message)=>{errors.push(target.failure(FailureCause::Stop{message}));false},
            StopState::Active=>true,
        });
        if pending.is_empty(){break;}
        if Instant::now()>=deadline{
            for target in pending{errors.push(target.failure(FailureCause::Timeout));}
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    for target in clean{if let Err(message)=backup(&target.id){errors.push(target.failure(FailureCause::Backup{message}));}}
    errors
}
