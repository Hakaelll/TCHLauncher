use crate::model::{Backup, CreateServer, DeleteOutcome, DeletePreview, ImportInspection, ImportServer, Server};
use base64::Engine;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;

pub fn connect(root: &Path) -> Result<Connection, String> {
    let db_dir = root.join("database");
    std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;
    let db = Connection::open(db_dir.join("tchlauncher.sqlite")).map_err(|e| e.to_string())?;
    db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
      CREATE TABLE IF NOT EXISTS servers (
        id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
        platform TEXT NOT NULL, directory TEXT NOT NULL UNIQUE,
        jar_path TEXT, java_path TEXT, memory_min_mb INTEGER NOT NULL,
        memory_max_mb INTEGER NOT NULL, created_at TEXT NOT NULL,
        storage_target_gb INTEGER NOT NULL DEFAULT 20
      );
      CREATE TABLE IF NOT EXISTS backups (
        id TEXT PRIMARY KEY, server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
        created_at TEXT NOT NULL, size_bytes INTEGER NOT NULL, path TEXT NOT NULL
      );")
        .map_err(|e| e.to_string())?;
    let schema_version: u32 = db.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if schema_version < 1 {
        db.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
        let migration = (|| -> Result<(), String> {
            let mut columns = db.prepare("PRAGMA table_info(servers)").map_err(|e| e.to_string())?;
            let names = columns.query_map([], |row| row.get::<_, String>(1)).map_err(|e| e.to_string())?;
            let names = names.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
            if !names.iter().any(|name| name == "storage_target_gb") {
                db.execute("ALTER TABLE servers ADD COLUMN storage_target_gb INTEGER NOT NULL DEFAULT 20", []).map_err(|e| e.to_string())?;
            }
            db.execute_batch("PRAGMA user_version=1").map_err(|e| e.to_string())
        })();
        if migration.is_ok() { db.execute_batch("COMMIT").map_err(|e| e.to_string())?; }
        else { let _ = db.execute_batch("ROLLBACK"); migration?; }
    }
    if schema_version < 2 {
        let mut columns=db.prepare("PRAGMA table_info(servers)").map_err(|e|e.to_string())?;
        let names=columns.query_map([],|row|row.get::<_,String>(1)).map_err(|e|e.to_string())?.collect::<rusqlite::Result<Vec<_>>>().map_err(|e|e.to_string())?;
        if !names.iter().any(|name|name=="pinned") {db.execute("ALTER TABLE servers ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0",[]).map_err(|e|e.to_string())?;}
        db.execute_batch("PRAGMA user_version=2").map_err(|e|e.to_string())?;
    }
    db.execute_batch("CREATE TABLE IF NOT EXISTS jvm_settings (
      server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE,
      mode TEXT NOT NULL CHECK(mode IN ('recommended','custom')),
      arguments TEXT NOT NULL DEFAULT ''
    );").map_err(|e|e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS application_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);").map_err(|e|e.to_string())?;
    Ok(db)
}

fn row_server(row: &rusqlite::Row<'_>) -> rusqlite::Result<Server> {
    Ok(Server {
        id: row.get(0)?, name: row.get(1)?, version: row.get(2)?,
        platform: row.get(3)?, directory: row.get(4)?, jar_path: row.get(5)?,
        java_path: row.get(6)?, memory_min_mb: row.get(7)?,
        memory_max_mb: row.get(8)?, created_at: row.get(9)?, storage_target_gb: row.get(10)?,
        state: "stopped".into(), image_data: None,pinned:row.get::<_,i64>(11)?!=0,
    })
}

const SERVER_SELECT: &str = "SELECT id,name,version,platform,directory,jar_path,java_path,memory_min_mb,memory_max_mb,created_at,storage_target_gb,pinned FROM servers";

pub fn list(root: &Path) -> Result<Vec<Server>, String> {
    let db = connect(root)?;
    let mut stmt = db.prepare(&format!("{SERVER_SELECT} ORDER BY pinned DESC,created_at DESC,id"))
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], row_server).map_err(|e| e.to_string())?;
    let mut servers = rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
    for server in &mut servers { server.image_data = read_image(root, &server.id); }
    Ok(servers)
}

pub fn set_pinned(root: &Path,id: &str,pinned: bool) -> Result<(),String> {
    get(root,id)?;
    connect(root)?.execute("UPDATE servers SET pinned=?2 WHERE id=?1",rusqlite::params![id,if pinned {1} else {0}]).map_err(|e|e.to_string())?;
    Ok(())
}

pub fn get(root: &Path, id: &str) -> Result<Server, String> {
    let db = connect(root)?;
    db.query_row(&format!("{SERVER_SELECT} WHERE id=?1"), [id], row_server)
        .optional().map_err(|e| e.to_string())?
        .ok_or_else(|| "No se encontró el servidor".to_string())
}

fn managed_directory(root: &Path, id: &str, server: &Server) -> bool {
    Path::new(&server.directory) == root.join("servers").join(id)
}

pub fn delete_preview(root: &Path, id: &str) -> Result<DeletePreview, String> {
    Uuid::parse_str(id).map_err(|_| "Identificador de servidor no válido".to_string())?;
    let server = get(root, id)?;
    Ok(DeletePreview {
        managed_files: managed_directory(root, id, &server),
        directory: server.directory.clone(),
        backup_count: list_backups(root, id)?.len(),
    })
}

struct StagedDeletion { original: PathBuf, staged: PathBuf, directory: bool }

fn rollback_deletion(items: &[StagedDeletion]) -> Vec<String> {
    items.iter().rev().filter_map(|item| fs::rename(&item.staged, &item.original).err()
        .map(|error| format!("{}: {error}", item.original.display()))).collect()
}

pub fn delete(root: &Path, id: &str) -> Result<DeleteOutcome, String> {
    Uuid::parse_str(id).map_err(|_| "Identificador de servidor no válido".to_string())?;
    let server = get(root, id)?;
    let managed = managed_directory(root, id, &server);
    let db = connect(root)?;
    let root_canonical = root.canonicalize().map_err(|e| e.to_string())?;
    let server_path = root.join("servers").join(id);
    let backup_path = root.join("backups").join(id);
    let image_path = root.join("server-images").join(id);
    let mut candidates = Vec::new();
    if managed { candidates.push((server_path, true)); }
    candidates.push((backup_path, true));
    candidates.push((image_path, false));
    candidates.push((root.join("server-covers").join(id), false));
    let mut staged = Vec::new();
    for (source, directory) in candidates {
        let result = (|| -> Result<(), String> {
            let meta = match fs::symlink_metadata(&source) {
                Ok(meta) => meta,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(error.to_string()),
            };
            if meta.file_type().is_symlink() || meta.is_dir() != directory {
                return Err(format!("Ruta inesperada: {}", source.display()));
            }
            let parent = source.parent().ok_or("Ruta de datos no válida")?.canonicalize().map_err(|e| e.to_string())?;
            let actual = source.canonicalize().map_err(|e| e.to_string())?;
            if !parent.starts_with(&root_canonical) || actual.parent() != Some(parent.as_path()) {
                return Err(format!("La ruta sale de los datos de la aplicación: {}", source.display()));
            }
            let moved = source.with_file_name(format!(".tch-deleting-{}", Uuid::new_v4()));
            fs::rename(&source, &moved).map_err(|e| format!("No se pudo preparar la eliminación: {e}"))?;
            staged.push(StagedDeletion { original: source, staged: moved, directory });
            Ok(())
        })();
        if let Err(error) = result {
            let rollback = rollback_deletion(&staged);
            return Err(if rollback.is_empty() { error } else { format!("{error}. No se pudo restaurar: {}", rollback.join("; ")) });
        }
    }
    match db.execute("DELETE FROM servers WHERE id=?1", [id]) {
        Ok(1) => (),
        result => {
            let error = result.err().map(|e| e.to_string()).unwrap_or_else(|| "El servidor ya no estaba registrado".into());
            let rollback = rollback_deletion(&staged);
            return Err(if rollback.is_empty() { error } else { format!("{error}. No se pudo restaurar: {}", rollback.join("; ")) });
        }
    }
    let cleanup_warnings = staged.into_iter().filter_map(|item| {
        let result = if item.directory { fs::remove_dir_all(&item.staged) } else { fs::remove_file(&item.staged) };
        result.err().map(|error| format!("{}: {error}", item.staged.display()))
    }).collect();
    Ok(DeleteOutcome { retained_directory: if managed { None } else { Some(server.directory) }, cleanup_warnings })
}

fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { Some("image/png") }
    else if bytes.starts_with(&[0xff, 0xd8, 0xff]) { Some("image/jpeg") }
    else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" { Some("image/webp") }
    else { None }
}

fn image_file(root: &Path, id: &str) -> PathBuf { root.join("server-images").join(id) }

fn read_image(root: &Path, id: &str) -> Option<String> {
    let legacy = image_file(root,id);
    let server = get(root,id).ok()?;
    let path = if legacy.exists() {legacy} else {Path::new(&server.directory).join("server-icon.png")};
    if std::fs::metadata(&path).ok()?.len() > 2_000_000 { return None; }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() > 2_000_000 { return None; }
    let icon = icon_png(&bytes).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(icon)))
}

fn image_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let size = std::fs::metadata(path).map_err(|e| format!("No se pudo abrir la imagen: {e}"))?.len();
    if size > 2_000_000 { return Err("La imagen debe ocupar menos de 2 MB".into()); }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() > 2_000_000 { return Err("La imagen debe ocupar menos de 2 MB".into()); }
    if image_mime(&bytes).is_none() { return Err("Selecciona un archivo PNG, JPG o WebP válido".into()); }
    icon_png(&bytes)?;
    Ok(bytes)
}

fn icon_png(bytes: &[u8]) -> Result<Vec<u8>,String> {
    use image::{ImageReader,imageops::FilterType};
    use std::io::Cursor;
    let mut reader=ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e|e.to_string())?;
    let mut limits=image::Limits::default();
    limits.max_image_width=Some(4096);limits.max_image_height=Some(4096);limits.max_alloc=Some(64*1024*1024);
    reader.limits(limits);
    let image=reader.decode().map_err(|_|"No se pudo leer la imagen. Usa un PNG, JPG o WebP válido de hasta 4096 × 4096.")?;
    if image.width()==64 && image.height()==64 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {return Ok(bytes.to_vec())}
    let icon=image.resize_to_fill(64,64,FilterType::Lanczos3);
    let mut output=Cursor::new(Vec::new());
    icon.write_to(&mut output,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    Ok(output.into_inner())
}

pub fn preview_image(path: &Path) -> Result<String, String> {
    let bytes = image_bytes(path)?;
    let mime = image_mime(&bytes).ok_or("Formato de imagen no válido")?;
    Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

pub fn read_cover(root:&Path,id:&str)->Result<Option<String>,String> {
    get(root,id)?;
    Uuid::parse_str(id).map_err(|_|"Identificador de servidor no válido")?;
    let path=root.join("server-covers").join(id);
    if !path.exists(){return Ok(None);}
    if fs::symlink_metadata(&path).map_err(|e|e.to_string())?.file_type().is_symlink() || !path.canonicalize().map_err(|e|e.to_string())?.starts_with(root.canonicalize().map_err(|e|e.to_string())?) {return Err("La imagen sale de los datos de la aplicación".into());}
    let bytes=image_bytes(&path)?;
    let mime=image_mime(&bytes).ok_or("Imagen no válida")?;
    Ok(Some(format!("data:{mime};base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes))))
}

fn write_image(root: &Path, id: &str, bytes: &[u8]) -> Result<(), String> {
    Uuid::parse_str(id).map_err(|_|"Identificador de servidor no válido")?;
    let server=get(root,id)?;
    let icon=icon_png(bytes)?;
    let cover_dir=root.join("server-covers");
    fs::create_dir_all(&cover_dir).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&cover_dir).map_err(|e|e.to_string())?.file_type().is_symlink() || !cover_dir.canonicalize().map_err(|e|e.to_string())?.starts_with(root.canonicalize().map_err(|e|e.to_string())?) {return Err("La carpeta de imágenes sale de los datos de la aplicación".into());}
    let targets=[Path::new(&server.directory).join("server-icon.png"),cover_dir.join(id)];
    let legacy=image_file(root,id);
    let mut backups=Vec::new();let mut temporary=Vec::new();let mut published=Vec::new();
    let result=(||->Result<(),String>{
        for path in targets.iter().chain(std::iter::once(&legacy)) {
            if let Ok(meta)=fs::symlink_metadata(path) {if !meta.is_file() || meta.file_type().is_symlink(){return Err("La imagen no puede ser un enlace ni una carpeta".into());}}
        }
        for (target,data) in targets.iter().zip([icon.as_slice(),bytes]) {
            let stage=target.with_file_name(format!(".image-{}.part",Uuid::new_v4()));
            temporary.push(stage.clone());fs::write(stage,data).map_err(|e|e.to_string())?;
        }
        for path in targets.iter().chain(std::iter::once(&legacy)) {
            if path.exists() {
                let backup=path.with_file_name(format!(".image-{}.previous",Uuid::new_v4()));
                fs::rename(path,&backup).map_err(|e|e.to_string())?;backups.push((path.clone(),backup));
            }
        }
        for (stage,target) in temporary.iter().zip(targets.iter()) {
            fs::rename(stage,target).map_err(|e|e.to_string())?;published.push(target.clone());
        }
        Ok(())
    })();
    if let Err(error)=result {
        for path in published {let _=fs::remove_file(path);}
        let mut failures=Vec::new();
        for (target,backup) in backups.into_iter().rev(){if let Err(cause)=fs::rename(backup,&target){failures.push(format!("{}: {cause}",target.display()));}}
        for stage in temporary{let _=fs::remove_file(stage);}
        return Err(if failures.is_empty(){error}else{format!("{error}. No se pudo recuperar la imagen anterior: {}",failures.join("; "))});
    }
    for (_,backup) in backups{let _=fs::remove_file(backup);}
    Ok(())
}

pub fn sync_legacy_icon(root: &Path,id: &str) -> Result<(),String> {
    let path=image_file(root,id);
    if path.exists() {let bytes=image_bytes(&path)?;write_image(root,id,&bytes)?;}
    Ok(())
}

pub fn set_image(root: &Path, id: &str, source: &Path) -> Result<Server, String> {
    get(root, id)?;
    let bytes = image_bytes(source)?;
    write_image(root, id, &bytes)?;
    let mut server = get(root, id)?;
    server.image_data = read_image(root, id);
    Ok(server)
}

pub fn clear_image(root: &Path, id: &str) -> Result<Server, String> {
    let server=get(root,id)?;
    for path in [image_file(root,id),root.join("server-covers").join(id),Path::new(&server.directory).join("server-icon.png")] {
        match std::fs::remove_file(path) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    get(root, id)
}

pub fn create(root: &Path, input: CreateServer) -> Result<Server, String> {
    create_with_seed(root,input,None)
}
pub fn create_with_seed(root: &Path, input: CreateServer,seed:Option<String>) -> Result<Server, String> {
    validate(&input)?;
    crate::properties::validate_seed(seed.as_deref().unwrap_or(""))?;
    let image = input.image_path.as_ref().map(|path| image_bytes(Path::new(path))).transpose()?;
    let id = Uuid::new_v4().to_string();
    let dir = root.join("servers").join(&id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let props = format!("# Managed by TCHLauncher\nserver-port={}\nmax-players={}\nmotd={}\nonline-mode=true\n", input.port, input.max_players, input.name.replace(['\n', '\r'], " "));
    std::fs::write(dir.join("server.properties"), props).map_err(|e| e.to_string())?;
    let mut defaults=crate::properties::editor_defaults(&input.version);
    defaults.insert("level-seed".into(),seed.unwrap_or_default());
    crate::properties::write(&dir.join("server.properties"),defaults)?;
    let db = connect(root)?;
    db.execute("INSERT INTO servers (id,name,version,platform,directory,memory_min_mb,memory_max_mb,created_at,storage_target_gb) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![id,input.name,input.version,input.platform,dir.to_string_lossy(),input.memory_min_mb,input.memory_max_mb,now,input.storage_target_gb])
        .map_err(|e| e.to_string())?;
    if let Some(bytes) = image { write_image(root, &id, &bytes)?; }
    let mut server = get(root, &id)?;
    server.image_data = read_image(root, &id);
    crate::eula::apply_recorded(root,&server)?;
    Ok(server)
}

#[cfg(test)]
mod creation_rules_tests {
    use super::*;
    fn input()->CreateServer{CreateServer{name:"Seed world".into(),version:"1.21.11".into(),platform:"fabric".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}}
    #[test]
    fn creation_persists_seed_and_editable_rules_before_java_or_world_exists(){
        let root=std::env::temp_dir().join(format!("tch-create-seed-{}",Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let server=create_with_seed(&root,input(),Some("-9223372036854775808".into())).unwrap();
        let dir=Path::new(&server.directory);let props=crate::properties::read(&dir.join("server.properties")).unwrap();
        assert_eq!(props["level-seed"],"-9223372036854775808");assert_eq!(props["gamemode"],"survival");assert_eq!(props["difficulty"],"easy");assert_eq!(props["online-mode"],"true");assert_eq!(props["server-port"],"25565");
        assert!(server.jar_path.is_none());assert!(!dir.join("world").exists());assert!(!dir.join("eula.txt").exists());
        let second=create(&root,input()).unwrap();assert_eq!(crate::properties::read(&Path::new(&second.directory).join("server.properties")).unwrap()["level-seed"],"");
        assert!(create_with_seed(&root,input(),Some("seed\nonline-mode=false".into())).is_err());assert_eq!(list(&root).unwrap().len(),2);
        fs::remove_dir_all(root).unwrap();
    }
}

pub fn inspect_import(path: &Path) -> Result<ImportInspection, String> {
    let source = path.canonicalize().map_err(|e| format!("No se pudo abrir la carpeta: {e}"))?;
    if !source.is_dir() || !source.join("server.properties").is_file() {
        return Err("La carpeta no contiene server.properties".into());
    }
    let mut jars = std::fs::read_dir(&source).map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("jar")))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect::<Vec<_>>();
    let fabric_base=source.join(".tch-fabric");
    if let Ok(generations)=fs::read_dir(&fabric_base) {
        for entry in generations.take(1000).filter_map(Result::ok) {
            let name=entry.file_name().to_string_lossy().into_owned();
            if Uuid::parse_str(&name).is_ok() {
                let launcher=entry.path().join("fabric-server-launch.jar");
                if launcher.is_file() && launcher.canonicalize().is_ok_and(|path|path.starts_with(&source)) {jars.push(format!(".tch-fabric/{name}/fabric-server-launch.jar"));}
            }
        }
    }
    jars.extend(crate::neoforge::discover(&source));
    jars.retain(|name|source.join(name).canonicalize().is_ok_and(|path|path.starts_with(&source)));
    jars.sort();
    if jars.is_empty() { return Err("No se encontró un archivo de arranque compatible en la carpeta".into()); }
    if jars.len()>1000{return Err("Hay demasiados archivos de arranque para inspeccionar la carpeta".into());}
    let launchers=jars.iter().map(|jar|crate::import_identity::inspect(&source,jar)).collect::<Vec<_>>();
    let versions=launchers.iter().filter_map(|launcher|launcher.version.as_ref()).collect::<std::collections::HashSet<_>>();
    let detected_version=(versions.len()==1).then(||(*versions.iter().next().unwrap()).clone());
    Ok(ImportInspection {
        path:source.to_string_lossy().into(),
        suggested_name:source.file_name().map(|n| n.to_string_lossy().into()).unwrap_or_else(|| "Servidor importado".into()),
        jar_files:jars,detected_version,launchers,
    })
}

#[cfg(test)]
pub fn import(root: &Path, input: ImportServer) -> Result<Server, String> {
    import_inner(root,input,&Uuid::new_v4().to_string(),None)
}

pub fn import_tracked(root:&Path,input:ImportServer,id:&str,task:&crate::download::DownloadTask)->Result<Server,String>{
    import_inner(root,input,id,Some(task))
}

fn import_inner(root:&Path,input:ImportServer,id:&str,task:Option<&crate::download::DownloadTask>)->Result<Server,String>{
    Uuid::parse_str(id).map_err(|_|"Identificador de servidor no válido")?;
    if let Some(task)=task {task.cancellation.check()?;task.report("Comprobando la carpeta del servidor","preparing");}
    let source = PathBuf::from(&input.path).canonicalize().map_err(|e| e.to_string())?;
    let inspection = inspect_import(&source)?;
    if !inspection.jar_files.contains(&input.jar_file) { return Err("El archivo de arranque seleccionado no existe en esa carpeta".into()); }
    let selected=inspection.launchers.iter().find(|launcher|launcher.filename==input.jar_file).ok_or("Falta el arranque seleccionado")?;
    if let Some(platform)=&selected.platform{if platform!=&input.platform{return Err(format!("El archivo seleccionado declara {platform}; revisa el tipo del servidor"));}}
    if let Some(version)=&selected.version{if version!=&input.version{return Err(format!("El archivo seleccionado declara Minecraft {version}; revisa la versión"));}}
    if input.platform=="neoforge"{crate::neoforge::validate_import(&source,&input.jar_file,&input.version)?;}
    let properties = crate::properties::read(&source.join("server.properties"))?;
    let port = properties.get("server-port").and_then(|v| v.parse::<u16>().ok()).unwrap_or(25565);
    let max_players = properties.get("max-players").and_then(|v| v.parse::<u32>().ok()).unwrap_or(20);
    validate(&CreateServer {name:input.name.clone(),version:input.version.clone(),platform:input.platform.clone(),memory_min_mb:1024,memory_max_mb:2048,max_players,port,storage_target_gb:input.storage_target_gb,image_path:input.image_path.clone()})?;
    let image = input.image_path.as_ref().map(|path| image_bytes(Path::new(path))).transpose()?;
    let db = connect(root)?;
    if input.copy && root.canonicalize().map_err(|e|e.to_string())?.starts_with(&source){return Err("Selecciona la carpeta de un servidor, no la biblioteca completa".into());}
    let target = if input.copy {
        let target = root.join("servers").join(&id);
        let stage = root.join("servers").join(format!(".{id}.part"));
        let prepared=(||{
            if let Some(task)=task {crate::backup::copy_tree_tracked(&source,&stage,task,"Importando archivos del servidor")?;task.cancellation.check()?;task.report("Comprobando el servidor importado","verifying");}
            else {crate::backup::copy_tree(&source,&stage)?;}
            let copied=inspect_import(&stage)?;
            if !copied.jar_files.contains(&input.jar_file){return Err("El archivo de arranque cambió durante la importación".into());}
            if input.platform=="neoforge"{crate::neoforge::validate_import(&stage,&input.jar_file,&input.version)?;}
            if let Some(task)=task {task.begin_publication("Incorporando el servidor a la biblioteca")?;}
            std::fs::rename(&stage,&target).map_err(|e|e.to_string())
        })();
        if let Err(error)=prepared {let _=std::fs::remove_dir_all(&stage);return Err(error);}
        target
    } else {
        if let Some(task)=task {task.begin_publication("Incorporando la carpeta a la biblioteca")?;}
        source
    };
    let jar = target.join(&input.jar_file);
    let inserted = db.execute("INSERT INTO servers (id,name,version,platform,directory,jar_path,memory_min_mb,memory_max_mb,created_at,storage_target_gb) VALUES (?1,?2,?3,?4,?5,?6,1024,2048,?7,?8)",
        params![id,input.name,input.version,input.platform,target.to_string_lossy(),jar.to_string_lossy(),chrono::Utc::now().to_rfc3339(),input.storage_target_gb])
        .map_err(|e| e.to_string());
    if inserted.is_err() && input.copy { let _ = std::fs::remove_dir_all(&target); }
    inserted?;
    if let Some(bytes) = image { write_image(root, &id, &bytes)?; }
    let server=get(root,&id)?;
    crate::eula::apply_recorded(root,&server)?;
    Ok(server)
}

fn validate(input: &CreateServer) -> Result<(), String> {
    let name = input.name.trim();
    if name.is_empty() || name.len() > 64 || name.chars().any(char::is_control) {
        return Err("El nombre debe tener entre 1 y 64 caracteres, sin saltos de línea".into());
    }
    if !matches!(input.platform.as_str(), "vanilla" | "paper" | "fabric" | "neoforge") {
        return Err("Esta plataforma aún no tiene instalación compatible".into());
    }
    if input.version.len() > 24 || !input.version.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
        return Err("Versión de Minecraft no válida".into());
    }
    if input.memory_min_mb < 512 || input.memory_max_mb > 65536 || input.memory_min_mb > input.memory_max_mb {
        return Err("La memoria inicial debe ser al menos 512 MB y no superar la máxima".into());
    }
    if input.max_players == 0 || input.max_players > 1000 || input.port == 0 {
        return Err("Revisa el número de jugadores y el puerto".into());
    }
    if !(1..=2048).contains(&input.storage_target_gb) { return Err("El objetivo de espacio debe estar entre 1 y 2048 GB".into()); }
    Ok(())
}

pub fn update_install(root: &Path, id: &str, jar: &Path, java: &Path) -> Result<(), String> {
    let db = connect(root)?;
    db.execute("UPDATE servers SET jar_path=?2, java_path=?3 WHERE id=?1",
        params![id, jar.to_string_lossy(), java.to_string_lossy()])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_java(root: &Path, id: &str, java: &Path) -> Result<(), String> {
    let db = connect(root)?;
    db.execute("UPDATE servers SET java_path=?2 WHERE id=?1",params![id,java.to_string_lossy()])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_memory(root: &Path, id: &str, min: u32, max: u32) -> Result<(), String> {
    if min < 512 || max > 65536 || min > max { return Err("Valores de memoria no válidos".into()); }
    let db = connect(root)?;
    db.execute("UPDATE servers SET memory_min_mb=?2,memory_max_mb=?3 WHERE id=?1", params![id,min,max])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_storage_target(root: &Path, id: &str, gb: u32) -> Result<(), String> {
    if !(1..=2048).contains(&gb) { return Err("El objetivo de espacio debe estar entre 1 y 2048 GB".into()); }
    let db = connect(root)?;
    let updated = db.execute("UPDATE servers SET storage_target_gb=?2 WHERE id=?1", params![id,gb]).map_err(|e| e.to_string())?;
    if updated == 0 { return Err("No se encontró el servidor".into()); }
    Ok(())
}

pub fn add_backup(root: &Path, backup: &Backup) -> Result<(), String> {
    let db = connect(root)?;
    db.execute("INSERT INTO backups (id,server_id,created_at,size_bytes,path) VALUES (?1,?2,?3,?4,?5)",
        params![backup.id, backup.server_id, backup.created_at, i64::try_from(backup.size_bytes).map_err(|e| e.to_string())?, backup.path])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list_backups(root: &Path, server_id: &str) -> Result<Vec<Backup>, String> {
    let db = connect(root)?;
    let mut stmt = db.prepare("SELECT id,server_id,created_at,size_bytes,path FROM backups WHERE server_id=?1 ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([server_id], |r| Ok(Backup { id:r.get(0)?,server_id:r.get(1)?,created_at:r.get(2)?,size_bytes:r.get::<_,i64>(3)?.max(0) as u64,path:r.get(4)?,retention_warning:None }))
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())
}

pub fn backup_path(root: &Path, server_id: &str, backup_id: &str) -> Result<PathBuf, String> {
    let db = connect(root)?;
    let path: Option<String> = db.query_row("SELECT path FROM backups WHERE server_id=?1 AND id=?2", params![server_id,backup_id], |r| r.get(0))
        .optional().map_err(|e| e.to_string())?;
    path.map(PathBuf::from).ok_or_else(|| "No se encontró el backup".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrates_existing_servers_and_persists_storage_target() {
        let root = std::env::temp_dir().join(format!("tch-migration-{}", Uuid::new_v4()));
        std::fs::create_dir_all(root.join("database")).unwrap();
        let db_path = root.join("database").join("tchlauncher.sqlite");
        {
            let old = Connection::open(&db_path).unwrap();
            old.execute_batch("CREATE TABLE servers (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
                platform TEXT NOT NULL, directory TEXT NOT NULL UNIQUE,
                jar_path TEXT, java_path TEXT, memory_min_mb INTEGER NOT NULL,
                memory_max_mb INTEGER NOT NULL, created_at TEXT NOT NULL
            );
            INSERT INTO servers (id,name,version,platform,directory,memory_min_mb,memory_max_mb,created_at)
            VALUES ('old','Existing','1.21.4','vanilla','C:/test-server',1024,2048,'2026-01-01');").unwrap();
        }
        assert_eq!(get(&root, "old").unwrap().storage_target_gb, 20);
        assert!(!get(&root,"old").unwrap().pinned);
        set_pinned(&root,"old",true).unwrap();
        assert!(get(&root,"old").unwrap().pinned);
        update_storage_target(&root, "old", 48).unwrap();
        assert_eq!(get(&root, "old").unwrap().storage_target_gb, 48);
        assert!(update_storage_target(&root, "old", 0).is_err());
        assert_eq!(list(&root).unwrap()[0].storage_target_gb, 48);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn rejects_unsafe_versions_and_memory() {
        let mut input = CreateServer {name:"Test".into(), version:"../x".into(), platform:"vanilla".into(), memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None};
        assert!(validate(&input).is_err());
        input.version="1.21.4".into();
        input.memory_min_mb=4096;
        assert!(validate(&input).is_err());
    }
    #[test]
    fn imports_copy_without_mutating_source() {
        let root = std::env::temp_dir().join(format!("tch-import-{}",Uuid::new_v4()));
        let source = root.join("existing");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("server.properties"),"server-port=25566\nmax-players=12\n").unwrap();
        std::fs::write(source.join("server.jar"),b"jar fixture").unwrap();
        let inspection = inspect_import(&source).unwrap();
        assert_eq!(inspection.jar_files,vec!["server.jar"]);
        let server = import(&root,ImportServer {path:source.to_string_lossy().into(),name:"Existing".into(),version:"1.21.4".into(),platform:"vanilla".into(),jar_file:"server.jar".into(),copy:true,storage_target_gb:20,image_path:None}).unwrap();
        assert_ne!(Path::new(&server.directory),source);
        assert_eq!(std::fs::read(source.join("server.jar")).unwrap(),b"jar fixture");
        assert_eq!(std::fs::read(server.jar_path.unwrap()).unwrap(),b"jar fixture");
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn server_image_is_local_and_can_be_changed_or_removed() {
        let root = std::env::temp_dir().join(format!("tch-image-{}",Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let image = root.join("icon.png");
        let mut image_data=std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(120,80).write_to(&mut image_data,image::ImageFormat::Png).unwrap();
        let png=image_data.into_inner();
        std::fs::write(&image, png).unwrap();
        let server = create(&root, CreateServer {name:"With icon".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:Some(image.to_string_lossy().into())}).unwrap();
        assert!(server.image_data.as_deref().unwrap().starts_with("data:image/png;base64,"));
        assert!(list(&root).unwrap()[0].image_data.is_some());
        assert_eq!(read_cover(&root,&server.id).unwrap(),Some(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(fs::read(&image).unwrap()))));
        let legacy_path=root.join("server-images").join(&server.id);
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path,fs::read(&image).unwrap()).unwrap();
        sync_legacy_icon(&root,&server.id).unwrap();
        assert!(!legacy_path.exists());
        let icon_path=Path::new(&server.directory).join("server-icon.png");
        let icon=image::open(&icon_path).unwrap();
        assert_eq!((icon.width(),icon.height()),(64,64));
        let expected=format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(fs::read(&icon_path).unwrap()));
        assert_eq!(list(&root).unwrap()[0].image_data.as_deref(),Some(expected.as_str()));
        std::fs::write(&image, b"not an image").unwrap();
        assert!(set_image(&root,&server.id,&image).is_err());
        assert!(list(&root).unwrap()[0].image_data.is_some());
        clear_image(&root,&server.id).unwrap();
        assert!(list(&root).unwrap()[0].image_data.is_none());
        assert!(!icon_path.exists());
        assert!(read_cover(&root,&server.id).unwrap().is_none());
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn deletes_managed_files_but_preserves_external_imports() {
        let root = std::env::temp_dir().join(format!("tch-delete-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let managed = create(&root, CreateServer {name:"Managed".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:1024,memory_max_mb:2048,max_players:20,port:25565,storage_target_gb:20,image_path:None}).unwrap();
        fs::write(Path::new(&managed.directory).join("server.jar"), b"managed").unwrap();
        crate::backup::create(&root, &managed.id).unwrap();
        fs::create_dir_all(root.join("server-images")).unwrap();
        fs::write(root.join("server-images").join(&managed.id), b"image").unwrap();
        let preview = delete_preview(&root, &managed.id).unwrap();
        assert!(preview.managed_files);
        assert_eq!(preview.backup_count, 1);
        let result = delete(&root, &managed.id).unwrap();
        assert!(result.retained_directory.is_none());
        assert!(result.cleanup_warnings.is_empty());
        assert!(!Path::new(&managed.directory).exists());
        assert!(!root.join("backups").join(&managed.id).exists());
        assert!(!root.join("server-images").join(&managed.id).exists());
        assert!(get(&root, &managed.id).is_err());

        let external = root.join("external");
        fs::create_dir_all(&external).unwrap();
        fs::write(external.join("server.properties"), b"server-port=25566\nmax-players=12\n").unwrap();
        fs::write(external.join("server.jar"), b"external").unwrap();
        let imported = import(&root, ImportServer {path:external.to_string_lossy().into(),name:"External".into(),version:"1.21.4".into(),platform:"vanilla".into(),jar_file:"server.jar".into(),copy:false,storage_target_gb:20,image_path:None}).unwrap();
        assert!(!delete_preview(&root, &imported.id).unwrap().managed_files);
        let result = delete(&root, &imported.id).unwrap();
        assert_eq!(result.retained_directory.as_deref(), Some(imported.directory.as_str()));
        assert_eq!(fs::read(external.join("server.jar")).unwrap(), b"external");
        assert!(get(&root, &imported.id).is_err());
        let _ = fs::remove_dir_all(root);
    }
}
