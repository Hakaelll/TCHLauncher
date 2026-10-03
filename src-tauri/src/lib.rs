mod backup;
mod application;
mod app_log;
mod backup_retention;
mod file_tasks;
mod scheduler;
mod crash_restart;
mod autostart;
mod loader_update;
mod loader_identity;
mod minecraft_update;
mod instance_publication;
mod content;
mod content_storage;
mod download;
mod eula;
mod fabric;
mod neoforge;
mod install;
mod jvm;
mod log;
mod model;
mod modpack;
mod pack_install;
mod network;
mod crash_diagnostics;
mod bore;
mod natpmp;
mod natpmp_manager;
pub mod tunnel;
mod upnp;
mod players;
mod process;
mod ticks;
mod properties;
mod runtime;
mod skin;
mod storage;
mod storage_usage;
mod world;

use model::{Backup, CreateServer, DeleteOutcome, DeletePreview, ImportInspection, ImportServer, LogEvent, Metrics, Server, VersionInfo};
use players::{PlayersSnapshot, WhitelistImport};
use process::ProcessManager;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageSelection { path: String, data: String }

pub struct Engine {
    _instance_lock: std::fs::File,
    scheduler_stopping: std::sync::atomic::AtomicBool,
    router: Arc<upnp::Manager>,
    natpmp:Arc<natpmp_manager::Manager>,
    tunnels:Arc<tunnel::Manager>,
    router_shutdown: Arc<std::sync::atomic::AtomicU8>,
    root: PathBuf,
    processes: ProcessManager,
    metrics_system: Arc<Mutex<sysinfo::System>>,
    sampled_pids: Arc<Mutex<HashSet<u32>>>,
    size_cache: Arc<Mutex<HashMap<String,(Instant,u64)>>>,
    operations: Arc<Mutex<HashSet<String>>>,
    installations: Arc<Mutex<HashMap<String,Arc<download::DownloadTask>>>>,
    content_plans: Arc<Mutex<HashMap<String,content_storage::PreparedPlan>>>,
}

struct OperationGuard {
    id: String,
    operations: Arc<Mutex<HashSet<String>>>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        if let Ok(mut operations) = self.operations.lock() { operations.remove(&self.id); }
    }
}

impl Engine {
    fn start_prepared(&self,id:&str,app:AppHandle)->Result<(),String> {
        if self.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){return Err("La aplicación se está cerrando".into());}
        storage::sync_legacy_icon(&self.root,id)?;
        let server=storage::get(&self.root,id)?;
        content_storage::recover(&server)?;
        eula::apply_recorded(&self.root,&server)?;
        runtime::validate_configured(&self.root,&server)?;
        if self.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){return Err("La aplicación se está cerrando".into());}
        self.processes.start(&server,app,&jvm::get(&self.root,id)?)
    }
    fn reserve_stopped(&self, id: &str) -> Result<OperationGuard, String> {
        let mut operations = self.operations.lock().map_err(|e| e.to_string())?;
        if operations.contains(id) { return Err("Ya hay una operación en curso para este servidor".into()); }
        if self.processes.is_active(id) { return Err("Detén el servidor antes de modificar sus archivos".into()); }
        storage::get(&self.root, id)?;
        operations.insert(id.into());
        Ok(OperationGuard { id: id.into(), operations: self.operations.clone() })
    }

    fn reserve_running(&self, id: &str) -> Result<OperationGuard, String> {
        let mut operations = self.operations.lock().map_err(|e| e.to_string())?;
        if operations.contains(id) { return Err("Ya hay una operación en curso para este servidor".into()); }
        if self.processes.state(id) != "running" { return Err("El servidor debe estar en ejecución para realizar esta acción".into()); }
        operations.insert(id.into());
        Ok(OperationGuard { id: id.into(), operations: self.operations.clone() })
    }

    fn invalidate_size(&self, id: &str) {
        if let Ok(mut cache) = self.size_cache.lock() { cache.remove(id); }
    }
}

#[tauri::command]
fn list_servers(engine: State<'_, Engine>) -> Result<Vec<Server>, String> {
    let mut servers = storage::list(&engine.root)?;
    let installations = engine.installations.lock().map_err(|e| e.to_string())?;
    for server in &mut servers { server.state = if installations.get(&server.id).is_some_and(|task|!task.is_file_operation()) { "installing".into() } else {engine.processes.state(&server.id)}; }
    Ok(servers)
}

#[tauri::command]
async fn get_versions(platform: Option<String>,experimental:Option<bool>) -> Result<Vec<VersionInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || install::platform_versions_with_options(platform.as_deref().unwrap_or("vanilla"),experimental.unwrap_or(false))).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn search_content(engine: State<'_, Engine>, id: String, query: String, offset: u32) -> Result<serde_json::Value, String> {
    let server = storage::get(&engine.root, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let task = download::DownloadTask::new(&server.id, "content_search", |_| {})?;
        content::search(&task, &server, &query, offset)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn plan_content(engine: State<'_, Engine>, id: String, project_id: String, version_id: Option<String>) -> Result<content_storage::ContentReview, String> {
    prepare_content_plan(engine,id,Some(project_id),version_id).await
}

#[tauri::command]
async fn plan_content_updates(engine: State<'_, Engine>, id: String) -> Result<content_storage::ContentReview, String> {
    prepare_content_plan(engine,id,None,None).await
}

async fn prepare_content_plan(engine: State<'_, Engine>, id: String, project_id: Option<String>, version_id: Option<String>) -> Result<content_storage::ContentReview, String> {
    let guard=engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root, &id)?;
    let plans=engine.content_plans.clone();
    let token=tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        content_storage::recover(&server)?;
        let task = download::DownloadTask::new(&server.id, "content_plan", |_| {})?;
        let plan=if let Some(project_id)=project_id {content::plan(&task, &server, &project_id, version_id.as_deref())?} else {content::updates(&task,&server)?};
        content::check_existing(&task,&content_storage::list(&server)?,&plan)?;
        let prepared=content_storage::review(&server,plan)?;
        let token=prepared.review.token.clone();
        let mut plans=plans.lock().map_err(|e|e.to_string())?;
        plans.retain(|_,plan|plan.created.elapsed()<Duration::from_secs(900));
        if plans.len()>=64 {plans.clear();}
        plans.insert(token.clone(),prepared);
        Ok::<_,String>(token)
    }).await.map_err(|e| e.to_string())??;
    // Return a review copy while retaining the authoritative native plan.
    let plans=engine.content_plans.lock().map_err(|e|e.to_string())?;
    Ok(plans.get(&token).ok_or("La revisión ha caducado")?.review.clone())
}

#[tauri::command]
async fn list_content(engine:State<'_,Engine>,id:String)->Result<Vec<content_storage::InstalledContent>,String> {
    let server=storage::get(&engine.root,&id)?;
    tauri::async_runtime::spawn_blocking(move ||content_storage::list(&server)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn content_icon(engine:State<'_,Engine>,url:String)->Result<String,String> {
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move ||content::icon(&root,&url)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn pick_modpack()->Result<Option<modpack::PackInspection>,String> {
    let Some(file)=rfd::AsyncFileDialog::new().add_filter("Modrinth modpack",&["mrpack"]).pick_file().await else {return Ok(None);};
    let path=file.path().to_path_buf();
    tauri::async_runtime::spawn_blocking(move ||modpack::inspect(&path).map(|pack|Some(pack.inspection))).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn inspect_modpack(path:String)->Result<modpack::PackInspection,String> {
    tauri::async_runtime::spawn_blocking(move ||modpack::inspect(std::path::Path::new(&path)).map(|pack|pack.inspection)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn install_modpack(app:AppHandle,engine:State<'_,Engine>,id:String,path:String,archive_sha512:String,selected:Vec<String>)->Result<(),String> {
    let guard=engine.reserve_stopped(&id)?;
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    eula::require_download_consent(&root,&server)?;
    let emitter=app.clone();let task=Arc::new(download::DownloadTask::new(&id,"pack_install",move |progress|{let _=emitter.emit("install-progress",progress);})?);
    engine.installations.lock().map_err(|e|e.to_string())?.insert(id.clone(),task.clone());
    process::emit_state(&app,&id,"installing");let worker=task.clone();
    let result=tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        pack_install::install(&root,&server,std::path::Path::new(&path),&archive_sha512,&selected,&worker)
    }).await.unwrap_or_else(|e|Err(format!("La importación se interrumpió: {e}")));
    task.finish(&result);engine.installations.lock().map_err(|e|e.to_string())?.remove(&id);engine.invalidate_size(&id);
    process::emit_state(&app,&id,if result.is_ok()||result.as_ref().err().is_some_and(|e|e==download::CANCELLED){"stopped"}else{"error"});result
}

#[tauri::command]
async fn remove_content(app:AppHandle,engine:State<'_,Engine>,id:String,filename:String,sha512:String)->Result<(),String> {
    let guard=engine.reserve_stopped(&id)?;
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    let jobs=engine.installations.clone();
    let result=tauri::async_runtime::spawn_blocking(move ||{let _guard=guard;file_tasks::remove_content(&root,&server,&filename,&sha512,&jobs,move|progress|{let _=app.emit("install-progress",progress);})}).await.map_err(|e|e.to_string())?;
    engine.invalidate_size(&id);result
}

#[tauri::command]
async fn install_content(app:AppHandle,engine:State<'_,Engine>,id:String,token:String)->Result<(),String> {
    let _guard=engine.reserve_stopped(&id)?;
    let prepared=engine.content_plans.lock().map_err(|e|e.to_string())?.remove(&token).ok_or("La revisión ha caducado. Revisa los cambios de nuevo.")?;
    if prepared.server_id!=id {return Err("La revisión pertenece a otro servidor".into());}
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    let emitter=app.clone();
    let task=Arc::new(download::DownloadTask::new(&id,"content_install",move |progress|{let _=emitter.emit("install-progress",progress);})?);
    engine.installations.lock().map_err(|e|e.to_string())?.insert(id.clone(),task.clone());
    process::emit_state(&app,&id,"installing");
    let worker=task.clone();
    let result=tauri::async_runtime::spawn_blocking(move ||content_storage::install(&root,&server,&prepared,&worker)).await.unwrap_or_else(|e|Err(e.to_string()));
    task.finish(&result);
    engine.installations.lock().map_err(|e|e.to_string())?.remove(&id);
    engine.invalidate_size(&id);
    process::emit_state(&app,&id,"stopped");
    result
}

#[tauri::command]
async fn content_versions(engine: State<'_, Engine>, id: String, project_id: String) -> Result<Vec<content::ContentVersion>, String> {
    let server = storage::get(&engine.root, &id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let task=download::DownloadTask::new(&server.id,"content_versions",|_| {})?;
        content::versions(&task,&server,&project_id)
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
fn create_server(engine: State<'_, Engine>, input: CreateServer) -> Result<Server, String> {
    storage::create(&engine.root,input)
}

#[tauri::command]
fn set_server_pinned(engine: State<'_, Engine>,id: String,pinned: bool) -> Result<(),String> {
    storage::set_pinned(&engine.root,&id,pinned)
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
fn set_server_image(app:AppHandle,engine: State<'_, Engine>, id: String, path: String) -> Result<Server, String> {
    let _guard=engine.reserve_stopped(&id)?;
    let server=storage::set_image(&engine.root, &id, &PathBuf::from(path))?;
    let _=app.emit("server-artwork-changed",&id);
    Ok(server)
}

#[tauri::command]
fn clear_server_image(engine: State<'_, Engine>, id: String) -> Result<Server, String> {
    let _guard=engine.reserve_stopped(&id)?;
    storage::clear_image(&engine.root, &id)
}

#[tauri::command]
fn preview_delete_server(engine: State<'_, Engine>, id: String) -> Result<DeletePreview, String> {
    storage::delete_preview(&engine.root, &id)
}

#[tauri::command]
async fn delete_server(engine: State<'_, Engine>, id: String) -> Result<DeleteOutcome, String> {
    let guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let deleted_id = id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        storage::delete(&root, &deleted_id)
    }).await.map_err(|e| e.to_string())?;
    if result.is_ok() { engine.invalidate_size(&id); }
    result
}

#[tauri::command]
async fn inspect_import(path: String) -> Result<ImportInspection, String> {
    tauri::async_runtime::spawn_blocking(move || storage::inspect_import(&PathBuf::from(path))).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_server(app:AppHandle,engine: State<'_, Engine>, input: ImportServer) -> Result<Server, String> {
    let root = engine.root.clone();
    let id=uuid::Uuid::new_v4().to_string();
    engine.operations.lock().map_err(|_|"No se pudo reservar la importación")?.insert(id.clone());
    let guard=OperationGuard{id:id.clone(),operations:engine.operations.clone()};
    let jobs=engine.installations.clone();
    tauri::async_runtime::spawn_blocking(move ||{let _guard=guard;file_tasks::import_server(&root,input,&id,&jobs,move|progress|{let _=app.emit("install-progress",progress);})})
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn install_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    run_install(app,engine,id,false,false).await
}

#[tauri::command]
async fn prepare_runtime(app: AppHandle, engine: State<'_, Engine>, id: String,reinstall: bool) -> Result<(),String> {
    run_install(app,engine,id,true,reinstall).await
}

#[tauri::command]
async fn get_runtime_info(engine: State<'_, Engine>,id: String) -> Result<runtime::RuntimeInfo,String> {
    let root = engine.root.clone();
    let server = storage::get(&root,&id)?;
    tauri::async_runtime::spawn_blocking(move ||runtime::inspect(&root,&server)).await.map_err(|error|error.to_string())
}

#[tauri::command]
fn get_jvm_settings(engine:State<'_,Engine>,id:String)->Result<jvm::Settings,String> {
    jvm::get(&engine.root,&id)
}

#[tauri::command]
fn preview_jvm(engine:State<'_,Engine>,id:String,settings:jvm::Settings)->Result<jvm::Preview,String> {
    jvm::preview(&storage::get(&engine.root,&id)?,&settings)
}

#[tauri::command]
async fn save_jvm_settings(engine:State<'_,Engine>,id:String,settings:jvm::Settings)->Result<(),String> {
    let guard=engine.reserve_stopped(&id)?;
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        jvm::save_checked(&root,&id,&settings)
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn pick_custom_runtime(engine: State<'_,Engine>,id:String)->Result<Option<runtime::RuntimeInfo>,String> {
    let _guard=engine.reserve_stopped(&id)?;
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let dialog=rfd::FileDialog::new().set_title("Elegir ejecutable Java");
        #[cfg(windows)] let dialog=dialog.add_filter("Ejecutable Java",&["exe"]);
        let Some(path)=dialog.pick_file() else {return Ok(None);};
        let major=install::required_java(&root,&server.version)?;
        runtime::candidate(&root,&server,&path,major).map(Some)
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn set_custom_runtime(engine: State<'_,Engine>,id:String,path:String)->Result<(),String> {
    let _guard=engine.reserve_stopped(&id)?;
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let major=install::required_java(&root,&server.version)?;
        let info=runtime::candidate(&root,&server,&PathBuf::from(path),major)?;
        storage::update_java(&root,&server.id,&PathBuf::from(info.path.ok_or("Java no devolvió una ruta")?))
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn get_server_cover(engine:State<'_,Engine>,id:String)->Result<Option<String>,String> {
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move||storage::read_cover(&root,&id)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn get_player_head(engine: State<'_, Engine>,uuid: String) -> Result<Option<String>,String> {
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move ||skin::player_head(&root,&uuid)).await.map_err(|e|e.to_string())?
}

async fn run_install(app: AppHandle, engine: State<'_, Engine>, id: String,runtime_only: bool,reinstall: bool) -> Result<(),String> {
    let _guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let server = storage::get(&root,&id)?;
    eula::require_download_consent(&root,&server)?;
    let emitter = app.clone();
    let kind = if !runtime_only {"install"} else if reinstall {"runtime_reinstall"} else {"runtime"};
    let task = Arc::new(download::DownloadTask::new(&id,kind,move |progress| { let _ = emitter.emit("install-progress",progress); })?);
    engine.installations.lock().map_err(|e| e.to_string())?.insert(id.clone(),task.clone());
    task.report("Preparando instalación","preparing");
    process::emit_state(&app,&id,"installing");
    let task_app = app.clone();
    let worker = task.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        if runtime_only {install::prepare_runtime(&root,&server,&worker,reinstall)} else {install::install(&root,&server,&worker)}
    })
        .await.unwrap_or_else(|error| Err(format!("La instalación se interrumpió: {error}")));
    task.finish(&result);
    engine.installations.lock().map_err(|e| e.to_string())?.remove(&id);
    engine.invalidate_size(&id);
    process::emit_state(&task_app,&id,if result.is_ok() || result.as_ref().err().is_some_and(|e| e == download::CANCELLED) {"stopped"} else {"error"});
    result
}

#[tauri::command]
fn cancel_install(engine: State<'_, Engine>, id: String, task_id: String) -> Result<bool,String> {
    let jobs = engine.installations.lock().map_err(|e| e.to_string())?;
    let Some(task) = jobs.get(&id).filter(|task| task.id == task_id) else { return Ok(false); };
    Ok(task.request_cancel())
}

#[tauri::command]
fn get_installation_progress(engine:State<'_,Engine>)->Result<Vec<model::Progress>,String>{
    let jobs=engine.installations.lock().map_err(|_|"No se pudieron consultar las tareas".to_string())?;
    jobs.values().map(|task|task.snapshot()).collect()
}

#[tauri::command]
fn accept_eula(engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let _guard = engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root,&id)?;
    eula::accept(&engine.root,&server)
}

#[tauri::command]
fn eula_accepted(engine: State<'_, Engine>, id: String) -> Result<bool, String> {
    let server = storage::get(&engine.root,&id)?;
    Ok(eula::recorded(&engine.root)? || eula::file_accepted(&server))
}

#[tauri::command]
fn eula_consent_recorded(engine: State<'_,Engine>) -> Result<bool,String> {eula::recorded(&engine.root)}

#[tauri::command]
async fn start_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let guard = engine.reserve_stopped(&id)?;
    let root=engine.root.clone();
    let processes=engine.processes.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        storage::sync_legacy_icon(&root,&id)?;
        let server=storage::get(&root,&id)?;
        content_storage::recover(&server)?;
        eula::apply_recorded(&root,&server)?;
        runtime::validate_configured(&root,&server)?;
        processes.start(&server,app,&jvm::get(&root,&id)?)
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
fn stop_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let operations = engine.operations.lock().map_err(|e| e.to_string())?;
    if operations.contains(&id) { return Err("Espera a que termine la operación en curso".into()); }
    engine.processes.stop(&id,app)
}

#[tauri::command]
async fn restart_server(app: AppHandle, engine: State<'_, Engine>, id: String) -> Result<(), String> {
    let guard = engine.reserve_running(&id)?;
    let processes = engine.processes.clone();
    let root = engine.root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        processes.stop(&id, app.clone())?;
        let deadline = Instant::now() + Duration::from_secs(45);
        while processes.is_active(&id) {
            if Instant::now() >= deadline { return Err("El servidor no se cerró a tiempo. Comprueba su estado antes de volver a iniciarlo".into()); }
            std::thread::sleep(Duration::from_millis(200));
        }
        let server = storage::get(&root, &id)?;
        content_storage::recover(&server)?;
        eula::apply_recorded(&root,&server)?;
        runtime::validate_configured(&root,&server)?;
        processes.start(&server, app,&jvm::get(&root,&id)?)
    }).await.map_err(|e| e.to_string())?
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
    let _guard = engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root,&id)?;
    properties::write(&PathBuf::from(server.directory).join("server.properties"),values)
}

#[tauri::command]
fn save_memory(engine: State<'_, Engine>, id: String, min: u32, max: u32) -> Result<(), String> {
    let _guard = engine.reserve_stopped(&id)?;
    storage::update_memory(&engine.root,&id,min,max)
}

#[tauri::command]
fn save_storage_target(engine: State<'_, Engine>, id: String, gb: u32) -> Result<(), String> {
    storage::update_storage_target(&engine.root, &id, gb)
}

#[tauri::command]
async fn create_backup(app:AppHandle,engine: State<'_, Engine>, id: String) -> Result<Backup, String> {
    let _guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let jobs=engine.installations.clone();let emitter=app.clone();
    tauri::async_runtime::spawn_blocking(move ||file_tasks::create_backup(&root,&id,&jobs,move|progress|{let _=emitter.emit("install-progress",progress);}))
        .await.unwrap_or_else(|error|Err(format!("La copia se interrumpió: {error}")))
}

#[tauri::command]
fn get_backup_policy(engine:State<'_,Engine>,id:String)->Result<backup_retention::Policy,String> {backup_retention::get(&engine.root,&id)}

#[tauri::command]
fn save_backup_policy(engine:State<'_,Engine>,id:String,policy:backup_retention::Policy)->Result<(),String> {
    let _guard=engine.reserve_stopped(&id)?;backup_retention::save(&engine.root,&id,&policy)
}

#[tauri::command]
async fn preview_backup_retention(engine:State<'_,Engine>,id:String)->Result<backup_retention::Plan,String> {
    let guard=engine.reserve_stopped(&id)?;let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move ||{let _guard=guard;backup_retention::recover(&root,&id)?;backup_retention::preview(&root,&id,&[])}).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn apply_backup_retention(engine:State<'_,Engine>,id:String,fingerprint:String)->Result<backup_retention::Plan,String> {
    let guard=engine.reserve_stopped(&id)?;let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move ||{let _guard=guard;backup_retention::apply(&root,&id,Some(&fingerprint),&[])}).await.map_err(|e|e.to_string())?
}

#[tauri::command]
fn list_backups(engine: State<'_, Engine>, id: String) -> Result<Vec<Backup>, String> {
    storage::get(&engine.root,&id)?;
    storage::list_backups(&engine.root,&id)
}

#[tauri::command]
async fn restore_backup(app:AppHandle,engine: State<'_, Engine>, id: String, backup_id: String) -> Result<(), String> {
    let _guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let restored_id = id.clone();
    let emitter=app.clone();let task=Arc::new(download::DownloadTask::new(&id,"restore",move|progress|{let _=emitter.emit("install-progress",progress);})?);
    engine.installations.lock().map_err(|e|e.to_string())?.insert(id.clone(),task.clone());
    let worker=task.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        backup::restore_tracked(&root,&restored_id,&backup_id,&worker)
    }).await.unwrap_or_else(|error|Err(format!("La restauración se interrumpió: {error}")));
    task.finish(&result);
    engine.installations.lock().map_err(|e|e.to_string())?.remove(&id);
    if result.is_ok() { engine.invalidate_size(&id); }
    result
}

#[tauri::command]
async fn list_worlds(engine: State<'_, Engine>, id: String) -> Result<Vec<world::WorldInfo>, String> {
    let server = storage::get(&engine.root, &id)?;
    tauri::async_runtime::spawn_blocking(move || world::list(&server)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pick_world_folder() -> Result<Option<world::WorldInspection>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let Some(path) = rfd::FileDialog::new().pick_folder() else { return Ok(None); };
        world::inspect(&path).map(Some)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn import_world(app: AppHandle, engine: State<'_, Engine>, id: String, path: String, name: String) -> Result<world::WorldInfo, String> {
    let guard = engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root, &id)?;
    let jobs=engine.installations.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _guard=guard;
        file_tasks::import_world(&server, &PathBuf::from(path), &name, &jobs, move|progress|{let _=app.emit("install-progress",progress);})
    }).await.map_err(|e| e.to_string())?;
    if result.is_ok() { engine.invalidate_size(&id); }
    result
}

#[tauri::command]
async fn duplicate_world(app: AppHandle, engine: State<'_, Engine>, id: String, name: String, new_name: String) -> Result<world::WorldInfo, String> {
    let guard = engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root, &id)?;
    let jobs=engine.installations.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        file_tasks::duplicate_world(&server,&name,&new_name,&jobs,move|progress|{let _=app.emit("install-progress",progress);})
    }).await.map_err(|e| e.to_string())?;
    if result.is_ok() { engine.invalidate_size(&id); }
    result
}

#[tauri::command]
async fn export_world(app: AppHandle, engine: State<'_, Engine>, id: String, name: String) -> Result<Option<String>, String> {
    let destination = tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
        .await.map_err(|e| e.to_string())?;
    let Some(destination) = destination else { return Ok(None); };
    let guard = engine.reserve_stopped(&id)?;
    let server = storage::get(&engine.root, &id)?;
    let jobs=engine.installations.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        file_tasks::export_world(&server,&name,&destination,&jobs,move|progress|{let _=app.emit("install-progress",progress);}).map(|path|Some(path.to_string_lossy().into_owned()))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_active_world(app: AppHandle, engine: State<'_, Engine>, id: String, name: String) -> Result<Option<Backup>, String> {
    let guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let server = storage::get(&root, &id)?;
    let jobs=engine.installations.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        file_tasks::set_active_world(&root,&server,&name,&jobs,move|progress|{let _=app.emit("install-progress",progress);})
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn delete_world(app: AppHandle, engine: State<'_, Engine>, id: String, name: String) -> Result<Backup, String> {
    let guard = engine.reserve_stopped(&id)?;
    let root = engine.root.clone();
    let server = storage::get(&root, &id)?;
    let jobs=engine.installations.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        file_tasks::delete_world(&root,&server,&name,&jobs,move|progress|{let _=app.emit("install-progress",progress);})
    }).await.map_err(|e| e.to_string())?;
    if result.is_ok() { engine.invalidate_size(&id); }
    result
}

#[tauri::command]
fn open_world_folder(engine: State<'_, Engine>, id: String, name: String) -> Result<(), String> {
    let server = storage::get(&engine.root, &id)?;
    world::open_folder(&server, &name)
}

#[tauri::command]
fn request_tps(engine:State<'_,Engine>,id:String)->Result<(),String>{engine.processes.request_ticks(&id)}

#[tauri::command]
async fn get_metrics(engine: State<'_, Engine>, id: String) -> Result<Metrics, String> {
    let root=engine.root.clone();
    let server = storage::get(&engine.root,&id)?;
    let pid_uptime = engine.processes.pid_and_uptime(&id);
    let tps=engine.processes.tick_snapshot(&id);
    let metrics_system = engine.metrics_system.clone();
    let sampled_pids = engine.sampled_pids.clone();
    let size_cache = engine.size_cache.clone();
    tauri::async_runtime::spawn_blocking(move || {
    let (memory_bytes,cpu_percent,disk_read_bytes,disk_written_bytes) = if let Some((pid,_)) = pid_uptime {
        let mut system = metrics_system.lock().map_err(|e| e.to_string())?;
        let sys_pid = sysinfo::Pid::from_u32(pid);
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]),true);
        let process = system.process(sys_pid);
        let sampled = sampled_pids.lock().map_err(|e| e.to_string())?.insert(pid);
        let cores = std::thread::available_parallelism().map(|count| count.get()).unwrap_or(1) as f32;
        let disk = process.map(|p| p.disk_usage());
        (process.map(|p| p.memory()),if sampled { None } else { process.map(|p| (p.cpu_usage() / cores).clamp(0.0,100.0)) },disk.map(|d| d.total_read_bytes),disk.map(|d| d.total_written_bytes))
    } else { (None,None,None,None) };
    let sampled_at = chrono::Utc::now().timestamp_millis();
    let cached = size_cache.lock().map_err(|e| e.to_string())?.get(&id).copied()
        .filter(|(when,_)| when.elapsed() < Duration::from_secs(60));
    let size_bytes = if let Some((_,size)) = cached { size } else {
        let size = backup::size(&PathBuf::from(server.directory));
        size_cache.lock().map_err(|e| e.to_string())?.insert(id.clone(), (Instant::now(),size));
        size
    };
    let storage_warning_percent=storage_usage::warning_percent(&root,&id)?;
    Ok(Metrics {server_id:id,sampled_at,pid:pid_uptime.map(|p|p.0),uptime_seconds:pid_uptime.map(|p|p.1),memory_bytes,cpu_percent,disk_read_bytes,disk_written_bytes,size_bytes,storage_warning_percent,tps})
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn system_memory() -> u64 { sysinfo::System::new_all().total_memory() }

#[tauri::command]
async fn get_network_status(engine: State<'_, Engine>, id: String) -> Result<network::NetworkSnapshot, String> {
    let server = storage::get(&engine.root, &id)?;
    let state = engine.processes.state(&id);
    tauri::async_runtime::spawn_blocking(move || network::inspect(&server, &state))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_public_ip() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(network::public_ip)
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_application_logs(engine:State<'_,Engine>)->Result<(),String> {
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move||app_log::open(&root)).await.map_err(|error|error.to_string())?
}

#[tauri::command]
async fn get_application_info(engine:State<'_,Engine>)->Result<application::Info,String>{
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move||application::info(&root)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
async fn get_storage_usage(engine:State<'_,Engine>,id:String)->Result<storage_usage::Usage,String>{
    let root=engine.root.clone();let server=storage::get(&root,&id)?;
    tauri::async_runtime::spawn_blocking(move||storage_usage::inspect(&root,&server)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
fn save_storage_warning(engine:State<'_,Engine>,id:String,percent:u8)->Result<(),String>{storage_usage::save_warning(&engine.root,&id,percent)}

#[tauri::command]
async fn open_application_folder(engine:State<'_,Engine>,key:String,id:Option<String>)->Result<(),String>{
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move||application::open_folder(&root,&key,id.as_deref())).await.map_err(|e|e.to_string())?
}

fn router_target(root:&std::path::Path,processes:&ProcessManager,id:&str)->Result<(std::net::Ipv4Addr,u16),String> {
    if processes.state(id)!="running" {return Err("Inicia el servidor antes de abrir su puerto en el router".into());}
    let server=storage::get(root,id)?;
    let snapshot=network::inspect(&server,"running")?;
    if snapshot.lan_address.is_none() || !matches!(snapshot.port_state,network::PortState::Listening) {
        return Err("El servidor debe responder en una dirección LAN antes de abrir su puerto".into());
    }
    let ip=snapshot.local_ip.and_then(|ip|ip.parse::<std::net::Ipv4Addr>().ok()).filter(|ip|ip.is_private())
        .ok_or("No se detectó una dirección LAN IPv4 privada compatible")?;
    Ok((ip,snapshot.port))
}

#[tauri::command]
async fn get_router_status(engine:State<'_,Engine>,id:String)->Result<upnp::Status,String> {
    storage::get(&engine.root,&id)?;
    let root=engine.root.clone();let router=engine.router.clone();
    tauri::async_runtime::spawn_blocking(move||router.status(&root,&id)).await.map_err(|error|error.to_string())?
}

#[tauri::command]
async fn detect_router(engine:State<'_,Engine>,id:String)->Result<upnp::Status,String> {
    let server=storage::get(&engine.root,&id)?;let state=engine.processes.state(&id);
    let root=engine.root.clone();let router=engine.router.clone();let shutdown=engine.router_shutdown.clone();
    tauri::async_runtime::spawn_blocking(move||{
        if shutdown.load(std::sync::atomic::Ordering::SeqCst)!=0 {return Err("La aplicación se está cerrando".into());}
        let snapshot=network::inspect(&server,&state)?;
        let ip=snapshot.local_ip.and_then(|ip|ip.parse::<std::net::Ipv4Addr>().ok()).filter(|ip|ip.is_private())
            .ok_or("No se detectó una dirección LAN IPv4 privada compatible")?;
        if snapshot.lan_address.is_none(){return Err("El servidor no tiene una dirección LAN compatible".into());}
        router.detect(&root,&id,ip)
    }).await.map_err(|error|error.to_string())?
}

#[tauri::command]
async fn open_router_port(engine:State<'_,Engine>,id:String,confirmed:bool,expected_ip:String,expected_port:u16)->Result<upnp::Status,String> {
    if !confirmed {return Err("Confirma la apertura temporal del puerto antes de continuar".into());}
    let reservation=engine.reserve_running(&id)?;
    let root=engine.root.clone();let router=engine.router.clone();let natpmp=engine.natpmp.clone();let processes=engine.processes.clone();
    let shutdown=engine.router_shutdown.clone();
    tauri::async_runtime::spawn_blocking(move||{
        let _reservation=reservation;
        if shutdown.load(std::sync::atomic::Ordering::SeqCst)!=0 {return Err("La aplicación se está cerrando".into());}
        let (ip,port)=router_target(&root,&processes,&id)?;
        if ip.to_string()!=expected_ip || port!=expected_port {return Err("La dirección o el puerto cambiaron. Revisa la conexión y confirma de nuevo".into());}
        if natpmp.status(&root,&id)?.is_some(){return Err("Retira la apertura NAT-PMP antes de usar UPnP".into());}
        router.open(&root,&id,port,ip,||shutdown.load(std::sync::atomic::Ordering::SeqCst)==0
            && router_target(&root,&processes,&id).is_ok_and(|target|target==(ip,port)))
    }).await.map_err(|error|error.to_string())?
}

#[tauri::command]
async fn close_router_port(engine:State<'_,Engine>,id:String)->Result<upnp::Status,String> {
    storage::get(&engine.root,&id)?;
    let root=engine.root.clone();let router=engine.router.clone();
    tauri::async_runtime::spawn_blocking(move||router.close(&root,&id)).await.map_err(|error|error.to_string())?
}

fn start_router_maintenance(app:AppHandle) {
    std::thread::spawn(move||loop {
        std::thread::sleep(Duration::from_secs(2));
        let engine=app.state::<Engine>();
        if engine.router_shutdown.load(std::sync::atomic::Ordering::SeqCst)!=0 {break;}
        let _=engine.tunnels.maintain(|id,target|engine.router_shutdown.load(std::sync::atomic::Ordering::SeqCst)==0&&tunnel_target(&engine.root,&engine.processes,id).is_ok_and(|current|current==target));
        if let Err(error)=engine.natpmp.maintain(&engine.root,|id,ip,port|
            engine.router_shutdown.load(std::sync::atomic::Ordering::SeqCst)==0&&router_target(&engine.root,&engine.processes,id).is_ok_and(|target|target==(ip,port))){
            app_log::record(&engine.root,app_log::Event::RouterMaintenanceFailed,None);eprintln!("NAT-PMP maintenance: {error}");
        }
        match engine.router.maintain(&engine.root,|id,ip,port|
            engine.router_shutdown.load(std::sync::atomic::Ordering::SeqCst)==0
            && router_target(&engine.root,&engine.processes,id).is_ok_and(|target|target==(ip,port))) {
            Ok(changed)=>{for id in changed {let _=app.emit("router-state",id);}},
            Err(error)=>{app_log::record(&engine.root,app_log::Event::RouterMaintenanceFailed,None);eprintln!("Router maintenance: {error}");},
        }
    });
}

fn tunnel_target(root:&std::path::Path,processes:&ProcessManager,id:&str)->Result<std::net::SocketAddr,String>{
    if processes.state(id)!="running"{return Err("Inicia el servidor antes de activar el túnel".into());}
    let server=storage::get(root,id)?;let snapshot=network::inspect(&server,"running")?;
    if !matches!(snapshot.port_state,network::PortState::Listening){return Err("El servidor todavía no responde".into());}
    let target=snapshot.local_address.parse::<std::net::SocketAddr>().map_err(|_|"El servidor necesita una dirección local IP válida")?;
    tunnel::Request{server_id:id.into(),local_address:target}.validate().map_err(tunnel_error)?;Ok(target)
}
fn tunnel_error(error:tunnel::Error)->String{match error{
    tunnel::Error::Unavailable|tunnel::Error::Temporary=>"No se pudo conectar con Bore. Comprueba tu conexión o inténtalo más tarde",
    tunnel::Error::Busy=>"Ya hay un túnel o una operación pendiente para este servidor",
    tunnel::Error::Stopped=>"El servidor se detuvo o cambió de dirección",
    tunnel::Error::ConfirmationRequired=>"Confirma el uso del túnel antes de continuar",
    tunnel::Error::CleanupRequired=>"No se pudo terminar el túnel. Reintenta su cierre",
    _=>"El túnel no está disponible para este destino",
}.into()}
#[tauri::command]
async fn get_crash_report(engine:State<'_,Engine>,id:String)->Result<Option<crash_diagnostics::Report>,String>{
    let server=storage::get(&engine.root,&id)?;
    if engine.processes.state(&id)!="crashed"{return Ok(None);}
    let Some(session)=engine.processes.exit_token(&id)else{return Ok(None);};
    tauri::async_runtime::spawn_blocking(move||crash_diagnostics::load(std::path::Path::new(&server.directory),&id,&session)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
fn get_tunnel_status(engine:State<'_,Engine>,id:String)->Result<Option<tunnel::Status>,String>{storage::get(&engine.root,&id)?;engine.tunnels.status(&id).map_err(tunnel_error)}
#[tauri::command]
async fn open_tunnel(engine:State<'_,Engine>,id:String,confirmed:bool,expected_address:String)->Result<tunnel::Status,String>{
    if !confirmed{return Err(tunnel_error(tunnel::Error::ConfirmationRequired));}
    let reservation=engine.reserve_running(&id)?;let root=engine.root.clone();let processes=engine.processes.clone();let tunnels=engine.tunnels.clone();let shutdown=engine.router_shutdown.clone();
    tauri::async_runtime::spawn_blocking(move||{
        let _reservation=reservation;let target=tunnel_target(&root,&processes,&id)?;
        if target.to_string()!=expected_address{return Err("La dirección cambió. Revisa y confirma de nuevo".into());}
        tunnels.connect("bore",&tunnel::Request{server_id:id.clone(),local_address:target},true,||shutdown.load(std::sync::atomic::Ordering::SeqCst)==0&&tunnel_target(&root,&processes,&id).is_ok_and(|current|current==target)).map_err(tunnel_error)
    }).await.map_err(|error|error.to_string())?
}
#[tauri::command]
async fn close_tunnel(engine:State<'_,Engine>,id:String)->Result<(),String>{
    storage::get(&engine.root,&id)?;let tunnels=engine.tunnels.clone();
    tauri::async_runtime::spawn_blocking(move||tunnels.disconnect(&id).map_err(tunnel_error)).await.map_err(|error|error.to_string())?
}

#[tauri::command]
async fn get_natpmp_status(engine:State<'_,Engine>,id:String)->Result<Option<natpmp_manager::Record>,String>{
    storage::get(&engine.root,&id)?;let root=engine.root.clone();let manager=engine.natpmp.clone();
    tauri::async_runtime::spawn_blocking(move||manager.status(&root,&id)).await.map_err(|error|error.to_string())?
}
#[derive(serde::Serialize)]
#[serde(rename_all="camelCase")]
struct NatPmpDetection{gateway:String,external_ip:String,public_address:bool}
#[tauri::command]
async fn detect_natpmp(engine:State<'_,Engine>,id:String,gateway:Option<String>)->Result<NatPmpDetection,String>{
    let server=storage::get(&engine.root,&id)?;let state=engine.processes.state(&id);let shutdown=engine.router_shutdown.clone();
    tauri::async_runtime::spawn_blocking(move||{
        if shutdown.load(std::sync::atomic::Ordering::SeqCst)!=0{return Err("La aplicación se está cerrando".into());}
        let network=network::inspect(&server,&state)?;
        let local=network.local_ip.and_then(|ip|ip.parse::<std::net::Ipv4Addr>().ok()).filter(|ip|ip.is_private()).ok_or("No se detectó una dirección LAN IPv4 compatible")?;
        let gateway=match gateway{Some(value)=>value.parse::<std::net::Ipv4Addr>().map_err(|_|"La dirección IPv4 del router no es válida")?,None=>natpmp::default_gateway(local)?};
        let external_ip=natpmp_manager::Manager::detect(local,gateway)?;
        let public_address=external_ip.parse().is_ok_and(upnp::public_ipv4);
        Ok(NatPmpDetection{gateway:gateway.to_string(),external_ip,public_address})
    }).await.map_err(|error|error.to_string())?
}
#[tauri::command]
async fn open_natpmp(engine:State<'_,Engine>,id:String,gateway:String,confirmed:bool,expected_ip:String,expected_port:u16)->Result<natpmp_manager::Record,String>{
    if !confirmed{return Err("Confirma la apertura NAT-PMP antes de continuar".into());}
    let reservation=engine.reserve_running(&id)?;let root=engine.root.clone();let manager=engine.natpmp.clone();let router=engine.router.clone();let processes=engine.processes.clone();let shutdown=engine.router_shutdown.clone();
    tauri::async_runtime::spawn_blocking(move||{
        let _reservation=reservation;
        let gateway=gateway.parse::<std::net::Ipv4Addr>().map_err(|_|"La dirección IPv4 del router no es válida")?;
        let (ip,port)=router_target(&root,&processes,&id)?;
        if ip.to_string()!=expected_ip||port!=expected_port{return Err("La dirección o el puerto cambiaron. Confirma de nuevo la apertura".into());}
        if router.status(&root,&id)?.has_mapping(){return Err("Retira la apertura UPnP antes de usar NAT-PMP".into());}
        manager.open(&root,&id,ip,gateway,port,||shutdown.load(std::sync::atomic::Ordering::SeqCst)==0&&router_target(&root,&processes,&id).is_ok_and(|target|target==(ip,port)))
    }).await.map_err(|error|error.to_string())?
}
#[tauri::command]
async fn close_natpmp(engine:State<'_,Engine>,id:String)->Result<(),String>{
    storage::get(&engine.root,&id)?;let root=engine.root.clone();let manager=engine.natpmp.clone();
    tauri::async_runtime::spawn_blocking(move||manager.close(&root,&id)).await.map_err(|error|error.to_string())?
}

#[tauri::command]
fn hide_window(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main").ok_or("No se encontró la ventana")?
        .hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_all_and_exit(app: AppHandle, engine: State<'_, Engine>) -> Result<(), String> {
    engine.scheduler_stopping.store(true,std::sync::atomic::Ordering::SeqCst);
    let ids = engine.processes.active_ids();
    for id in &ids { if let Err(error)=engine.processes.stop(id,app.clone()) {engine.scheduler_stopping.store(false,std::sync::atomic::Ordering::SeqCst);return Err(error);} }
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

#[tauri::command]
fn list_scheduled_tasks(engine:State<'_,Engine>,id:String)->Result<Vec<scheduler::Task>,String> {scheduler::list(&engine.root,&id)}
#[tauri::command]
fn get_crash_restart(engine:State<'_,Engine>,id:String)->Result<crash_restart::Status,String> {crash_restart::status(&engine.root,&id)}
#[tauri::command]
fn get_os_autostart()->Result<autostart::OsStatus,String>{autostart::os_status()}
#[tauri::command]
async fn update_minecraft(app:AppHandle,engine:State<'_,Engine>,id:String,target:String,fingerprint:String,confirmed_warnings:bool)->Result<Option<String>,String> {
    let _guard=engine.reserve_stopped(&id)?;
    let server=storage::get(&engine.root,&id)?;eula::require_download_consent(&engine.root,&server)?;
    let emitter=app.clone();let task=Arc::new(download::DownloadTask::new(&id,"minecraft_update",move|progress|{let _=emitter.emit("install-progress",progress);})?);
    engine.installations.lock().map_err(|e|e.to_string())?.insert(id.clone(),task.clone());
    task.report("Revisando el cambio de Minecraft","preparing");process::emit_state(&app,&id,"installing");
    let root=engine.root.clone();let worker=task.clone();let server_id=id.clone();
    let result=tauri::async_runtime::spawn_blocking(move||minecraft_update::update(&root,&server_id,&target,&fingerprint,confirmed_warnings,&worker)).await.unwrap_or_else(|e|Err(format!("El cambio de Minecraft se interrumpió: {e}")));
    task.finish(&result.as_ref().map(|_|()).map_err(Clone::clone));
    engine.installations.lock().map_err(|e|e.to_string())?.remove(&id);engine.invalidate_size(&id);process::emit_state(&app,&id,"stopped");result
}
#[tauri::command]
async fn review_minecraft_update(engine:State<'_,Engine>,id:String,target:String)->Result<minecraft_update::Review,String> {
    let guard=engine.reserve_stopped(&id)?;
    let server=storage::get(&engine.root,&id)?;
    let root=engine.root.clone();
    tauri::async_runtime::spawn_blocking(move||{
        let _guard=guard;
        let task=download::DownloadTask::new(&server.id,"minecraft_review",|_|{})?;
        minecraft_update::review(&root,&server,&target,&task)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn review_loader_update(engine:State<'_,Engine>,id:String)->Result<loader_update::Review,String> {
    let guard=engine.reserve_stopped(&id)?;let server=storage::get(&engine.root,&id)?;
    tauri::async_runtime::spawn_blocking(move||{let _guard=guard;loader_update::review(&server,&download::DownloadTask::new(&server.id,"loader_review",|_|{})?)}).await.map_err(|e|e.to_string())?
}
#[tauri::command]
async fn update_loader(app:AppHandle,engine:State<'_,Engine>,id:String,target:String,fingerprint:String)->Result<Option<String>,String> {
    let _guard=engine.reserve_stopped(&id)?;let server=storage::get(&engine.root,&id)?;eula::require_download_consent(&engine.root,&server)?;
    let emitter=app.clone();let task=Arc::new(download::DownloadTask::new(&id,"loader_update",move|progress|{let _=emitter.emit("install-progress",progress);})?);
    engine.installations.lock().map_err(|e|e.to_string())?.insert(id.clone(),task.clone());task.report("Preparando actualización del loader","preparing");process::emit_state(&app,&id,"installing");
    let root=engine.root.clone();let worker=task.clone();let server_id=id.clone();
    let result=tauri::async_runtime::spawn_blocking(move||loader_update::update(&root,&server_id,&target,&fingerprint,&worker)).await.unwrap_or_else(|e|Err(format!("La actualización se interrumpió: {e}")));
    task.finish(&result.as_ref().map(|_|()).map_err(Clone::clone));engine.installations.lock().map_err(|e|e.to_string())?.remove(&id);engine.invalidate_size(&id);process::emit_state(&app,&id,"stopped");result
}
#[tauri::command]
fn set_os_autostart(enabled:bool)->Result<autostart::OsStatus,String>{autostart::set_os(enabled)}
#[tauri::command]
fn get_startup_servers(engine:State<'_,Engine>)->Result<Vec<autostart::ServerOption>,String>{autostart::servers(&engine.root)}
#[tauri::command]
fn set_startup_server(engine:State<'_,Engine>,id:String,enabled:bool)->Result<(),String>{
    let operations=engine.operations.lock().map_err(|e|e.to_string())?;
    if operations.contains(&id){return Err("Espera a que termine la operación actual para cambiar el inicio automático".into());}
    autostart::set_server(&engine.root,&id,enabled)
}
#[tauri::command]
fn save_crash_restart(engine:State<'_,Engine>,id:String,policy:crash_restart::Policy)->Result<(),String> {
    let operations=engine.operations.lock().map_err(|e|e.to_string())?;
    if operations.contains(&id){return Err("Espera a que termine la operación actual para cambiar el reinicio automático".into());}
    crash_restart::save(&engine.root,&id,&policy,engine.processes.exit_token(&id).as_deref())
}
#[tauri::command]
fn add_scheduled_task(engine:State<'_,Engine>,id:String,input:scheduler::Input)->Result<scheduler::Task,String> {scheduler::add(&engine.root,&id,input,chrono::Utc::now().timestamp_millis())}
#[tauri::command]
fn set_scheduled_task_enabled(engine:State<'_,Engine>,id:String,task_id:String,enabled:bool)->Result<(),String> {scheduler::set_enabled(&engine.root,&id,&task_id,enabled,chrono::Utc::now().timestamp_millis())}
#[tauri::command]
fn remove_scheduled_task(engine:State<'_,Engine>,id:String,task_id:String)->Result<(),String> {scheduler::remove(&engine.root,&id,&task_id)}

fn execute_scheduled(app:&AppHandle,task:&scheduler::Task)->Result<String,String> {
    let engine=app.state::<Engine>();let id=&task.server_id;
    if engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){return Err("La aplicación se está cerrando".into());}
    let active=engine.processes.is_active(id);
    let _guard=if active {engine.reserve_running(id)?}else{engine.reserve_stopped(id)?};
    match task.action.as_str() {
        "start"=>{if active{return Ok("Sin cambios: el servidor ya está en ejecución".into());}engine.start_prepared(id,app.clone())?;Ok("Inicio solicitado".into())},
        "stop"=>{if !active{return Ok("Sin cambios: el servidor ya está detenido".into());}engine.processes.stop(id,app.clone())?;wait_scheduled_stop(&engine,id)?;Ok("Servidor detenido".into())},
        "restart"|"backup_restart"=>{
            if task.action=="restart"&&!active{return Err("No se reinició: el servidor estaba detenido".into());}
            if active {engine.processes.stop(id,app.clone())?;wait_scheduled_stop(&engine,id)?;}
            let copy=if task.action=="backup_restart" {let emitter=app.clone();Some(file_tasks::create_backup(&engine.root,id,&engine.installations,move|progress|{let _=emitter.emit("install-progress",progress);}))}else{None};
            // Resume a server explicitly stopped for a backup even if copying
            // failed. Report both failures if its restart also fails.
            let resumed=if active {engine.start_prepared(id,app.clone())}else{Ok(())};
            match (copy,resumed) {
                (Some(Err(copy)),Err(start))=>Err(format!("Copia: {copy}. Reinicio: {start}")),
                (Some(Err(copy)),Ok(())) if copy==download::CANCELLED=>Ok(if active{"Copia cancelada; inicio solicitado".into()}else{"Copia cancelada; servidor detenido".into()}),
                (Some(Err(copy)),Ok(()))=>Err(format!("Copia: {copy}. {}",if active{"Inicio solicitado de nuevo."}else{"El servidor permanece detenido."})),
                (_,Err(start))=>Err(format!("No se pudo volver a iniciar: {start}")),
                (Some(Ok(copy)),Ok(()))=>Ok(copy.retention_warning.unwrap_or_else(||if active{"Copia guardada; inicio solicitado".into()}else{"Copia guardada".into()})),
                (None,Ok(()))=>Ok("Reinicio solicitado".into()),
            }
        },
        "backup"=>{
            if active{return Err("Copia omitida: el servidor estaba en ejecución. Usa «Copia con parada y reinicio» si permites una interrupción.".into());}
            let emitter=app.clone();let copy=match file_tasks::create_backup(&engine.root,id,&engine.installations,move|progress|{let _=emitter.emit("install-progress",progress);}){
                Err(error) if error==download::CANCELLED=>return Ok("Copia cancelada; servidor detenido".into()),
                value=>value?,
            };
            Ok(copy.retention_warning.unwrap_or_else(||"Copia guardada".into()))
        },
        _=>Err("Acción programada no válida".into()),
    }
}
fn wait_scheduled_stop(engine:&Engine,id:&str)->Result<(),String> {
    let deadline=Instant::now()+Duration::from_secs(45);
    while engine.processes.is_active(id) {
        if Instant::now()>=deadline{return Err("El servidor no se detuvo a tiempo".into());}
        std::thread::sleep(Duration::from_millis(200));
    }
    if engine.processes.state(id)!="stopped" {return Err("El proceso no se cerró correctamente; revisa la consola antes de continuar".into());}Ok(())
}
fn start_scheduler(app:AppHandle) {
    std::thread::spawn(move||{
        let workers=Arc::new(std::sync::atomic::AtomicUsize::new(0));
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let engine=app.state::<Engine>();
            if engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){continue;}
            for (id,exit) in engine.processes.exits() {
                if exit.unexpected&&!engine.processes.is_active(&id) {
                    match crash_restart::observe(&engine.root,&id,&exit.token,chrono::Utc::now().timestamp_millis()) {Ok(true)=>{let _=app.emit("scheduler-updated",&id);},Ok(false)=>(),Err(error)=>eprintln!("Crash could not be recorded: {error}")}
                }
            }
            for id in engine.processes.active_ids() {
                let healthy=engine.processes.healthy_for(&id,600);
                if let Ok(true)=crash_restart::active(&engine.root,&id,healthy){let _=app.emit("scheduler-updated",&id);}
            }
            if workers.load(std::sync::atomic::Ordering::SeqCst)>=4 {continue;}
            match crash_restart::claim(&engine.root,chrono::Utc::now().timestamp_millis()) {
                Ok(Some(claim))=>{
                    workers.fetch_add(1,std::sync::atomic::Ordering::SeqCst);let workers=workers.clone();let app=app.clone();
                    let _=app.emit("scheduler-updated",&claim.server_id);
                    std::thread::spawn(move||{
                        let engine=app.state::<Engine>();
                        let result=(||->Result<bool,String>{
                            if !crash_restart::allowed(&engine.root,&claim)?||engine.processes.exit_token(&claim.server_id).as_deref()!=Some(&claim.token){crash_restart::cancel(&engine.root,&claim)?;return Ok(false);}
                            if engine.operations.lock().map_err(|e|e.to_string())?.contains(&claim.server_id){crash_restart::defer(&engine.root,&claim,chrono::Utc::now().timestamp_millis())?;return Ok(false);}
                            let _guard=engine.reserve_stopped(&claim.server_id)?;
                            if !crash_restart::allowed(&engine.root,&claim)?||engine.processes.exit_token(&claim.server_id).as_deref()!=Some(&claim.token){crash_restart::cancel(&engine.root,&claim)?;return Ok(false);}
                            engine.start_prepared(&claim.server_id,app.clone())?;Ok(true)
                        })();
                        if !matches!(&result,Ok(false)) {
                            if let Err(error)=crash_restart::finish(&engine.root,&claim,result.map(|_|()),chrono::Utc::now().timestamp_millis()){eprintln!("Crash restart result could not be stored: {error}");}
                        }
                        let _=app.emit("scheduler-updated",&claim.server_id);
                        workers.fetch_sub(1,std::sync::atomic::Ordering::SeqCst);
                    });
                },
                Ok(None)=>(),Err(error)=>eprintln!("Crash restart could not be claimed: {error}"),
            }
            if workers.load(std::sync::atomic::Ordering::SeqCst)>=4 {continue;}
            match scheduler::claim(&engine.root,chrono::Utc::now().timestamp_millis()) {
                Ok(Some(task))=>{
                    workers.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
                    let workers=workers.clone();let app=app.clone();
                    let _=app.emit("scheduler-updated",&task.server_id);
                    std::thread::spawn(move||{
                        let result=execute_scheduled(&app,&task).unwrap_or_else(|error|format!("Error: {error}"));
                        if let Err(error)=scheduler::finish(&app.state::<Engine>().root,&task.id,&result){eprintln!("Scheduler result could not be stored: {error}");}
                        let _=app.emit("scheduler-updated",&task.server_id);
                        workers.fetch_sub(1,std::sync::atomic::Ordering::SeqCst);
                    });
                },
                Ok(None)=>(),
                Err(error)=>eprintln!("Scheduler could not claim a task: {error}"),
            }
        }
    });
}

fn start_selected_servers(app:AppHandle) {
    std::thread::spawn(move||{
        let engine=app.state::<Engine>();
        let selected=match autostart::servers(&engine.root){Ok(items)=>items,Err(error)=>{eprintln!("Startup preferences could not be read: {error}");return;}};
        for item in selected.into_iter().filter(|item|item.enabled) {
            if engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){break;}
            let result=(||->Result<String,String>{
                if engine.processes.is_active(&item.id){return Ok("Servidor ya activo; no se duplica el inicio".into());}
                let _guard=engine.reserve_stopped(&item.id)?;
                if !autostart::enabled(&engine.root,&item.id)?{return Ok("Inicio cancelado: se desactivó la preferencia".into());}
                engine.start_prepared(&item.id,app.clone())?;Ok("Inicio automático solicitado".into())
            })();
            let message=match result {Ok(message)=>message,Err(error)=>{if !engine.scheduler_stopping.load(std::sync::atomic::Ordering::SeqCst){show_main(&app);}format!("Error: {error}")}};
            if let Err(error)=autostart::finish(&engine.root,&item.id,&message){eprintln!("Startup result could not be saved: {error}");}
            let _=app.emit("startup-result",(&item.name,&message));
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(windows)]
            if let Some(window)=app.get_webview_window("main") {
                window.with_webview(|webview| {
                    use windows_core::Interface;
                    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings4;
                    // WebView2 settings must be accessed on the owning UI thread.
                    let result=unsafe {(|| -> windows_core::Result<()> {
                        let settings=webview.controller().CoreWebView2()?.Settings()?.cast::<ICoreWebView2Settings4>()?;
                        settings.SetIsGeneralAutofillEnabled(false)?;
                        settings.SetIsPasswordAutosaveEnabled(false)
                    })()};
                    if let Err(error)=result {eprintln!("Could not disable WebView autofill: {error}");}
                })?;
            }
            let root = app.path().app_data_dir()?;
            std::fs::create_dir_all(&root)?;
            let instance_lock=scheduler::exclusive_owner(&root).map_err(std::io::Error::other)?;
            app_log::record(&root,app_log::Event::ApplicationStarted,None);
            storage::connect(&root).map_err(std::io::Error::other)?;
            instance_publication::recover_all(&root).map_err(std::io::Error::other)?;
            backup::recover_all(&root).map_err(std::io::Error::other)?;
            pack_install::recover_all(&root).map_err(std::io::Error::other)?;
            loader_update::recover_all(&root).map_err(std::io::Error::other)?;
            scheduler::recover(&root).map_err(std::io::Error::other)?;
            crash_restart::recover(&root).map_err(std::io::Error::other)?;
            app.manage(Engine { _instance_lock:instance_lock,router:Arc::new(upnp::Manager::default()),natpmp:Arc::new(natpmp_manager::Manager::default()),tunnels:Arc::new(tunnel::Manager::new(vec![Arc::new(bore::Provider::default())]).expect("valid built-in tunnel provider")),router_shutdown:Arc::new(std::sync::atomic::AtomicU8::new(0)),scheduler_stopping:std::sync::atomic::AtomicBool::new(false),root,processes:ProcessManager::default(),metrics_system:Arc::new(Mutex::new(sysinfo::System::new())),sampled_pids:Arc::new(Mutex::new(HashSet::new())),size_cache:Arc::new(Mutex::new(HashMap::new())),operations:Arc::new(Mutex::new(HashSet::new())),installations:Arc::new(Mutex::new(HashMap::new())),content_plans:Arc::new(Mutex::new(HashMap::new())) });
            start_scheduler(app.handle().clone());
            start_router_maintenance(app.handle().clone());
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
            if std::env::args_os().any(|argument|argument=="--autostart") {
                if let Some(window)=app.get_webview_window("main"){let _=window.hide();}
            }
            start_selected_servers(app.handle().clone());
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
        .invoke_handler(tauri::generate_handler![get_crash_report,get_tunnel_status,open_tunnel,close_tunnel,get_natpmp_status,detect_natpmp,open_natpmp,close_natpmp,request_tps,get_storage_usage,save_storage_warning,get_application_info,open_application_folder,update_minecraft,review_minecraft_update,review_loader_update,update_loader,get_os_autostart,set_os_autostart,get_startup_servers,set_startup_server,get_crash_restart,save_crash_restart,list_scheduled_tasks,add_scheduled_task,set_scheduled_task_enabled,remove_scheduled_task,get_backup_policy,save_backup_policy,preview_backup_retention,apply_backup_retention,install_modpack,pick_modpack,inspect_modpack,remove_content,plan_content_updates,content_icon,list_content,install_content,content_versions,search_content,plan_content,list_servers,set_server_pinned,get_versions,create_server,pick_import_folder,pick_server_image,set_server_image,clear_server_image,preview_delete_server,delete_server,inspect_import,import_server,install_server,cancel_install,get_installation_progress,prepare_runtime,get_runtime_info,get_jvm_settings,preview_jvm,save_jvm_settings,pick_custom_runtime,set_custom_runtime,get_player_head,get_server_cover,accept_eula,eula_accepted,eula_consent_recorded,start_server,stop_server,restart_server,send_command,get_players,manage_player,pick_whitelist_file,import_whitelist,read_log,export_log,get_properties,save_properties,save_memory,save_storage_target,create_backup,list_backups,restore_backup,list_worlds,pick_world_folder,import_world,duplicate_world,export_world,set_active_world,delete_world,open_world_folder,get_metrics,system_memory,get_network_status,get_public_ip,open_application_logs,get_router_status,detect_router,open_router_port,close_router_port,hide_window,stop_all_and_exit])
        .build(tauri::generate_context!())
        .expect("failed to build TCHLauncher")
        .run(|app,event| {
            if let tauri::RunEvent::ExitRequested {api,code,..}=event {
                let engine=app.state::<Engine>();
                let phase=engine.router_shutdown.load(std::sync::atomic::Ordering::SeqCst);
                if phase==2 {return;}
                api.prevent_exit();
                if engine.router_shutdown.compare_exchange(0,1,std::sync::atomic::Ordering::SeqCst,std::sync::atomic::Ordering::SeqCst).is_ok() {
                    engine.scheduler_stopping.store(true,std::sync::atomic::Ordering::SeqCst);
                    app_log::record(&engine.root,app_log::Event::ApplicationStopping,None);
                    let router=engine.router.clone();let natpmp=engine.natpmp.clone();let tunnels=engine.tunnels.clone();let root=engine.root.clone();let shutdown=engine.router_shutdown.clone();
                    let app=app.clone();
                    std::thread::spawn(move||{
                        let _=tunnels.disconnect_all();
                        router.close_all(&root);
                        natpmp.close_all(&root);
                        shutdown.store(2,std::sync::atomic::Ordering::SeqCst);
                        app.exit(code.unwrap_or(0));
                    });
                }
            }
        });
}
