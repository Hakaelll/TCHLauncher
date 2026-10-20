use std::time::{Duration,Instant};

pub struct Target {pub id:String,pub label:String}
pub enum StopState {Active,Clean,Failed(String)}

// Request every graceful stop before waiting. One failed server must not prevent
// other owned processes from stopping or their configured safety copies running.
pub fn stop_all(
    targets:&[Target],
    mut request:impl FnMut(&str)->Result<(),String>,
    mut observe:impl FnMut(&str)->StopState,
    mut backup:impl FnMut(&str)->Result<(),String>,
    timeout:Duration,
)->Result<(),String>{
    let mut pending=Vec::new();let mut clean=Vec::new();let mut errors=Vec::new();
    for target in targets{
        match request(&target.id){Ok(())=>pending.push(target),Err(error)=>errors.push(format!("{}: {error}",target.label))}
    }
    let deadline=Instant::now()+timeout;
    while !pending.is_empty(){
        pending.retain(|target|match observe(&target.id){
            StopState::Clean=>{clean.push(*target);false},
            StopState::Failed(error)=>{errors.push(format!("{}: {error}",target.label));false},
            StopState::Active=>true,
        });
        if pending.is_empty(){break;}
        if Instant::now()>=deadline{
            for target in pending{errors.push(format!("{}: No se detuvo a tiempo. Revisa la consola; la aplicación permanece abierta.",target.label));}
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    for target in clean{if let Err(error)=backup(&target.id){errors.push(format!("{}: Copia tras la parada: {error}",target.label));}}
    if errors.is_empty(){Ok(())}else{Err(errors.join("\n"))}
}
