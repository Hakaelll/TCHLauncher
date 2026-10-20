use crate::model::{Server, VersionInfo};
use crate::download::DownloadTask;
use crate::storage;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc,Mutex,OnceLock,TryLockError};
use uuid::Uuid;
use walkdir::WalkDir;

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
static RUNTIME_LOCKS: OnceLock<Mutex<HashMap<PathBuf,Arc<Mutex<()>>>>> = OnceLock::new();

fn client() -> Result<Client, String> {
    Client::builder().user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION")," (https://github.com/Hakaelll/TCHLauncher)"))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" { attempt.error("HTTPS required") }
            else if attempt.previous().len() > 10 { attempt.error("too many redirects") }
            else { attempt.follow() }
        }))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build().map_err(|e| e.to_string())
}

fn json(client: &Client, url: &str) -> Result<Value, String> {
    if !url.starts_with("https://") { return Err("La fuente de descarga no utiliza HTTPS".into()); }
    client.get(url).timeout(std::time::Duration::from_secs(30)).send().and_then(|r| r.error_for_status()).and_then(|r| r.json())
        .map_err(|e| format!("No se pudo consultar {url}: {e}"))
}

fn catalog_versions(data:&Value,experimental:bool)->Result<Vec<VersionInfo>,String> {
    let items=data["versions"].as_array().ok_or("Respuesta de versiones no válida")?;
    Ok(items.iter().filter_map(|entry| {
        let kind=entry["type"].as_str()?;
        if kind!="release" && !(experimental && kind=="snapshot") {return None;}
        let id=entry["id"].as_str()?;
        crate::fabric::validate_version(id).ok()?;
        Some(VersionInfo {id:id.into(),kind:kind.into()})
    }).collect())
}

pub fn versions()->Result<Vec<VersionInfo>,String> {catalog_versions(&json(&client()?,MANIFEST)?,false)}

fn fabric_catalog(data:&Value,official:Vec<VersionInfo>,experimental:bool)->Result<Vec<VersionInfo>,String> {
    let items=data.as_array().ok_or("Catálogo de Minecraft para Fabric no válido")?;
    let supported:HashMap<_,_>=items.iter().filter_map(|entry|Some((entry["version"].as_str()?,entry["stable"].as_bool()?))).collect();
    Ok(official.into_iter().filter(|version|match supported.get(version.id.as_str()) {
        Some(true)=>version.kind=="release",
        Some(false)=>experimental && version.kind=="snapshot",
        None=>false,
    }).collect())
}

pub fn platform_versions_with_options(platform:&str,experimental:bool)->Result<Vec<VersionInfo>,String> {
    if experimental && !matches!(platform,"vanilla"|"fabric") {return Err("El catálogo experimental solo está disponible para Vanilla y Fabric.".into());}
    if platform=="vanilla" {return catalog_versions(&json(&client()?,MANIFEST)?,experimental);}
    if platform=="paper" {let client=client()?;return crate::paper_catalog::versions(&client,catalog_versions(&json(&client,MANIFEST)?,false)?);}
    if platform=="neoforge" {return crate::neoforge::versions(&DownloadTask::new("catalog","install",|_|{})?,versions()?);}
    if platform!="fabric" {return Err("Plataforma de servidor no compatible".into());}
    let task=DownloadTask::new("catalog","install",|_|{})?;
    let data=task.json("https://meta.fabricmc.net/v2/versions/game","Consultando versiones Fabric")?;
    fabric_catalog(&data,catalog_versions(&json(&client()?,MANIFEST)?,experimental)?,experimental)
}

pub(crate) fn version_metadata(task: &DownloadTask, version: &str) -> Result<Value, String> {
    let manifest = task.json(MANIFEST, "Consultando versiones de Minecraft")?;
    let entry = manifest["versions"].as_array().ok_or("Respuesta de Mojang no válida")?
        .iter().find(|v| v["id"].as_str() == Some(version))
        .ok_or_else(|| format!("Minecraft {version} no está en el catálogo oficial"))?;
    let url = entry["url"].as_str().ok_or("Falta la URL de la versión")?;
    task.json(url, "Consultando requisitos de Minecraft")
}

pub fn required_java(root:&Path,version:&str)->Result<u64,String> {
    if let Some(major)=crate::runtime::cached_requirement(root,version){return Ok(major);}
    let task=DownloadTask::new("runtime-check","runtime",|_|{})?;
    let meta=version_metadata(&task,version)?;
    let major=meta["javaVersion"]["majorVersion"].as_u64().ok_or("Mojang no indicó la versión Java necesaria")?;
    crate::runtime::remember_requirement(root,version,major)?;
    Ok(major)
}

fn platform_archive() -> Result<(&'static str, &'static str, &'static str), String> {
    let os = match std::env::consts::OS { "windows" => "windows", "linux" => "linux", _ => return Err("La instalación automática de Java no está disponible en este sistema".into()) };
    let arch = match std::env::consts::ARCH { "x86_64" => "x64", "aarch64" => "aarch64", _ => return Err("Arquitectura no compatible con Java gestionado".into()) };
    Ok((os, arch, if os == "windows" { "zip" } else { "tar.gz" }))
}

fn extract_archive(archive: &Path, destination: &Path, format: &str, task: &DownloadTask) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    if format == "zip" {
        let mut zip = zip::ZipArchive::new(File::open(archive).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if zip.len() > 20_000 { return Err("El archivo Java tiene demasiadas entradas".into()); }
        let mut unpacked_size = 0u64;
        for i in 0..zip.len() {
            task.cancellation.check()?;
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            unpacked_size = unpacked_size.saturating_add(entry.size());
            if unpacked_size > 2_000_000_000 { return Err("El archivo Java extraído supera 2 GB".into()); }
            let relative = entry.enclosed_name().ok_or("Ruta no segura dentro del archivo Java")?;
            if entry.unix_mode().map(|m| m & 0o170000 == 0o120000).unwrap_or(false) {
                return Err("El archivo Java contiene un enlace simbólico no permitido".into());
            }
            let path = destination.join(relative);
            if entry.is_dir() { fs::create_dir_all(path).map_err(|e| e.to_string())?; }
            else {
                if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut out = File::create(path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
    } else {
        let gz = flate2::read::GzDecoder::new(File::open(archive).map_err(|e| e.to_string())?);
        let mut tar = tar::Archive::new(gz);
        let mut unpacked_size = 0u64;
        let mut entry_count = 0usize;
        for item in tar.entries().map_err(|e| e.to_string())? {
            task.cancellation.check()?;
            let mut entry = item.map_err(|e| e.to_string())?;
            entry_count += 1;
            unpacked_size = unpacked_size.saturating_add(entry.size());
            if entry_count > 20_000 || unpacked_size > 2_000_000_000 { return Err("El archivo Java extraído supera los límites de seguridad".into()); }
            if !matches!(entry.header().entry_type(), tar::EntryType::Regular | tar::EntryType::Directory) { continue; }
            if !entry.unpack_in(destination).map_err(|e| e.to_string())? { return Err("Ruta no segura en el archivo Java".into()); }
        }
    }
    Ok(())
}

fn java_executable(root: &Path) -> Option<PathBuf> {
    let name = if cfg!(windows) { "java.exe" } else { "java" };
    WalkDir::new(root).max_depth(5).into_iter().filter_map(Result::ok)
        .find(|e| e.file_type().is_file() && e.file_name() == name && e.path().parent().and_then(|p| p.file_name()).is_some_and(|p| p == "bin"))
        .map(|e| e.into_path())
}

fn verify_java(path: &Path, major: u64) -> Result<(), String> {
    let output = crate::runtime::version_output(path)?;
    let version = crate::runtime::parse_version(&output).ok_or("Java no devolvió una versión reconocible")?;
    if !version.starts_with(&format!("{major}.")) && !(major == 8 && version.starts_with("1.8.")) {
        return Err(format!("Java no devolvió la versión {major} esperada: {version}"));
    }
    Ok(())
}

fn ensure_java(root: &Path, major: u64, task: &DownloadTask, reinstall: bool) -> Result<PathBuf, String> {
    let name = format!("temurin-{major}-{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    // A fresh generation does not replace a runtime another server may be using.
    let target = root.join("runtimes").join(if reinstall {format!("{name}-{}",Uuid::new_v4())} else {name});
    let runtime_lock = RUNTIME_LOCKS.get_or_init(||Mutex::new(HashMap::new())).lock().map_err(|e|e.to_string())?
        .entry(target.clone()).or_insert_with(||Arc::new(Mutex::new(()))).clone();
    let mut waiting = false;
    let _runtime_guard = loop {
        task.cancellation.check()?;
        match runtime_lock.try_lock() {
            Ok(guard) => break guard,
            Err(TryLockError::WouldBlock) => {
                if !waiting {task.report("Esperando a que termine la preparación de Java","preparing");waiting=true;}
                std::thread::sleep(std::time::Duration::from_millis(50));
            },
            Err(TryLockError::Poisoned(_)) => return Err("La preparación de Java se interrumpió. Reinicia el launcher para volver a intentarlo.".into()),
        }
    };
    if let Some(path) = java_executable(&target) { if verify_java(&path, major).is_ok() { return Ok(path); } }
    let (os, arch, format) = platform_archive()?;
    let mut package = None;
    for kind in ["jre","jdk"] {
        let meta = task.json(&format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture={arch}&image_type={kind}&os={os}&vendor=eclipse"), "Buscando Java compatible")?;
        if let Some(asset) = temurin_package(&meta,major,format) {package = Some(asset);break;}
    }
    let (url,checksum) = package.ok_or_else(||format!("Adoptium no ofrece Temurin Java {major} para {os}/{arch}, ni como JRE ni como JDK"))?;
    let stage = root.join("runtimes").join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let archive = stage.join(format!("java.{format}"));
    let result = (|| {
        task.download(&url, &archive, Some(("sha256", &checksum)), &format!("Temurin Java {major}"))?;
        let unpacked = stage.join("unpacked");
        task.report("Extrayendo Java", "extracting");
        extract_archive(&archive, &unpacked, format, task)?;
        let path = java_executable(&unpacked).ok_or("El paquete Java no contiene java ejecutable")?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).map_err(|e| e.to_string())?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;
        }
        task.cancellation.check()?;
        task.report("Comprobando Java", "verifying");
        verify_java(&path, major)?;
        task.cancellation.check()?;
        let previous = target.with_extension(format!("previous-{}",Uuid::new_v4()));
        if target.exists() { fs::rename(&target,&previous).map_err(|e| e.to_string())?; }
        if let Err(e) = fs::rename(&unpacked, &target) {
            if previous.exists() { let _ = fs::rename(&previous,&target); }
            return Err(e.to_string());
        }
        if previous.exists() { let _ = fs::remove_dir_all(previous); }
        java_executable(&target).ok_or_else(|| "Java instalado pero no localizable".into())
    })();
    let _ = fs::remove_dir_all(&stage);
    result
}

fn temurin_package(meta: &Value,major: u64,format: &str) -> Option<(String,String)> {
    meta.as_array()?.iter().find_map(|item| {
        if item["version"]["major"].as_u64()? != major {return None;}
        let package = &item["binary"]["package"];
        let link = package["link"].as_str()?;
        let checksum = package["checksum"].as_str()?;
        if !link.starts_with("https://") || !package["name"].as_str()?.ends_with(format) || checksum.len() != 64 || !checksum.chars().all(|c|c.is_ascii_hexdigit()) {return None;}
        Some((link.into(),checksum.into()))
    })
}

pub fn install(root: &Path, server: &Server, task: &DownloadTask) -> Result<(), String> {
    install_requested(root,server,task,None)
}

pub fn install_requested(root:&Path,server:&Server,task:&DownloadTask,requested:Option<&str>)->Result<(),String> {
    if requested.is_some() && (server.jar_path.is_some() || !matches!(server.platform.as_str(),"fabric"|"neoforge")) {
        return Err("La versión exacta del loader requiere una instancia nueva de Fabric o NeoForge".into());
    }
    task.cancellation.check()?;
    let meta = version_metadata(task, &server.version)?;
    let major = meta["javaVersion"]["majorVersion"].as_u64().ok_or("Mojang no indicó la versión Java necesaria")?;
    crate::runtime::remember_requirement(root,&server.version,major)?;
    if let Some(jar) = &server.jar_path {
        if !Path::new(jar).is_file() { return Err("El archivo .jar importado ya no existe".into()); }
        let java = ensure_java(root,major,task,false)?;
        storage::update_java(root,&server.id,&java)?;
        return Ok(());
    }
    if server.platform=="neoforge" {
        let java=ensure_java(root,major,task,false)?;
        let entrypoint=if requested.is_some() {crate::neoforge::install_requested(server,&meta,&java,task,requested)?} else {crate::neoforge::install(server,&meta,&java,task)?};
        let result=task.cancellation.check().and_then(|_|storage::update_install(root,&server.id,&entrypoint,&java));
        if result.is_err() {
            if let Some(generation)=entrypoint.ancestors().find(|path|path.parent().is_some_and(|parent|parent.file_name().is_some_and(|name|name==".tch-neoforge"))) {let _=fs::remove_dir_all(generation);}
        }
        return result;
    }
    if server.platform=="fabric" {
        let java=ensure_java(root,major,task,false)?;
        let jar=if requested.is_some() {crate::fabric::install_requested(server,&meta,task,requested)?} else {crate::fabric::install(server,&meta,task)?};
        let result=task.cancellation.check().and_then(|_|storage::update_install(root,&server.id,&jar,&java));
        if result.is_err() {if let Some(parent)=jar.parent(){let _=fs::remove_dir_all(parent);}}
        return result;
    }
    let (url, checksum): (String, Option<(&str, String)>) = if server.platform == "vanilla" {
        let artifact = &meta["downloads"]["server"];
        let url = artifact["url"].as_str().ok_or("Esta versión no tiene servidor Vanilla")?;
        let hash = artifact["sha1"].as_str().ok_or("Mojang no proporcionó checksum")?;
        (url.into(), Some(("sha1", hash.into())))
    } else {
        let builds = task.json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{}/builds", server.version), "Buscando Paper compatible")?;
        let stable = builds.as_array().ok_or("Respuesta de Paper no válida")?.iter()
            .find(|b| b["channel"].as_str() == Some("STABLE"))
            .ok_or("Paper no tiene una versión estable para esta versión de Minecraft")?;
        let download = &stable["downloads"]["server:default"];
        let url = download["url"].as_str().ok_or("Paper no proporcionó URL")?;
        let hash = download["checksums"]["sha256"].as_str().map(|s| ("sha256",s.into()));
        (url.into(), hash)
    };
    let java = ensure_java(root, major, task,false)?;
    let jar = Path::new(&server.directory).join("server.jar");
    task.download(&url, &jar, checksum.as_ref().map(|(a,b)| (*a,b.as_str())), "Servidor Minecraft")?;
    task.cancellation.check()?;
    storage::update_install(root, &server.id, &jar, &java)?;
    Ok(())
}

pub fn prepare_runtime(root: &Path,server: &Server,task: &DownloadTask,reinstall: bool) -> Result<(),String> {
    let meta = version_metadata(task,&server.version)?;
    let major = meta["javaVersion"]["majorVersion"].as_u64().ok_or("Mojang no indicó la versión Java necesaria")?;
    crate::runtime::remember_requirement(root,&server.version,major)?;
    let java = ensure_java(root,major,task,reinstall)?;
    task.cancellation.check()?;
    storage::update_java(root,&server.id,&java)
}

// Prepare a complete fresh instance without publishing paths in SQLite. Pack
// installation commits those paths only after the directory switch succeeds.
pub fn prepare_fresh_instance(root:&Path,server:&Server,task:&DownloadTask,loader:Option<&str>)->Result<(PathBuf,PathBuf),String> {
    if server.jar_path.is_some() {return Err("La preparación de un paquete requiere una instancia nueva".into());}
    let metadata=version_metadata(task,&server.version)?;
    let major=metadata["javaVersion"]["majorVersion"].as_u64().ok_or("Minecraft no indicó el runtime necesario")?;
    crate::runtime::remember_requirement(root,&server.version,major)?;
    let java=if let Some(configured)=&server.java_path {
        let path=PathBuf::from(configured);verify_java(&path,major)?;path
    }else{ensure_java(root,major,task,false)?};
    let jar=match server.platform.as_str() {
        "fabric"=>crate::fabric::install_requested(server,&metadata,task,loader)?,
        "neoforge"=>crate::neoforge::install_requested(server,&metadata,&java,task,loader)?,
        "vanilla" if loader.is_none()=>{
            let artifact=&metadata["downloads"]["server"];let jar=Path::new(&server.directory).join("server.jar");
            task.download(artifact["url"].as_str().ok_or("Minecraft no ofrece servidor")?,&jar,Some(("sha1",artifact["sha1"].as_str().ok_or("Minecraft no indicó checksum")?)),"Servidor Minecraft del paquete")?;jar
        },
        "paper"=>{
            let builds=task.json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{}/builds",server.version),"Preparando Paper compatible")?;
            let stable=builds.as_array().ok_or("Catálogo Paper no válido")?.iter().find(|build|{
                build["channel"].as_str()==Some("STABLE")&&loader.is_none_or(|requested|build["id"].as_u64().is_some_and(|id|id.to_string()==requested))
            }).ok_or("El build estable de Paper revisado ya no está disponible")?;
            let artifact=&stable["downloads"]["server:default"];let jar=Path::new(&server.directory).join("server.jar");
            task.download(artifact["url"].as_str().ok_or("Paper no indicó URL")?,&jar,Some(("sha256",artifact["checksums"]["sha256"].as_str().ok_or("Paper no indicó checksum")?)),"Servidor Paper preparado")?;jar
        },
        _=>return Err("Motor del modpack no compatible".into()),
    };
    task.cancellation.check()?;Ok((jar,java))
}

pub fn prepare_loader_update(root:&Path,server:&Server,task:&DownloadTask,loader:&str)->Result<PathBuf,String> {
    let metadata=version_metadata(task,&server.version)?;
    let major=metadata["javaVersion"]["majorVersion"].as_u64().ok_or("Minecraft no indicó el runtime necesario")?;
    crate::runtime::remember_requirement(root,&server.version,major)?;
    let java=PathBuf::from(server.java_path.as_ref().ok_or("Prepara Java antes de actualizar el loader")?);
    verify_java(&java,major)?;
    match server.platform.as_str(){
        "fabric"=>crate::fabric::install_requested(server,&metadata,task,Some(loader)),
        "neoforge"=>crate::neoforge::install_requested(server,&metadata,&java,task,Some(loader)),
        _=>Err("Este motor no tiene un loader actualizable".into()),
    }
}
