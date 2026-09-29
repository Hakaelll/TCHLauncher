mod backup;
mod install;
mod log;
mod model;
mod players;
mod process;
mod properties;
mod storage;

use model::{Backup, CreateServer, ImportInspection, ImportServer, LogEvent, Metrics, Server, VersionInfo};
use players::{PlayersSnapshot, WhitelistImport};
use process::ProcessManager;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageSelection { path: String, data: String }

pub struct Engine {
    root: PathBuf,
    processes: ProcessManager,
    metrics_system: Mutex<sysinfo::System>,
    sampled_pids: Mutex<HashSet<u32>>,
    size_cache: Mutex<HashMap<String,(Instant,u64)>>,
}

#[tauri::command]
fn list_servers(engine: State<'_, Engine>) -> Result<Vec<Server>, String> {
    let mut servers = storage::list(&engine.root)?;
    for server in &mut servers { server.state = engine.processes.state(&server.id); }
    Ok(servers)
}

#[tauri::command]
async fn get_versions() -> Result<Vec<VersionInfo>, String> {
    tauri::async_runtime::spawn_blocking(install::versions).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn create_server(engine: State<'_, Engine>, input: CreateServer) -> Result<Server, String> {
    storage::create(&engine.root,input)
}

#[tauri::command]
async fn pick_import_folder() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().into()))
        .await.ok().flatten()
}

#[tauri::command]
async fn pick_server_image() -> Result<Option<ImageSelection>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let Some(path) = rfd::FileDialog::new().add_filter("Imagen", &["png", "jpg", "jpeg", "webp"]).pick_file() else { return Ok(None); };
        let data = storage::preview_image(&path)?;
        Ok(Some(ImageSelection { path: path.to_string_lossy().into(), data }))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_server_image(engine: State<'_, Engine>, id: String, path: String) -> Result<Server, String> {
    storage::set_image(&engine.root, &id, &PathBuf::from(path))
}

#[tauri::command]
fn clear_server_image(engine: State<'_, Engine>, id: String) -> Result<Server, String> {
    storage::clear_image(&engine.root, &id)
}

#[tauri::command]
async fn inspect_import(path: String) -> Result<ImportInspection, String> {
    tauri::async_runtime::spawn_blocking(move || storage::inspect_import(&PathBuf::from(path))).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_server(engine: State<'_, Engine>, input: ImportServer) -> Result<Server, String> {
    let root = engine.root.clone();
    tauri::async_runtime::spawn_blocking(move || storage::import(&root,input)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn install_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    if engine.processes.is_active(&id) { return Err("Detén el servidor antes de reinstalar".into()); }
    let root = engine.root.clone();
    let server = storage::get(&root,&id)?;
    process::emit_state(&app,&id,"installing");
    let task_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || install::install(&root,&server,&task_app))
        .await.map_err(|e| e.to_string())?;
    process::emit_state(&app,&id,if result.is_ok() {"stopped"} else {"error"});
    result
}

#[tauri::command]
fn accept_eula(engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let server = storage::get(&engine.root,&id)?;
    if server.jar_path.is_none() { return Err("Instala el servidor antes de aceptar la EULA".into()); }
    std::fs::write(PathBuf::from(server.directory).join("eula.txt"),"# Accepted explicitly in TCHLauncher\neula=true\n")
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn eula_accepted(engine: State<'_, Engine>, id: String) -> Result<bool, String> {
    let server = storage::get(&engine.root,&id)?;
    Ok(std::fs::read_to_string(PathBuf::from(server.directory).join("eula.txt")).unwrap_or_default().lines().any(|l| l.trim()=="eula=true"))
}

#[tauri::command]
fn start_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let server = storage::get(&engine.root,&id)?;
    engine.processes.start(&server,app)
}

#[tauri::command]
fn stop_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    engine.processes.stop(&id,app)
}

#[tauri::command]
fn send_command(engine: State<'_, Engine>, id: String, command: String) -> Result<(), String> {
    storage::get(&engine.root,&id)?;
    engine.processes.send(&id,&command)
}

#[tauri::command]
fn get_players(engine: State<'_, Engine>, id: String) -> Result<PlayersSnapshot, String> {
    let server = storage::get(&engine.root, &id)?;
    players::snapshot(&server, engine.processes.online(&id))
}

#[tauri::command]
fn manage_player(engine: State<'_, Engine>, id: String, action: String, target: Option<String>, reason: Option<String>) -> Result<(), String> {
    storage::get(&engine.root, &id)?;
    if engine.processes.state(&id) != "running" { return Err("Inicia el servidor para gestionar jugadores".into()); }
    let command = players::command(&action, target.as_deref(), reason.as_deref())?;
    engine.processes.send(&id, &command)
}

#[tauri::command]
async fn pick_whitelist_file() -> Result<Option<WhitelistImport>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let Some(path) = rfd::FileDialog::new().add_filter("Lista de jugadores", &["txt", "json"]).pick_file() else { return Ok(None); };
        players::read_whitelist_import(&path).map(Some)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn import_whitelist(engine: State<'_, Engine>, id: String, names: Vec<String>) -> Result<usize, String> {
    storage::get(&engine.root, &id)?;
    if engine.processes.state(&id) != "running" { return Err("Inicia el servidor antes de importar la lista blanca".into()); }
    let names = players::validate_names(&names)?;
    for (index, name) in names.iter().enumerate() {
        if let Err(e) = engine.processes.send(&id, &players::command("whitelist_add", Some(name), None)?) {
            return Err(format!("Se enviaron {index} de {} jugadores antes del error: {e}", names.len()));
        }
    }
    Ok(names.len())
}

#[tauri::command]
fn read_log(engine: State<'_, Engine>, id: String) -> Result<Vec<LogEvent>, String> {
    let server = storage::get(&engine.root,&id)?;
    let path = PathBuf::from(server.directory).join("launcher-logs/latest.log");
    log::recent(&path, &id)
}

#[tauri::command]
async fn export_log(engine: State<'_, Engine>, id: String) -> Result<Option<String>, String> {
    let server = storage::get(&engine.root, &id)?;
    let source = PathBuf::from(server.directory).join("launcher-logs/latest.log");
    if !source.is_file() { return Err("Aún no hay registros para exportar".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        let suggested_name = format!("tchlauncher-{}-log.txt", server.id);
        let Some(destination) = rfd::FileDialog::new().add_filter("Archivo de texto", &["txt", "log"])
            .set_file_name(&suggested_name).save_file() else { return Ok(None); };
        log::export_snapshot(&source, &destination)?;
        Ok(Some(destination.to_string_lossy().into_owned()))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_properties(engine: State<'_, Engine>, id: String) -> Result<BTreeMap<String,String>, String> {
    let server = storage::get(&engine.root,&id)?;
    properties::read(&PathBuf::from(server.directory).join("server.properties"))
}

#[tauri::command]
fn save_properties(engine: State<'_, Engine>, id: String, values: BTreeMap<String,String>) -> Result<(), String> {
    if engine.processes.is_active(&id) { return Err("Detén el servidor antes de cambiar las propiedades".into()); }
    let server = storage::get(&engine.root,&id)?;
    properties::write(&PathBuf::from(server.directory).join("server.properties"),values)
}

#[tauri::command]
fn save_memory(engine: State<'_, Engine>, id: String, min: u32, max: u32) -> Result<(), String> {
    if engine.processes.is_active(&id) { return Err("Detén el servidor antes de cambiar la memoria".into()); }
    storage::get(&engine.root,&id)?;
    storage::update_memory(&engine.root,&id,min,max)
}

#[tauri::command]
fn save_storage_target(engine: State<'_, Engine>, id: String, gb: u32) -> Result<(), String> {
    storage::update_storage_target(&engine.root, &id, gb)
}

#[tauri::command]
async fn create_backup(engine: State<'_, Engine>, id: String) -> Result<Backup, String> {
    if engine.processes.is_active(&id) { return Err("Detén el servidor antes de crear un backup consistente".into()); }
    let root = engine.root.clone();
    tauri::async_runtime::spawn_blocking(move || backup::create(&root,&id)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn list_backups(engine: State<'_, Engine>, id: String) -> Result<Vec<Backup>, String> {
    storage::get(&engine.root,&id)?;
    storage::list_backups(&engine.root,&id)
}

#[tauri::command]
async fn restore_backup(engine: State<'_, Engine>, id: String, backup_id: String) -> Result<(), String> {
    if engine.processes.is_active(&id) { return Err("Detén el servidor antes de restaurar".into()); }
    let root = engine.root.clone();
    tauri::async_runtime::spawn_blocking(move || backup::restore(&root,&id,&backup_id)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_metrics(engine: State<'_, Engine>, id: String) -> Result<Metrics, String> {
    let server = storage::get(&engine.root,&id)?;
    let pid_uptime = engine.processes.pid_and_uptime(&id);
    let (memory_bytes,cpu_percent) = if let Some((pid,_)) = pid_uptime {
        let mut system = engine.metrics_system.lock().unwrap();
        let sys_pid = sysinfo::Pid::from_u32(pid);
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]),true);
        let process = system.process(sys_pid);
        let sampled = engine.sampled_pids.lock().unwrap().insert(pid);
        (process.map(|p| p.memory()),if sampled { None } else { process.map(|p| p.cpu_usage()) })
    } else { (None,None) };
    let cached = engine.size_cache.lock().unwrap().get(&id).copied()
        .filter(|(when,_)| when.elapsed() < Duration::from_secs(60));
    let size_bytes = if let Some((_,size)) = cached { size } else {
        let size = backup::size(&PathBuf::from(server.directory));
        engine.size_cache.lock().unwrap().insert(id, (Instant::now(),size));
        size
    };
    Ok(Metrics {pid:pid_uptime.map(|p|p.0),uptime_seconds:pid_uptime.map(|p|p.1),memory_bytes,cpu_percent,size_bytes})
}

#[tauri::command]
fn system_memory() -> u64 { sysinfo::System::new_all().total_memory() }

#[tauri::command]
fn hide_window(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main").ok_or("No se encontró la ventana")?
        .hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_all_and_exit(app: AppHandle, engine: State<'_, Engine>) -> Result<(), String> {
    let ids = engine.processes.active_ids();
    for id in &ids { engine.processes.stop(id,app.clone())?; }
    let processes = engine.processes.processes.clone();
    std::thread::spawn(move || {
        for _ in 0..75 {
            if processes.lock().unwrap().is_empty() { break; }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        app.exit(0);
    });
    Ok(())
}

fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            storage::connect(&root).map_err(std::io::Error::other)?;
            app.manage(Engine { root,processes:ProcessManager::default(),metrics_system:Mutex::new(sysinfo::System::new()),sampled_pids:Mutex::new(HashSet::new()),size_cache:Mutex::new(HashMap::new()) });
            let show = tauri::menu::MenuItem::with_id(app,"show","Mostrar TCHLauncher",true,None::<&str>)?;
            let quit = tauri::menu::MenuItem::with_id(app,"quit","Salir",true,None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app,&[&show,&quit])?;
            tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().ok_or("Icono de ventana ausente")?.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app,event| match event.id().as_ref() {
                    "show" => show_main(app),
                    "quit" => {
                        let engine = app.state::<Engine>();
                        if engine.processes.any_active() {
                            show_main(app);
                            let _ = app.emit("close-with-servers",());
                        } else { app.exit(0); }
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray,event| {
                    if let tauri::tray::TrayIconEvent::Click { button:tauri::tray::MouseButton::Left, button_state:tauri::tray::MouseButtonState::Up,.. } = event {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window,event| {
            if let tauri::WindowEvent::CloseRequested { api,.. } = event {
                let app = window.app_handle();
                if app.state::<Engine>().processes.any_active() {
                    api.prevent_close();
                    let _ = app.emit("close-with-servers",());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![list_servers,get_versions,create_server,pick_import_folder,pick_server_image,set_server_image,clear_server_image,inspect_import,import_server,install_server,accept_eula,eula_accepted,start_server,stop_server,send_command,get_players,manage_player,pick_whitelist_file,import_whitelist,read_log,export_log,get_properties,save_properties,save_memory,save_storage_target,create_backup,list_backups,restore_backup,get_metrics,system_memory,hide_window,stop_all_and_exit])
        .run(tauri::generate_context!())
        .expect("failed to run TCHLauncher");
}
