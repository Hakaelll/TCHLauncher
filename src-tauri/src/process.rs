use crate::log;
use crate::model::StateEvent;
use crate::model::Server;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub struct ProcessControl {
    stop_requested: std::sync::atomic::AtomicBool,
    forced: std::sync::atomic::AtomicBool,
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    started: Instant,
    startup_phase:Mutex<crate::startup::Phase>,
    running_since: Mutex<Option<Instant>>,
    state: Mutex<String>,
    online: Mutex<HashSet<String>>,
    paper: bool,
    ticks: Mutex<crate::ticks::Collector>,
    output:Mutex<crate::crash_diagnostics::Output>,
    session:String,
}

impl ProcessControl {
    fn new(child:Child,stdin:ChildStdin,paper:bool,started:Instant)->Self{
        Self{forced:std::sync::atomic::AtomicBool::new(false),stop_requested:std::sync::atomic::AtomicBool::new(false),child:Mutex::new(child),stdin:Mutex::new(stdin),started,startup_phase:Mutex::new(crate::startup::Phase::default()),running_since:Mutex::new(None),state:Mutex::new("starting".into()),online:Mutex::new(HashSet::new()),paper,ticks:Mutex::new(crate::ticks::Collector::default()),output:Mutex::new(crate::crash_diagnostics::Output::default()),session:uuid::Uuid::new_v4().to_string()}
    }
}

#[derive(Clone, Default)]
pub struct ProcessManager {
    pub processes: Arc<Mutex<HashMap<String, Arc<ProcessControl>>>>,
    terminal_states: Arc<Mutex<HashMap<String,String>>>,
    exits: Arc<Mutex<HashMap<String,Exit>>>,
}

#[derive(Clone)]
pub struct Exit {pub token:String,pub unexpected:bool,pub successful:bool,pub was_running:bool}

#[derive(serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct ForceStopPreview {pub session_id:String,pub pid:u32}

fn validate_force_stop(state:&str,session:&str,expected:&str)->Result<(),String>{
    if state!="stopping" {return Err("Solicita primero la parada normal del servidor".into());}
    if session!=expected {return Err("El proceso ha cambiado. Vuelve a revisar su estado".into());}
    Ok(())
}

fn terminate(control:&ProcessControl,expected:&str)->Result<(),String>{
    let state=control.state.lock().map_err(|e|e.to_string())?;
    validate_force_stop(&state,&control.session,expected)?;
    let mut child=control.child.lock().map_err(|e|e.to_string())?;
    if child.try_wait().map_err(|e|e.to_string())?.is_some(){return Err("El proceso ya ha terminado".into());}
    child.kill().map_err(|e|format!("No se pudo terminar el proceso: {e}"))?;
    control.forced.store(true,std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

pub fn emit_state(app: &AppHandle, id: &str, state: &str) {
    use tauri::Manager;
    let diagnostic=match state {
        "starting"=>Some(crate::app_log::Event::ServerStarting),"running"=>Some(crate::app_log::Event::ServerRunning),
        "stopping"=>Some(crate::app_log::Event::ServerStopping),"stopped"=>Some(crate::app_log::Event::ServerStopped),
        "crashed"=>Some(crate::app_log::Event::ServerCrashed),_=>None,
    };
    if let (Some(event),Ok(root))=(diagnostic,app.path().app_data_dir()) {crate::app_log::record(&root,event,Some(id));}
    let _ = app.emit("server-state", StateEvent { server_id: id.into(), state: state.into() });
}

const MAX_OUTPUT_LINE_BYTES:usize=64*1024;
fn read_output<R:std::io::Read+Send+'static>(reader:R,mut consume:impl FnMut(String)+Send+'static)->std::thread::JoinHandle<()>{
    std::thread::spawn(move||{
        let mut reader=BufReader::new(reader);
        let mut line=Vec::new();let mut truncated=false;
        loop{
            let bytes=match reader.fill_buf(){Ok(bytes)=>bytes,Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,Err(_)=>break};
            if bytes.is_empty(){
                if !line.is_empty()||truncated{consume(output_line(&line,truncated,false));}
                break;
            }
            let newline=bytes.iter().position(|byte|*byte==b'\n');
            let length=newline.unwrap_or(bytes.len());
            let retained=length.min(MAX_OUTPUT_LINE_BYTES-line.len());
            line.extend_from_slice(&bytes[..retained]);truncated|=retained<length;
            reader.consume(length+usize::from(newline.is_some()));
            if newline.is_some(){consume(output_line(&line,truncated,true));line.clear();truncated=false;}
        }
    })
}
fn output_line(bytes:&[u8],truncated:bool,terminated:bool)->String{
    let bytes=if !truncated&&terminated{bytes.strip_suffix(b"\r").unwrap_or(bytes)}else{bytes};
    let mut line=String::from_utf8_lossy(bytes).into_owned();
    if truncated{line.push_str(" [output line truncated at 64 KiB]");}
    line
}

fn pump<R: std::io::Read + Send + 'static>(reader: R, app: AppHandle, id: String, log_file: Arc<Mutex<File>>, control: Arc<ProcessControl>,stdout:bool) ->std::thread::JoinHandle<()> {
    read_output(reader,move |line| {
            if let Ok(mut output)=control.output.lock(){output.push(&line);}
            if stdout&&control.paper{if let Ok(mut ticks)=control.ticks.lock(){ticks.receive(&line,Instant::now(),chrono::Utc::now().timestamp_millis());}}
            let event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &line).ok())
                .unwrap_or_else(|| log::parse(&id, &log::stamp(&line)));
            if *control.state.lock().unwrap()=="starting" {
                let changed={let mut phase=control.startup_phase.lock().unwrap();let before=*phase;phase.observe(&event);(*phase!=before).then_some(*phase)};
                if let Some(phase)=changed{let _=app.emit("server-startup",crate::startup::Event{server_id:id.clone(),session_id:control.session.clone(),phase});}
            }
            if event.event.as_deref() == Some("startup") {
                let mut state = control.state.lock().unwrap();
                if *state == "starting" {
                    *state = "running".into();
                    *control.running_since.lock().unwrap()=Some(Instant::now());
                    emit_state(&app, &id, "running");
                }
            }
            crate::players::apply_log_event(&mut control.online.lock().unwrap(),&event);
            let _ = app.emit("server-log", event);
    })
}

fn save_exit(directory:&std::path::Path,id:&str,control:&ProcessControl,code:Option<i32>,complete:bool,monitor_failed:bool){
    let time=chrono::Utc::now().timestamp_millis();
    let lines=control.output.lock().map(|output|output.lines()).unwrap_or_default();
    let forced=control.forced.load(std::sync::atomic::Ordering::SeqCst);
    let mut diagnoses=crate::crash_diagnostics::analyze_exit(&lines,forced);
    if monitor_failed{diagnoses.insert(0,crate::crash_diagnostics::Diagnosis{kind:"process_monitor".into(),title:"Falló la supervisión de Java".into(),explanation:"El launcher no pudo consultar el proceso y solicitó su finalización. Esta salida no se considera una parada limpia.".into(),recommendation:"Revisa la consola antes de volver a iniciar el servidor.".into(),action:"console".into(),evidence:vec![]});}
    diagnoses.truncate(6);
    let report=crate::crash_diagnostics::Report{server_id:id.into(),session_id:control.session.clone(),ended_at:time,exit_code:code,stop_requested:control.stop_requested.load(std::sync::atomic::Ordering::SeqCst),forced,output_complete:complete,diagnoses,recent_files:crate::crash_diagnostics::recent_files(directory,time)};
    if let Err(error)=crate::crash_diagnostics::save(directory,&report){eprintln!("Could not save process diagnostics: {error}");}
}
fn drained(stdout:&std::thread::JoinHandle<()>,stderr:&std::thread::JoinHandle<()>)->bool{
    let deadline=Instant::now()+Duration::from_secs(2);
    while !(stdout.is_finished()&&stderr.is_finished())&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(10));}
    stdout.is_finished()&&stderr.is_finished()
}

// Killing does not reap an owned child. Never discard ownership when neither
// termination nor an already completed exit can be confirmed.
fn terminate_and_reap(child:&mut Child)->std::io::Result<std::process::ExitStatus>{
    match child.kill(){
        Ok(())=>child.wait(),
        Err(error)=>match child.try_wait(){Ok(Some(status))=>Ok(status),_=>Err(error)},
    }
}

impl ProcessManager {
    fn record_terminal(&self,id:&str,control:&Arc<ProcessControl>,successful:bool)->String{
        let successful=successful&&!control.forced.load(std::sync::atomic::Ordering::SeqCst);
        let state=if successful{"stopped"}else{"crashed"};
        *control.state.lock().unwrap()=state.into();
        self.terminal_states.lock().unwrap().insert(id.into(),state.into());
        self.exits.lock().unwrap().insert(id.into(),Exit{token:control.session.clone(),successful,was_running:control.running_since.lock().unwrap().is_some(),unexpected:!successful&&!control.stop_requested.load(std::sync::atomic::Ordering::SeqCst)});
        state.into()
    }
    fn release_terminal(&self,id:&str,control:&Arc<ProcessControl>){
        let mut processes=self.processes.lock().unwrap();
        if processes.get(id).is_some_and(|current|Arc::ptr_eq(current,control)){processes.remove(id);}
    }
    pub fn tick_snapshot(&self,id:&str)->crate::ticks::Snapshot{
        let process=self.processes.lock().unwrap().get(id).cloned();
        process.filter(|process|process.paper&&*process.state.lock().unwrap()=="running").and_then(|process|process.ticks.lock().ok().map(|ticks|ticks.snapshot(Instant::now()))).unwrap_or(crate::ticks::Snapshot{reading:None,pending:false})
    }
    pub fn request_ticks(&self,id:&str)->Result<(),String>{
        let process=self.processes.lock().unwrap().get(id).cloned().ok_or("Inicia el servidor antes de consultar los TPS")?;
        if !process.paper{return Err("La consulta de TPS está disponible para Paper".into());}
        if *process.state.lock().unwrap()!="running"{return Err("Espera a que el servidor termine de iniciar".into());}
        process.ticks.lock().map_err(|_|"No se pudo preparar la consulta de TPS")?.begin(Instant::now())?;
        let result=(||{let mut stdin=process.stdin.lock().map_err(|_|"No se pudo abrir la consola")?;writeln!(stdin,"spigot:tps").and_then(|_|stdin.flush()).map_err(|e|format!("No se pudo consultar los TPS: {e}"))})();
        if result.is_err(){if let Ok(mut ticks)=process.ticks.lock(){ticks.failed();}}
        result
    }
    pub fn exits(&self)->HashMap<String,Exit> {self.exits.lock().unwrap().clone()}
    pub fn exit_token(&self,id:&str)->Option<String> {self.exits.lock().unwrap().get(id).map(|exit|exit.token.clone())}
    pub fn healthy_for(&self,id:&str,seconds:u64)->bool {
        self.processes.lock().unwrap().get(id).is_some_and(|process|*process.state.lock().unwrap()=="running"&&process.running_since.lock().unwrap().is_some_and(|since|since.elapsed()>=Duration::from_secs(seconds)))
    }
    pub fn active_ids(&self) -> Vec<String> {
        self.processes.lock().unwrap().keys().cloned().collect()
    }

    pub fn any_active(&self) -> bool {
        !self.processes.lock().unwrap().is_empty()
    }

    pub fn state(&self, id: &str) -> String {
        self.processes.lock().unwrap().get(id)
            .map(|p| p.state.lock().unwrap().clone())
            .unwrap_or_else(|| self.terminal_states.lock().unwrap().get(id).cloned().unwrap_or_else(|| "stopped".into()))
    }

    pub fn is_active(&self, id: &str) -> bool { self.processes.lock().unwrap().contains_key(id) }

    pub fn online(&self, id: &str) -> Vec<String> {
        self.processes.lock().unwrap().get(id)
            .map(|p| p.online.lock().unwrap().iter().cloned().collect()).unwrap_or_default()
    }

    pub fn session_token(&self,id:&str)->Option<String>{
        self.processes.lock().ok()?.get(id).map(|process|process.session.clone())
    }

    pub fn process_snapshot(&self, id: &str) -> Option<(u32,u64,Option<u64>,crate::startup::Phase)> {
        self.processes.lock().unwrap().get(id)
            .map(|p| (p.child.lock().unwrap().id(),p.started.elapsed().as_secs(),p.running_since.lock().unwrap().map(|ready|ready.duration_since(p.started).as_millis() as u64),*p.startup_phase.lock().unwrap()))
    }

    pub fn memory_sample_identity(&self,id:&str)->Option<((u32,u64,Option<u64>,crate::startup::Phase),String)>{
        self.processes.lock().ok()?.get(id).map(|p|((p.child.lock().unwrap().id(),p.started.elapsed().as_secs(),p.running_since.lock().unwrap().map(|ready|ready.duration_since(p.started).as_millis() as u64),*p.startup_phase.lock().unwrap()),p.session.clone()))
    }

    pub fn start(&self, server: &Server, app: AppHandle, jvm:&crate::jvm::Settings) -> Result<(), String> {
        use tauri::Manager;
        let engine=app.state::<crate::Engine>();
        if engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("La aplicación se está cerrando".into());
        }
        if let Err(error)=engine.backup_after_stop(&server.id,app.clone()){eprintln!("Stop backup before start: {error}");}
        let jar = server.jar_path.as_ref().ok_or("Instala primero el servidor")?;
        let java = server.java_path.as_ref().ok_or("Java no está instalado")?;
        if !std::path::Path::new(jar).is_file() || !std::path::Path::new(java).is_file() {
            return Err("Falta el archivo del servidor o el runtime Java. Reinstala el servidor.".into());
        }
        if !fs::read_to_string(std::path::Path::new(&server.directory).join("eula.txt")).unwrap_or_default().lines().any(|line| line.trim() == "eula=true") {
            return Err("Debes aceptar explícitamente la EULA de Minecraft antes de iniciar".into());
        }
        let properties = crate::properties::read(&std::path::Path::new(&server.directory).join("server.properties"))?;
        let port = properties.get("server-port").and_then(|v| v.parse::<u16>().ok()).ok_or("El puerto del servidor no es válido")?;
        let probe = std::net::TcpListener::bind(("0.0.0.0",port))
            .map_err(|e| format!("El puerto {port} no está disponible: {e}"))?;
        drop(probe);
        let mut map = self.processes.lock().unwrap();
        if map.contains_key(&server.id) { return Err("Este servidor ya está en ejecución".into()); }
        let log_dir = std::path::Path::new(&server.directory).join("launcher-logs");
        fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
        let log_path = log_dir.join("latest.log");
        let log_file = Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(&log_path)
            .map_err(|e| format!("No se pudo abrir el log del launcher: {e}"))?));
        let mut command=Command::new(java);
        #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        if engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("La aplicación se está cerrando".into());
        }
        let started=Instant::now();
        let mut child = command
            .args(crate::jvm::launch_arguments(server,jvm)?)
            .current_dir(&server.directory)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| format!("Java no pudo arrancar: {e}"))?;
        let stdin = child.stdin.take().ok_or("No se pudo abrir la entrada del servidor")?;
        let stdout = child.stdout.take().ok_or("No se pudo abrir la consola del servidor")?;
        let stderr = child.stderr.take().ok_or("No se pudo abrir la salida de errores")?;
        let control = Arc::new(ProcessControl::new(child,stdin,server.platform=="paper",started));
        map.insert(server.id.clone(), control.clone());
        self.terminal_states.lock().unwrap().remove(&server.id);
        self.exits.lock().unwrap().remove(&server.id);
        drop(map);
        emit_state(&app, &server.id, "starting");
        let _=app.emit("server-startup",crate::startup::Event{server_id:server.id.clone(),session_id:control.session.clone(),phase:crate::startup::Phase::Launching});
        let stdout_pump=pump(stdout, app.clone(), server.id.clone(), log_file.clone(),control.clone(),true);
        let stderr_pump=pump(stderr, app.clone(), server.id.clone(), log_file.clone(),control.clone(),false);
        let manager=self.clone();
        let id = server.id.clone();
        let directory=std::path::PathBuf::from(&server.directory);
        let mut monitor_failed=false;
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            let exit = control.child.lock().unwrap().try_wait();
            match exit {
                Ok(Some(status)) => {
                    let complete=drained(&stdout_pump,&stderr_pump);save_exit(&directory,&id,&control,status.code(),complete,monitor_failed);
                    let final_state=manager.record_terminal(&id,&control,status.success()&&!monitor_failed);
                    emit_state(&app,&id,&final_state);
                    manager.release_terminal(&id,&control);
                    let forced=control.forced.load(std::sync::atomic::Ordering::SeqCst);
                    let message = if forced {format!("Proceso finalizado: {status}. Cierre forzado solicitado; guardado sin confirmar")}else{format!("Proceso finalizado: {status}")};
                    let mut event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &message).ok())
                        .unwrap_or_else(|| log::parse(&id, &log::stamp(&message)));
                    if !status.success()||forced||monitor_failed { event.level = "error".into(); event.event = Some("crash".into()); }
                    let _ = app.emit("server-log", event);
                    break;
                }
                Ok(None) => (),
                Err(e) => {
                    if !monitor_failed {
                        monitor_failed=true;
                        *control.state.lock().unwrap()="stopping".into();
                        emit_state(&app,&id,"stopping");
                        let message=format!("No se pudo consultar el proceso: {e}. Se intenta finalizar Java; el launcher conserva su control hasta confirmar la salida.");
                        let mut event=log_file.lock().ok().and_then(|mut file|log::append(&mut file,&id,&message).ok()).unwrap_or_else(||log::parse(&id,&log::stamp(&message)));
                        event.level="error".into();let _=app.emit("server-log",event);
                    }
                    let cleanup=terminate_and_reap(&mut control.child.lock().unwrap());
                    let status=match cleanup {
                        Ok(status)=>status,
                        // Retain the PID/session and retry observation. Releasing it
                        // here would allow another Java process to start unnoticed.
                        Err(_)=>continue,
                    };
                    let complete=drained(&stdout_pump,&stderr_pump);save_exit(&directory,&id,&control,status.code(),complete,true);
                    manager.record_terminal(&id,&control,false);
                    emit_state(&app,&id,"crashed");
                    manager.release_terminal(&id,&control);
                    let message = format!("Java finalizado tras un error al consultar el proceso: {e}. Estado: {status}");
                    let mut event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &message).ok())
                        .unwrap_or_else(|| log::parse(&id, &log::stamp(&message)));
                    event.level = "error".into(); event.event = Some("crash".into());
                    let _ = app.emit("server-log", event);
                    break;
                }
            }
        });
        Ok(())
    }

    pub fn send(&self, id: &str, command: &str) -> Result<(), String> {
        if command.is_empty() || command.len() > 512 || command.contains(['\r','\n']) {
            return Err("Comando vacío o no válido".into());
        }
        let process = self.processes.lock().unwrap().get(id).cloned().ok_or("El servidor no está en ejecución")?;
        let mut stdin = process.stdin.lock().unwrap();
        writeln!(stdin, "{}", command.trim_start_matches('/')).and_then(|_| stdin.flush())
            .map_err(|e| format!("No se pudo enviar el comando: {e}"))
    }

    pub fn stop(&self, id: &str, app: AppHandle) -> Result<(), String> {
        let process = self.processes.lock().unwrap().get(id).cloned().ok_or("El servidor no está en ejecución")?;
        let previous_state = {
            let mut state = process.state.lock().unwrap();
            if *state == "stopping" { return Ok(()); }
            if *state != "starting" && *state != "running" { return Err("El proceso ya ha terminado".into()); }
            let previous = state.clone();
            process.stop_requested.store(true,std::sync::atomic::Ordering::SeqCst);
            *state = "stopping".into();
            emit_state(&app,id,"stopping");
            previous
        };
        if let Err(error) = self.send(id,"stop") {
            process.stop_requested.store(false,std::sync::atomic::Ordering::SeqCst);
            let mut state = process.state.lock().unwrap();
            if *state == "stopping" { *state = previous_state; emit_state(&app,id,&state); }
            return Err(error);
        }
        Ok(())
    }

    pub fn preview_force_stop(&self,id:&str)->Result<ForceStopPreview,String>{
        let process=self.processes.lock().map_err(|e|e.to_string())?.get(id).cloned().ok_or("El servidor no está en ejecución")?;
        let state=process.state.lock().map_err(|e|e.to_string())?;
        validate_force_stop(&state,&process.session,&process.session)?;
        let mut child=process.child.lock().map_err(|e|e.to_string())?;
        if child.try_wait().map_err(|e|e.to_string())?.is_some(){return Err("El proceso ya ha terminado".into());}
        Ok(ForceStopPreview{session_id:process.session.clone(),pid:child.id()})
    }

    pub fn force_stop(&self,id:&str,expected:&str)->Result<(),String>{
        let process=self.processes.lock().map_err(|e|e.to_string())?.get(id).cloned().ok_or("El servidor no está en ejecución")?;
        terminate(&process,expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn captured_output<R:std::io::Read+Send+'static>(reader:R)->Vec<String>{
        let lines=Arc::new(Mutex::new(Vec::new()));let captured=lines.clone();
        read_output(reader,move|line|captured.lock().unwrap().push(line)).join().unwrap();
        Arc::try_unwrap(lines).unwrap().into_inner().unwrap()
    }
    #[test]
    fn malformed_output_keeps_later_startup_events_and_final_unterminated_lines(){
        let mut bytes=b"bad \xff\xfe text\r\n\n".to_vec();
        bytes.extend_from_slice(b"[12:00:00] [Server thread/INFO]: Done (1.2s)! For help, type \"help\"\r\nlast\r");
        let lines=captured_output(std::io::Cursor::new(bytes));
        assert_eq!(lines.len(),4);assert!(lines[0].contains(char::REPLACEMENT_CHARACTER));assert!(!lines[0].ends_with('\r'));assert_eq!(lines[1],"");
        assert_eq!(log::parse("server",&lines[2]).event.as_deref(),Some("startup"));assert_eq!(lines[3],"last\r");
    }
    #[test]
    fn oversized_output_is_bounded_and_drained_without_creating_fake_lines(){
        let mut bytes=vec![b'x';4*1024*1024];
        bytes.extend_from_slice(b"\n[12:00:00] [Server thread/INFO]: Done (1.2s)! For help, type \"help\"\n");
        let lines=captured_output(std::io::Cursor::new(bytes));
        assert_eq!(lines.len(),2);assert!(lines[0].len()<MAX_OUTPUT_LINE_BYTES+64);assert!(lines[0].ends_with("[output line truncated at 64 KiB]"));
        assert_eq!(log::parse("server",&lines[1]).event.as_deref(),Some("startup"));
        let lines=captured_output(std::io::Cursor::new(vec![0xff;MAX_OUTPUT_LINE_BYTES*2]));
        assert_eq!(lines.len(),1);assert!(lines[0].len()<MAX_OUTPUT_LINE_BYTES*3+64);
    }
    #[test]
    fn interrupted_and_fragmented_output_preserves_utf8_and_line_boundaries(){
        struct Fragmented{bytes:std::io::Cursor<Vec<u8>>,interrupt:bool}
        impl std::io::Read for Fragmented{
            fn read(&mut self,buffer:&mut [u8])->std::io::Result<usize>{
                if self.interrupt{self.interrupt=false;return Err(std::io::ErrorKind::Interrupted.into());}
                self.interrupt=true;let length=buffer.len().min(1);std::io::Read::read(&mut self.bytes,&mut buffer[..length])
            }
        }
        let text="世界\r\n\nready\n";
        assert_eq!(captured_output(Fragmented{bytes:std::io::Cursor::new(text.as_bytes().to_vec()),interrupt:true}),vec!["世界","","ready"]);
    }
    #[test]
    #[ignore="Spawned only by the force-stop process test with an explicit fixture marker"]
    fn force_stop_child_fixture(){
        if std::env::var("TCH_FORCE_STOP_FIXTURE").as_deref()!=Ok("owned-test-child"){return;}
        std::thread::sleep(Duration::from_secs(60));
    }

    #[test]
    fn monitor_cleanup_reaps_a_real_owned_child_before_releasing_its_handle(){
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args(["--exact","process::tests::force_stop_child_fixture","--ignored"])
            .env("TCH_FORCE_STOP_FIXTURE","owned-test-child").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        struct OwnedChild(Child);impl Drop for OwnedChild{fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
        let mut child=OwnedChild(command.spawn().unwrap());
        assert!(child.0.try_wait().unwrap().is_none());
        let status=terminate_and_reap(&mut child.0).unwrap();
        assert_eq!(child.0.try_wait().unwrap(),Some(status));
        // Repeated cleanup of an already reaped child remains a completed exit.
        assert_eq!(terminate_and_reap(&mut child.0).unwrap(),status);
    }

    #[test]
    fn a_monitor_failure_never_becomes_a_clean_stop_even_if_exit_code_is_zero(){
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.arg("--list").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let mut child=command.spawn().unwrap();let stdin=child.stdin.take().unwrap();
        let status=child.wait().unwrap();assert!(status.success());
        let control=Arc::new(ProcessControl::new(child,stdin,false,Instant::now()));
        let manager=ProcessManager::default();manager.processes.lock().unwrap().insert("monitor".into(),control.clone());
        let monitor_failed=true;
        assert_eq!(manager.record_terminal("monitor",&control,status.success()&&!monitor_failed),"crashed");
        assert!(!manager.exits()["monitor"].successful);assert!(manager.exits()["monitor"].unexpected);
        let directory=std::env::temp_dir().join(format!("tch-monitor-exit-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();
        save_exit(&directory,"monitor",&control,status.code(),true,true);
        let report=crate::crash_diagnostics::last_failure(&directory,"monitor").unwrap().unwrap();
        assert_eq!(report.exit_code,Some(0));assert_eq!(report.diagnoses[0].kind,"process_monitor");assert_eq!(report.session_id,control.session);
        control.forced.store(true,std::sync::atomic::Ordering::SeqCst);
        for line in ["java.lang.UnsupportedClassVersionError: Main","java.lang.OutOfMemoryError: Java heap space","java.net.BindException: address in use","Incompatible mods found!","Error occurred while enabling Example v1"]{control.output.lock().unwrap().push(line);}
        save_exit(&directory,"monitor",&control,status.code(),true,true);
        let report=crate::crash_diagnostics::last_failure(&directory,"monitor").unwrap().unwrap();
        assert_eq!(report.diagnoses.len(),6);assert_eq!(report.diagnoses[0].kind,"process_monitor");assert_eq!(report.diagnoses[1].kind,"forced_stop");
        fs::remove_dir_all(directory).unwrap();
        manager.release_terminal("monitor",&control);assert!(!manager.is_active("monitor"));
    }

    #[test]
    fn forced_stop_requires_stopping_and_exact_session_and_never_marks_a_clean_exit(){
        let mut command=Command::new(std::env::current_exe().unwrap());
        command.args(["--exact","process::tests::force_stop_child_fixture","--ignored"])
            .env("TCH_FORCE_STOP_FIXTURE","owned-test-child").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
        #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let mut child=command.spawn().unwrap();let stdin=child.stdin.take().unwrap();
        let control=Arc::new(ProcessControl::new(child,stdin,false,Instant::now()));
        struct Cleanup(Arc<ProcessControl>);
        impl Drop for Cleanup{fn drop(&mut self){if let Ok(mut child)=self.0.child.lock(){let _=child.kill();let _=child.wait();}}}
        let _cleanup=Cleanup(control.clone());
        let manager=ProcessManager::default();manager.processes.lock().unwrap().insert("test".into(),control.clone());
        assert!(manager.preview_force_stop("test").is_err());
        assert!(manager.force_stop("test",&control.session).is_err());
        assert!(control.child.lock().unwrap().try_wait().unwrap().is_none());
        *control.state.lock().unwrap()="stopping".into();control.stop_requested.store(true,std::sync::atomic::Ordering::SeqCst);
        *control.running_since.lock().unwrap()=Some(Instant::now());
        let preview=manager.preview_force_stop("test").unwrap();assert_eq!(preview.session_id,control.session);assert_eq!(preview.pid,control.child.lock().unwrap().id());
        assert!(manager.force_stop("test","obsolete-session").is_err());
        assert!(!control.forced.load(std::sync::atomic::Ordering::SeqCst));
        assert!(control.child.lock().unwrap().try_wait().unwrap().is_none());
        manager.force_stop("test",&preview.session_id).unwrap();
        let deadline=Instant::now()+Duration::from_secs(5);
        loop{if control.child.lock().unwrap().try_wait().unwrap().is_some(){break;}assert!(Instant::now()<deadline);std::thread::sleep(Duration::from_millis(10));}
        assert!(control.forced.load(std::sync::atomic::Ordering::SeqCst));
        assert!(manager.force_stop("test",&preview.session_id).is_err());
        // Even a platform reporting success after termination cannot trigger clean-stop backups.
        assert_eq!(manager.record_terminal("test",&control,true),"crashed");
        let exit=manager.exits()["test"].clone();assert!(exit.was_running);assert!(!exit.successful);assert!(!exit.unexpected);
        let directory=std::env::temp_dir().join(format!("tch-forced-child-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();
        save_exit(&directory,"test",&control,Some(0),true,false);
        let report=crate::crash_diagnostics::last_failure(&directory,"test").unwrap().unwrap();assert!(report.forced);assert!(report.stop_requested);assert_eq!(report.session_id,preview.session_id);assert_eq!(report.diagnoses[0].kind,"forced_stop");
        fs::remove_dir_all(directory).unwrap();
        manager.release_terminal("test",&control);assert!(!manager.is_active("test"));
        assert!(manager.force_stop("test",&preview.session_id).is_err());
    }
    #[test]
    #[ignore="Requires TCH_TEST_JAVA; runs Java on a deliberately unsupported test class, never Minecraft"]
    fn real_java_exit_preserves_stderr_session_report_and_terminal_state(){
        let java=std::env::var("TCH_TEST_JAVA").expect("Set TCH_TEST_JAVA to an installed runtime");
        let directory=std::env::temp_dir().join(format!("tch-process-diagnostic-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();
        // Java rejects major version 32767 before executing any class code.
        fs::write(directory.join("Unsupported.class"),[0xca,0xfe,0xba,0xbe,0,0,0x7f,0xff]).unwrap();
        let manager=ProcessManager::default();let id=uuid::Uuid::new_v4().to_string();let mut old_session=None;
        for failure in [true,false]{
            let mut command=Command::new(&java);
            #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
            if failure{command.args(["-cp",".","Unsupported"]);}else{command.arg("-version");}
            let started=Instant::now();
            let mut child=command.current_dir(&directory).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
            let stdin=child.stdin.take().unwrap();let stdout=child.stdout.take().unwrap();let stderr=child.stderr.take().unwrap();let control=Arc::new(ProcessControl::new(child,stdin,false,started));manager.processes.lock().unwrap().insert(id.clone(),control.clone());
            assert_eq!(manager.process_snapshot(&id).unwrap().2,None);
            assert_eq!(manager.process_snapshot(&id).unwrap().3,crate::startup::Phase::Launching);
            let file=Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(directory.join("launcher-logs/latest.log")).unwrap()));
            let out_control=control.clone();let out_file=file.clone();let out_id=id.clone();
            let out=read_output(stdout,move|line|{out_control.output.lock().unwrap().push(&line);log::append(&mut out_file.lock().unwrap(),&out_id,&line).unwrap();});
            let err_control=control.clone();let err_file=file.clone();let err_id=id.clone();
            let err=read_output(stderr,move|line|{err_control.output.lock().unwrap().push(&line);log::append(&mut err_file.lock().unwrap(),&err_id,&line).unwrap();});
            let deadline=Instant::now()+Duration::from_secs(10);let status=loop{if let Some(status)=control.child.lock().unwrap().try_wait().unwrap(){break status;}if Instant::now()>=deadline{let _=control.child.lock().unwrap().kill();panic!("Java did not finish in time");}std::thread::sleep(Duration::from_millis(10));};
            assert_eq!(status.success(),!failure);let complete=drained(&out,&err);assert!(complete);out.join().unwrap();err.join().unwrap();save_exit(&directory,&id,&control,status.code(),complete,false);
            let state=manager.record_terminal(&id,&control,status.success());manager.release_terminal(&id,&control);assert_eq!(state,if failure{"crashed"}else{"stopped"});assert_eq!(manager.state(&id),state);assert!(!manager.is_active(&id));
            let session=manager.exit_token(&id).unwrap();assert_eq!(session,control.session);let report=crate::crash_diagnostics::load(&directory,&id,&session).unwrap().unwrap();assert_eq!(report.exit_code,status.code());assert!(report.output_complete);assert!(!report.stop_requested);assert_eq!(manager.exits()[&id].unexpected,failure);assert_eq!(manager.exits()[&id].successful,!failure);assert!(!manager.exits()[&id].was_running);assert!(!crate::stop_backup::pending(&directory,&id,&manager.exits()[&id]).unwrap());
            if failure{assert!(report.diagnoses.iter().any(|diagnosis|diagnosis.kind=="java_version"));assert!(fs::read_to_string(directory.join("launcher-logs/latest.log")).unwrap().contains("UnsupportedClassVersionError"));}else{assert!(!report.diagnoses.iter().any(|diagnosis|diagnosis.kind=="java_version"));assert!(crate::crash_diagnostics::load(&directory,&id,old_session.as_deref().unwrap()).unwrap().is_none());}
            let history=crate::crash_diagnostics::last_failure(&directory,&id).unwrap().unwrap();
            assert_eq!(history.session_id,if failure{session.as_str()}else{old_session.as_deref().unwrap()});
            assert!(history.diagnoses.iter().any(|diagnosis|diagnosis.kind=="java_version"));
            let fresh_manager=ProcessManager::default();assert_eq!(fresh_manager.state(&id),"stopped");assert!(fresh_manager.exit_token(&id).is_none());
            assert!(crate::crash_diagnostics::last_failure(&directory,&id).unwrap().is_some());
            old_session=Some(session);
        }
        assert!(!directory.join("eula.txt").exists());assert!(!directory.join("world").exists());fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn a_finished_crash_remains_visible_without_an_active_process() {
        let manager = ProcessManager::default();
        manager.terminal_states.lock().unwrap().insert("server".into(),"crashed".into());
        assert!(!manager.is_active("server"));
        assert_eq!(manager.state("server"),"crashed");
        assert_eq!(manager.state("unknown"),"stopped");
    }
}

