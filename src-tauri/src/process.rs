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

// The process engine owns lifecycle/state. Only event delivery depends on Tauri,
// allowing integration tests to observe the production engine without a window.
trait ProcessEvents:Send+Sync{
    fn closing(&self)->bool;
    fn publish_state(&self,id:&str,state:&str);
    fn startup(&self,event:crate::startup::Event);
    fn log(&self,event:crate::model::LogEvent);
}
impl ProcessEvents for AppHandle{
    fn closing(&self)->bool{use tauri::Manager;self.state::<crate::Engine>().scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst)}
    fn publish_state(&self,id:&str,state:&str){emit_state(self,id,state)}
    fn startup(&self,event:crate::startup::Event){let _=self.emit("server-startup",event);}
    fn log(&self,event:crate::model::LogEvent){let _=self.emit("server-log",event);}
}
const MAX_OUTPUT_LINE_BYTES:usize=64*1024;
struct OutputReader{
    handle:std::thread::JoinHandle<()>,
    complete:Arc<std::sync::atomic::AtomicBool>,
}
impl OutputReader{
    fn is_finished(&self)->bool{self.handle.is_finished()}
    fn is_complete(&self)->bool{self.is_finished()&&self.complete.load(std::sync::atomic::Ordering::Acquire)}
}

fn read_output_observed<R:std::io::Read+Send+'static>(reader:R,mut consume:impl FnMut(String)+Send+'static,on_error:impl FnOnce(std::io::Error)+Send+'static)->OutputReader{
    let complete=Arc::new(std::sync::atomic::AtomicBool::new(false));
    let result=complete.clone();
    let handle=std::thread::spawn(move||{
        let mut reader=BufReader::new(reader);
        let mut line=Vec::new();let mut truncated=false;let mut all_lines_retained=true;
        loop{
            let bytes=match reader.fill_buf(){Ok(bytes)=>bytes,Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,Err(error)=>{on_error(error);break;}};
            if bytes.is_empty(){
                if !line.is_empty()||truncated{consume(output_line(&line,truncated,false));}
                result.store(all_lines_retained,std::sync::atomic::Ordering::Release);
                break;
            }
            let newline=bytes.iter().position(|byte|*byte==b'\n');
            let length=newline.unwrap_or(bytes.len());
            let retained=length.min(MAX_OUTPUT_LINE_BYTES-line.len());
            line.extend_from_slice(&bytes[..retained]);truncated|=retained<length;all_lines_retained&=retained==length;
            reader.consume(length+usize::from(newline.is_some()));
            if newline.is_some(){consume(output_line(&line,truncated,true));line.clear();truncated=false;}
        }
    });
    OutputReader{handle,complete}
}
fn output_line(bytes:&[u8],truncated:bool,terminated:bool)->String{
    let bytes=if !truncated&&terminated{bytes.strip_suffix(b"\r").unwrap_or(bytes)}else{bytes};
    let mut line=String::from_utf8_lossy(bytes).into_owned();
    if truncated{line.push_str(" [output line truncated at 64 KiB]");}
    line
}
fn console_command(command:&str)->Result<&str,String>{
    if command.len()>512||command.chars().any(char::is_control){return Err("Comando vacío o no válido".into());}
    let command=command.trim();
    if command.trim_matches('/').trim().is_empty(){return Err("Comando vacío o no válido".into());}
    Ok(command.strip_prefix('/').unwrap_or(command))
}

fn pump<R: std::io::Read + Send + 'static>(reader: R, events:Arc<dyn ProcessEvents>, id: String, log_file: Arc<Mutex<File>>, control: Arc<ProcessControl>,stdout:bool) ->OutputReader {
    let failure_events=events.clone();let failure_id=id.clone();let failure_file=log_file.clone();let failure_control=control.clone();
    read_output_observed(reader,move |line| {
            if let Ok(mut output)=control.output.lock(){output.push(&line);}
            if stdout&&control.paper{if let Ok(mut ticks)=control.ticks.lock(){ticks.receive(&line,Instant::now(),chrono::Utc::now().timestamp_millis());}}
            let event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &line).ok())
                .unwrap_or_else(|| log::parse(&id, &log::stamp(&line)));
            if *control.state.lock().unwrap()=="starting" {
                let changed={let mut phase=control.startup_phase.lock().unwrap();let before=*phase;phase.observe(&event);(*phase!=before).then_some(*phase)};
                if let Some(phase)=changed{events.startup(crate::startup::Event{server_id:id.clone(),session_id:control.session.clone(),phase});}
            }
            if event.event.as_deref() == Some("startup") {
                let mut state = control.state.lock().unwrap();
                if *state == "starting" {
                    *state = "running".into();
                    *control.running_since.lock().unwrap()=Some(Instant::now());
                    events.publish_state(&id, "running");
                }
            }
            crate::players::apply_log_event(&mut control.online.lock().unwrap(),&event);
            events.log(event);
    },move |error|{
        let stream=if stdout{"stdout"}else{"stderr"};
        let message=format!("[TCHLauncher] No se pudo leer la salida de Java ({stream}): {error}. La consola y el diagnóstico pueden estar incompletos.");
        if let Ok(mut output)=failure_control.output.lock(){output.push(&message);}
        let mut event=failure_file.lock().ok().and_then(|mut file|log::append(&mut file,&failure_id,&message).ok()).unwrap_or_else(||log::parse(&failure_id,&log::stamp(&message)));
        event.level="error".into();event.event=None;failure_events.log(event);
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
fn drained(stdout:&OutputReader,stderr:&OutputReader)->bool{
    let deadline=Instant::now()+Duration::from_secs(2);
    while !(stdout.is_finished()&&stderr.is_finished())&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(10));}
    stdout.is_complete()&&stderr.is_complete()
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

    #[cfg(test)]
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
        drop(engine);
        self.launch(server,Arc::new(app),jvm)
    }

    fn launch(&self,server:&Server,events:Arc<dyn ProcessEvents>,jvm:&crate::jvm::Settings)->Result<(),String>{
        if events.closing(){return Err("La aplicación se está cerrando".into());}
        let jar = server.jar_path.as_ref().ok_or("Instala primero el servidor")?;
        let java = server.java_path.as_ref().ok_or("Java no está instalado")?;
        if !std::path::Path::new(jar).is_file() || !std::path::Path::new(java).is_file() {
            return Err("Falta el archivo del servidor o el runtime Java. Reinstala el servidor.".into());
        }
        if !crate::eula::file_accepted(server) {
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
        if events.closing() {
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
        events.publish_state(&server.id,"starting");
        events.startup(crate::startup::Event{server_id:server.id.clone(),session_id:control.session.clone(),phase:crate::startup::Phase::Launching});
        let stdout_pump=pump(stdout, events.clone(), server.id.clone(), log_file.clone(),control.clone(),true);
        let stderr_pump=pump(stderr, events.clone(), server.id.clone(), log_file.clone(),control.clone(),false);
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
                    events.publish_state(&id,&final_state);
                    manager.release_terminal(&id,&control);
                    let forced=control.forced.load(std::sync::atomic::Ordering::SeqCst);
                    let message = if forced {format!("Proceso finalizado: {status}. Cierre forzado solicitado; guardado sin confirmar")}else{format!("Proceso finalizado: {status}")};
                    let mut event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &message).ok())
                        .unwrap_or_else(|| log::parse(&id, &log::stamp(&message)));
                    if !status.success()||forced||monitor_failed { event.level = "error".into(); event.event = Some("crash".into()); }
                    events.log(event);
                    break;
                }
                Ok(None) => (),
                Err(e) => {
                    if !monitor_failed {
                        monitor_failed=true;
                        *control.state.lock().unwrap()="stopping".into();
                        events.publish_state(&id,"stopping");
                        let message=format!("No se pudo consultar el proceso: {e}. Se intenta finalizar Java; el launcher conserva su control hasta confirmar la salida.");
                        let mut event=log_file.lock().ok().and_then(|mut file|log::append(&mut file,&id,&message).ok()).unwrap_or_else(||log::parse(&id,&log::stamp(&message)));
                        event.level="error".into();events.log(event);
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
                    events.publish_state(&id,"crashed");
                    manager.release_terminal(&id,&control);
                    let message = format!("Java finalizado tras un error al consultar el proceso: {e}. Estado: {status}");
                    let mut event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &message).ok())
                        .unwrap_or_else(|| log::parse(&id, &log::stamp(&message)));
                    event.level = "error".into(); event.event = Some("crash".into());
                    events.log(event);
                    break;
                }
            }
        });
        Ok(())
    }

    pub fn send(&self, id: &str, command: &str) -> Result<(), String> {
        let command=console_command(command)?;
        let process = self.processes.lock().unwrap().get(id).cloned().ok_or("El servidor no está en ejecución")?;
        let mut stdin = process.stdin.lock().unwrap();
        writeln!(stdin, "{}", command).and_then(|_| stdin.flush())
            .map_err(|e| format!("No se pudo enviar el comando: {e}"))
    }

    pub fn send_batch(&self,id:&str,commands:&[String],expected_session:&str)->Result<usize,String>{
        if commands.is_empty()||commands.len()>1000{return Err("La lista debe contener entre 1 y 1000 jugadores".into());}
        let commands=commands.iter().map(|command|console_command(command)).collect::<Result<Vec<_>,_>>()?;
        let process=self.processes.lock().map_err(|_|"No se pudo abrir la consola")?.get(id).cloned().ok_or("El servidor no está en ejecución")?;
        if process.session!=expected_session{return Err("La sesión del servidor ha cambiado. Actualiza la lista antes de importar.".into());}
        for(index,command)in commands.iter().enumerate(){
            let result=(||{
                if *process.state.lock().map_err(|_|"No se pudo consultar el proceso")?!="running"{return Err("El servidor ya no está en ejecución".to_string());}
                let mut stdin=process.stdin.lock().map_err(|_|"No se pudo abrir la consola")?;
                writeln!(stdin,"{command}").and_then(|_|stdin.flush()).map_err(|error|format!("No se pudo enviar el comando: {error}"))
            })();
            if let Err(error)=result{return Err(format!("Se enviaron {index} de {} jugadores antes del error: {error}",commands.len()));}
        }
        Ok(commands.len())
    }

    pub fn stop(&self, id: &str, app: AppHandle) -> Result<(), String> {
        self.request_stop(id,&app)
    }

    fn request_stop(&self,id:&str,events:&dyn ProcessEvents)->Result<(),String>{
        let process = self.processes.lock().unwrap().get(id).cloned().ok_or("El servidor no está en ejecución")?;
        let previous_state = {
            let mut state = process.state.lock().unwrap();
            if *state == "stopping" { return Ok(()); }
            if *state != "starting" && *state != "running" { return Err("El proceso ya ha terminado".into()); }
            let previous = state.clone();
            process.stop_requested.store(true,std::sync::atomic::Ordering::SeqCst);
            *state = "stopping".into();
            events.publish_state(id,"stopping");
            previous
        };
        if let Err(error) = self.send(id,"stop") {
            process.stop_requested.store(false,std::sync::atomic::Ordering::SeqCst);
            let mut state = process.state.lock().unwrap();
            if *state == "stopping" { *state = previous_state; events.publish_state(id,&state); }
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
#[path="process_lifecycle_test.rs"]
mod lifecycle_tests;
