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
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    started: Instant,
    running_since: Mutex<Option<Instant>>,
    state: Mutex<String>,
    online: Mutex<HashSet<String>>,
    paper: bool,
    ticks: Mutex<crate::ticks::Collector>,
    output:Mutex<crate::crash_diagnostics::Output>,
    session:String,
}

impl ProcessControl {
    fn new(child:Child,stdin:ChildStdin,paper:bool)->Self{
        Self{stop_requested:std::sync::atomic::AtomicBool::new(false),child:Mutex::new(child),stdin:Mutex::new(stdin),started:Instant::now(),running_since:Mutex::new(None),state:Mutex::new("starting".into()),online:Mutex::new(HashSet::new()),paper,ticks:Mutex::new(crate::ticks::Collector::default()),output:Mutex::new(crate::crash_diagnostics::Output::default()),session:uuid::Uuid::new_v4().to_string()}
    }
}

#[derive(Clone, Default)]
pub struct ProcessManager {
    pub processes: Arc<Mutex<HashMap<String, Arc<ProcessControl>>>>,
    terminal_states: Arc<Mutex<HashMap<String,String>>>,
    exits: Arc<Mutex<HashMap<String,Exit>>>,
}

#[derive(Clone)]
pub struct Exit {pub token:String,pub unexpected:bool}

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

fn read_output<R:std::io::Read+Send+'static>(reader:R,mut consume:impl FnMut(String)+Send+'static)->std::thread::JoinHandle<()>{
    std::thread::spawn(move||{for line in BufReader::new(reader).lines(){let Ok(line)=line else{break;};consume(line);}})
}

fn pump<R: std::io::Read + Send + 'static>(reader: R, app: AppHandle, id: String, log_file: Arc<Mutex<File>>, control: Arc<ProcessControl>,stdout:bool) ->std::thread::JoinHandle<()> {
    read_output(reader,move |line| {
            if let Ok(mut output)=control.output.lock(){output.push(&line);}
            if stdout&&control.paper{if let Ok(mut ticks)=control.ticks.lock(){ticks.receive(&line,Instant::now(),chrono::Utc::now().timestamp_millis());}}
            let event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &line).ok())
                .unwrap_or_else(|| log::parse(&id, &log::stamp(&line)));
            if event.event.as_deref() == Some("startup") {
                let mut state = control.state.lock().unwrap();
                if *state == "starting" {
                    *state = "running".into();
                    *control.running_since.lock().unwrap()=Some(Instant::now());
                    emit_state(&app, &id, "running");
                }
            }
            if let Some((name, joined)) = crate::players::log_player_change(&line) {
                let mut online = control.online.lock().unwrap();
                if joined { online.insert(name.to_string()); }
                else { online.remove(name); }
            }
            let _ = app.emit("server-log", event);
    })
}

fn save_exit(directory:&std::path::Path,id:&str,control:&ProcessControl,code:Option<i32>,complete:bool){
    let time=chrono::Utc::now().timestamp_millis();
    let lines=control.output.lock().map(|output|output.lines()).unwrap_or_default();
    let report=crate::crash_diagnostics::Report{server_id:id.into(),session_id:control.session.clone(),ended_at:time,exit_code:code,stop_requested:control.stop_requested.load(std::sync::atomic::Ordering::SeqCst),output_complete:complete,diagnoses:crate::crash_diagnostics::analyze(&lines),recent_files:crate::crash_diagnostics::recent_files(directory,time)};
    if let Err(error)=crate::crash_diagnostics::save(directory,&report){eprintln!("Could not save process diagnostics: {error}");}
}
fn drained(stdout:&std::thread::JoinHandle<()>,stderr:&std::thread::JoinHandle<()>)->bool{
    let deadline=Instant::now()+Duration::from_secs(2);
    while !(stdout.is_finished()&&stderr.is_finished())&&Instant::now()<deadline{std::thread::sleep(Duration::from_millis(10));}
    stdout.is_finished()&&stderr.is_finished()
}

impl ProcessManager {
    fn record_terminal(&self,id:&str,control:&Arc<ProcessControl>,successful:bool)->String{
        let state=if successful{"stopped"}else{"crashed"};
        *control.state.lock().unwrap()=state.into();
        self.terminal_states.lock().unwrap().insert(id.into(),state.into());
        self.exits.lock().unwrap().insert(id.into(),Exit{token:control.session.clone(),unexpected:!successful&&!control.stop_requested.load(std::sync::atomic::Ordering::SeqCst)});
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

    pub fn pid_and_uptime(&self, id: &str) -> Option<(u32,u64)> {
        self.processes.lock().unwrap().get(id)
            .map(|p| (p.child.lock().unwrap().id(),p.started.elapsed().as_secs()))
    }

    pub fn start(&self, server: &Server, app: AppHandle, jvm:&crate::jvm::Settings) -> Result<(), String> {
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
        let mut child = command
            .args(crate::jvm::launch_arguments(server,jvm)?)
            .current_dir(&server.directory)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| format!("Java no pudo arrancar: {e}"))?;
        let stdin = child.stdin.take().ok_or("No se pudo abrir la entrada del servidor")?;
        let stdout = child.stdout.take().ok_or("No se pudo abrir la consola del servidor")?;
        let stderr = child.stderr.take().ok_or("No se pudo abrir la salida de errores")?;
        let control = Arc::new(ProcessControl::new(child,stdin,server.platform=="paper"));
        map.insert(server.id.clone(), control.clone());
        self.terminal_states.lock().unwrap().remove(&server.id);
        self.exits.lock().unwrap().remove(&server.id);
        drop(map);
        emit_state(&app, &server.id, "starting");
        let stdout_pump=pump(stdout, app.clone(), server.id.clone(), log_file.clone(),control.clone(),true);
        let stderr_pump=pump(stderr, app.clone(), server.id.clone(), log_file.clone(),control.clone(),false);
        let manager=self.clone();
        let id = server.id.clone();
        let directory=std::path::PathBuf::from(&server.directory);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            let exit = control.child.lock().unwrap().try_wait();
            match exit {
                Ok(Some(status)) => {
                    let complete=drained(&stdout_pump,&stderr_pump);save_exit(&directory,&id,&control,status.code(),complete);
                    let final_state=manager.record_terminal(&id,&control,status.success());
                    emit_state(&app,&id,&final_state);
                    manager.release_terminal(&id,&control);
                    let message = format!("Proceso finalizado: {status}");
                    let mut event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &message).ok())
                        .unwrap_or_else(|| log::parse(&id, &log::stamp(&message)));
                    if !status.success() { event.level = "error".into(); event.event = Some("crash".into()); }
                    let _ = app.emit("server-log", event);
                    break;
                }
                Ok(None) => (),
                Err(e) => {
                    let _ = control.child.lock().unwrap().kill();
                    let complete=drained(&stdout_pump,&stderr_pump);save_exit(&directory,&id,&control,None,complete);
                    manager.record_terminal(&id,&control,false);
                    emit_state(&app,&id,"crashed");
                    manager.release_terminal(&id,&control);
                    let message = format!("No se pudo consultar el proceso: {e}");
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
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(30));
            if let Ok(mut child) = process.child.lock() {
                if child.try_wait().ok().flatten().is_none() { let _ = child.kill(); }
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            let mut child=command.current_dir(&directory).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
            let stdin=child.stdin.take().unwrap();let stdout=child.stdout.take().unwrap();let stderr=child.stderr.take().unwrap();let control=Arc::new(ProcessControl::new(child,stdin,false));manager.processes.lock().unwrap().insert(id.clone(),control.clone());
            let file=Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(directory.join("launcher-logs/latest.log")).unwrap()));
            let out_control=control.clone();let out_file=file.clone();let out_id=id.clone();
            let out=read_output(stdout,move|line|{out_control.output.lock().unwrap().push(&line);log::append(&mut out_file.lock().unwrap(),&out_id,&line).unwrap();});
            let err_control=control.clone();let err_file=file.clone();let err_id=id.clone();
            let err=read_output(stderr,move|line|{err_control.output.lock().unwrap().push(&line);log::append(&mut err_file.lock().unwrap(),&err_id,&line).unwrap();});
            let deadline=Instant::now()+Duration::from_secs(10);let status=loop{if let Some(status)=control.child.lock().unwrap().try_wait().unwrap(){break status;}if Instant::now()>=deadline{let _=control.child.lock().unwrap().kill();panic!("Java did not finish in time");}std::thread::sleep(Duration::from_millis(10));};
            assert_eq!(status.success(),!failure);let complete=drained(&out,&err);assert!(complete);out.join().unwrap();err.join().unwrap();save_exit(&directory,&id,&control,status.code(),complete);
            let state=manager.record_terminal(&id,&control,status.success());manager.release_terminal(&id,&control);assert_eq!(state,if failure{"crashed"}else{"stopped"});assert_eq!(manager.state(&id),state);assert!(!manager.is_active(&id));
            let session=manager.exit_token(&id).unwrap();assert_eq!(session,control.session);let report=crate::crash_diagnostics::load(&directory,&id,&session).unwrap().unwrap();assert_eq!(report.exit_code,status.code());assert!(report.output_complete);assert!(!report.stop_requested);assert_eq!(manager.exits()[&id].unexpected,failure);
            if failure{assert!(report.diagnoses.iter().any(|diagnosis|diagnosis.kind=="java_version"));assert!(fs::read_to_string(directory.join("launcher-logs/latest.log")).unwrap().contains("UnsupportedClassVersionError"));}else{assert!(!report.diagnoses.iter().any(|diagnosis|diagnosis.kind=="java_version"));assert!(crate::crash_diagnostics::load(&directory,&id,old_session.as_deref().unwrap()).unwrap().is_none());}
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

