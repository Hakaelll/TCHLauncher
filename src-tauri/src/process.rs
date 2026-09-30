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
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    started: Instant,
    state: Mutex<String>,
    online: Mutex<HashSet<String>>,
}

#[derive(Clone, Default)]
pub struct ProcessManager {
    pub processes: Arc<Mutex<HashMap<String, Arc<ProcessControl>>>>,
    terminal_states: Arc<Mutex<HashMap<String,String>>>,
}

pub fn emit_state(app: &AppHandle, id: &str, state: &str) {
    let _ = app.emit("server-state", StateEvent { server_id: id.into(), state: state.into() });
}

fn pump<R: std::io::Read + Send + 'static>(reader: R, app: AppHandle, id: String, log_file: Arc<Mutex<File>>, control: Arc<ProcessControl>) {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let Ok(line) = line else { break; };
            let event = log_file.lock().ok().and_then(|mut file| log::append(&mut file, &id, &line).ok())
                .unwrap_or_else(|| log::parse(&id, &log::stamp(&line)));
            if event.event.as_deref() == Some("startup") {
                let mut state = control.state.lock().unwrap();
                if *state == "starting" {
                    *state = "running".into();
                    emit_state(&app, &id, "running");
                }
            }
            if let Some((name, joined)) = crate::players::log_player_change(&line) {
                let mut online = control.online.lock().unwrap();
                if joined { online.insert(name.to_string()); }
                else { online.remove(name); }
            }
            let _ = app.emit("server-log", event);
        }
    });
}

impl ProcessManager {
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

    pub fn start(&self, server: &Server, app: AppHandle) -> Result<(), String> {
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
        let mut child = Command::new(java)
            .arg(format!("-Xms{}M",server.memory_min_mb))
            .arg(format!("-Xmx{}M",server.memory_max_mb))
            .arg("-jar").arg(jar).arg("nogui")
            .current_dir(&server.directory)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| format!("Java no pudo arrancar: {e}"))?;
        let stdin = child.stdin.take().ok_or("No se pudo abrir la entrada del servidor")?;
        let stdout = child.stdout.take().ok_or("No se pudo abrir la consola del servidor")?;
        let stderr = child.stderr.take().ok_or("No se pudo abrir la salida de errores")?;
        let control = Arc::new(ProcessControl { child: Mutex::new(child), stdin: Mutex::new(stdin), started: Instant::now(),state:Mutex::new("starting".into()),online:Mutex::new(HashSet::new()) });
        map.insert(server.id.clone(), control.clone());
        self.terminal_states.lock().unwrap().remove(&server.id);
        drop(map);
        emit_state(&app, &server.id, "starting");
        pump(stdout, app.clone(), server.id.clone(), log_file.clone(),control.clone());
        pump(stderr, app.clone(), server.id.clone(), log_file.clone(),control.clone());
        let all = self.processes.clone();
        let terminal_states = self.terminal_states.clone();
        let id = server.id.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            let exit = control.child.lock().unwrap().try_wait();
            match exit {
                Ok(Some(status)) => {
                    let final_state = if status.success() { "stopped" } else { "crashed" };
                    *control.state.lock().unwrap() = final_state.into();
                    terminal_states.lock().unwrap().insert(id.clone(),final_state.into());
                    emit_state(&app, &id, final_state);
                    all.lock().unwrap().remove(&id);
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
                    *control.state.lock().unwrap() = "crashed".into();
                    terminal_states.lock().unwrap().insert(id.clone(),"crashed".into());
                    emit_state(&app, &id, "crashed");
                    all.lock().unwrap().remove(&id);
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
            *state = "stopping".into();
            emit_state(&app,id,"stopping");
            previous
        };
        if let Err(error) = self.send(id,"stop") {
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
    fn a_finished_crash_remains_visible_without_an_active_process() {
        let manager = ProcessManager::default();
        manager.terminal_states.lock().unwrap().insert("server".into(),"crashed".into());
        assert!(!manager.is_active("server"));
        assert_eq!(manager.state("server"),"crashed");
        assert_eq!(manager.state("unknown"),"stopped");
    }
}

