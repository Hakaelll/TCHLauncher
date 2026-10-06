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

#[cfg(test)] mod tests{
    use super::*;
    fn targets()->Vec<Target>{["a","b","c"].into_iter().map(|id|Target{id:id.into(),label:format!("Server {id}")}).collect()}
    #[test] fn one_failed_request_does_not_skip_other_stops_or_copies(){
        let events=std::cell::RefCell::new(Vec::new());
        let result=stop_all(&targets(),|id|{events.borrow_mut().push(format!("stop:{id}"));if id=="a"{Err("stdin closed".into())}else{Ok(())}},|id|{events.borrow_mut().push(format!("observe:{id}"));StopState::Clean},|id|{events.borrow_mut().push(format!("copy:{id}"));if id=="b"{Err("disk full".into())}else{Ok(())}},Duration::ZERO);
        let error=result.unwrap_err();assert!(error.contains("Server a: stdin closed"));assert!(error.contains("Server b: Copia tras la parada: disk full"));
        assert_eq!(&*events.borrow(),&["stop:a","stop:b","stop:c","observe:b","observe:c","copy:b","copy:c"]);
    }
    #[test] fn one_shared_deadline_keeps_live_and_unclean_servers_out_of_backups(){
        let copies=std::cell::RefCell::new(Vec::new());
        let error=stop_all(&targets(),|_|Ok(()),|id|match id{"a"=>StopState::Active,"b"=>StopState::Failed("crashed".into()),_=>StopState::Clean},|id|{copies.borrow_mut().push(id.to_string());Ok(())},Duration::ZERO).unwrap_err();
        assert!(error.contains("Server a: No se detuvo a tiempo"));assert!(error.contains("Server b: crashed"));assert_eq!(&*copies.borrow(),&["c"]);
    }
    #[test] fn successful_and_empty_batches_have_no_errors(){
        let mut copies=0;assert!(stop_all(&targets(),|_|Ok(()),|_|StopState::Clean,|_|{copies+=1;Ok(())},Duration::ZERO).is_ok());assert_eq!(copies,3);
        assert!(stop_all(&[],|_|panic!("empty request"),|_|panic!("empty observe"),|_|panic!("empty copy"),Duration::ZERO).is_ok());
    }
}
