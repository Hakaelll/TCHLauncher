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

pub fn versions() -> Result<Vec<VersionInfo>, String> {
    let data = json(&client()?, MANIFEST)?;
    let items = data["versions"].as_array().ok_or("Respuesta de versiones no válida")?;
    Ok(items.iter().filter_map(|v| {
        let kind = v["type"].as_str()?;
        if kind != "release" { return None; }
        Some(VersionInfo { id: v["id"].as_str()?.into(), kind: kind.into() })
    }).take(50).collect())
}

pub fn platform_versions(platform:&str) -> Result<Vec<VersionInfo>,String> {
    if matches!(platform,"vanilla"|"paper") {return versions();}
    if platform=="neoforge" {return crate::neoforge::versions(&DownloadTask::new("catalog","install",|_|{})?,versions()?);}
    if platform!="fabric" {return Err("Plataforma de servidor no compatible".into());}
    let task=DownloadTask::new("catalog","install",|_|{})?;
    let data=task.json("https://meta.fabricmc.net/v2/versions/game","Consultando versiones Fabric")?;
    let items=data.as_array().ok_or("Catálogo de Minecraft para Fabric no válido")?;
    Ok(items.iter().filter(|item|item["stable"]==true).filter_map(|item|Some(VersionInfo{id:item["version"].as_str()?.into(),kind:"release".into()})).collect())
}

fn version_metadata(task: &DownloadTask, version: &str) -> Result<Value, String> {
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
    let java=ensure_java(root,major,task,false)?;
    let jar=match server.platform.as_str() {
        "fabric"=>crate::fabric::install_requested(server,&metadata,task,loader)?,
        "neoforge"=>crate::neoforge::install_requested(server,&metadata,&java,task,loader)?,
        "vanilla" if loader.is_none()=>{
            let artifact=&metadata["downloads"]["server"];let jar=Path::new(&server.directory).join("server.jar");
            task.download(artifact["url"].as_str().ok_or("Minecraft no ofrece servidor")?,&jar,Some(("sha1",artifact["sha1"].as_str().ok_or("Minecraft no indicó checksum")?)),"Servidor Minecraft del paquete")?;jar
        },
        _=>return Err("Motor del modpack no compatible".into()),
    };
    task.cancellation.check()?;Ok((jar,java))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore="Downloads official NeoForge/Minecraft; boots original and copied installations to EULA refusal"]
    fn installs_neoforge_and_imports_without_accepting_eula() {
        let minecraft=std::env::var("TCH_TEST_MINECRAFT").unwrap_or_else(|_|"1.21.4".into());
        use std::process::{Command,Stdio};
        let root=std::env::temp_dir().join(format!("tch neoforge live {}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let result=(|| -> Result<(),String> {
            if let Ok(java)=std::env::var("TCH_TEST_JAVA") {
                let java=Path::new(&java).canonicalize().map_err(|e|e.to_string())?;
                verify_java(&java,21)?;
                let directory=java.parent().and_then(Path::parent).ok_or("Java has no runtime directory")?;
                let target=root.join("runtimes").join(format!("temurin-21-{}-{}",std::env::consts::OS,std::env::consts::ARCH));
                crate::backup::copy_tree(directory,&target)?;
            }
            let server=storage::create(&root,crate::model::CreateServer{name:"NeoForge validation".into(),version:minecraft.clone(),platform:"neoforge".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None})?;
            let directory=Path::new(&server.directory);
            let properties=fs::read(directory.join("server.properties")).map_err(|e|e.to_string())?;
            fs::create_dir(directory.join("config")).map_err(|e|e.to_string())?;
            fs::write(directory.join("config/preserved.txt"),b"keep this configuration").map_err(|e|e.to_string())?;
            let task=DownloadTask::new(&server.id,"install",|event| {if event.stage!="downloading" {eprintln!("{}: {}",event.stage,event.label);}})?;
            install(&root,&server,&task)?;
            let installed=storage::get(&root,&server.id)?;
            assert_eq!(fs::read(directory.join("server.properties")).unwrap(),properties);
            assert_eq!(fs::read(directory.join("config/preserved.txt")).unwrap(),b"keep this configuration");
            assert_eq!(crate::runtime::cached_requirement(&root,&minecraft),Some(21));
            assert!(!directory.join("eula.txt").exists());
            assert!(Path::new(installed.jar_path.as_ref().unwrap()).is_file());
            let boot=|server:&Server,label:&str| -> Result<(),String> {
                let output_path=root.join(format!("{label}.log"));let output=File::create(&output_path).map_err(|e|e.to_string())?;
                let mut command=Command::new(server.java_path.as_ref().unwrap());
                #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
                let mut child=command.args(crate::jvm::launch_arguments(server,&crate::jvm::Settings::default())?)
                    .current_dir(&server.directory).stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output)
                    .spawn().map_err(|e|e.to_string())?;
                let began=std::time::Instant::now();
                loop {
                    if child.try_wait().map_err(|e|e.to_string())?.is_some(){break;}
                    if began.elapsed()>std::time::Duration::from_secs(90) {let _=child.kill();let _=child.wait();return Err("NeoForge did not reach the EULA gate within 90 seconds".into());}
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                let log=fs::read_to_string(output_path).map_err(|e|e.to_string())?;
                let eula=fs::read_to_string(Path::new(&server.directory).join("eula.txt")).map_err(|_|format!("No EULA file was generated: {log}"))?;
                assert!(eula.lines().any(|line|line.trim()=="eula=false"),"{log}");
                assert!(log.to_lowercase().contains("eula"),"{log}");
                assert!(!Path::new(&server.directory).join("world/level.dat").exists());
                Ok(())
            };
            boot(&installed,"original")?;
            let inspection=storage::inspect_import(directory)?;
            assert_eq!(inspection.detected_version.as_deref(),Some(minecraft.as_str()));
            let args=inspection.jar_files.iter().find(|path|path.contains(".tch-neoforge")).ok_or("NeoForge not detected by import")?;
            let imported=storage::import(&root,crate::model::ImportServer{path:server.directory.clone(),name:"Copied NeoForge".into(),version:minecraft.clone(),platform:"neoforge".into(),jar_file:args.clone(),copy:true,storage_target_gb:2,image_path:None})?;
            storage::update_java(&root,&imported.id,Path::new(installed.java_path.as_ref().unwrap()))?;
            let imported=storage::get(&root,&imported.id)?;
            let launch=crate::jvm::launch_arguments(&imported,&crate::jvm::Settings::default())?;
            assert!(!launch.iter().any(|arg|arg.contains(&server.id)));
            assert!(launch.iter().any(|arg|arg.contains(&imported.id)));
            boot(&imported,"imported")?;
            assert_eq!(fs::read(Path::new(&imported.directory).join("config/preserved.txt")).unwrap(),b"keep this configuration");
            Ok(())
        })();
        if result.is_ok(){let _=fs::remove_dir_all(&root);} else {eprintln!("NeoForge diagnostics retained at {}",root.display());}
        result.unwrap();
    }
    #[test]
    #[ignore="Downloads official Fabric and Minecraft 1.21.4; checks EULA refusal without accepting it"]
    fn installs_fabric_and_reaches_minecraft_eula_gate() {
        use std::process::{Command,Stdio};
        let requested=std::env::var("TCH_TEST_LOADER").ok();
        let root=std::env::temp_dir().join(format!("tch fabric live {}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let result=(|| -> Result<(),String> {
            let server=storage::create(&root,crate::model::CreateServer {name:"Fabric validation".into(),version:"1.21.4".into(),platform:"fabric".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None})?;
            let task=DownloadTask::new(&server.id,"install",|event| {if event.stage!="downloading" {eprintln!("{}: {}",event.stage,event.label);}})?;
            install_requested(&root,&server,&task,requested.as_deref())?;
            let installed=storage::get(&root,&server.id)?;
            let jar=Path::new(installed.jar_path.as_ref().ok_or("Missing launcher")?);
            if let Some(loader)=&requested {
                let metadata:Value=serde_json::from_slice(&fs::read(jar.parent().unwrap().join("installation.json")).unwrap()).unwrap();
                assert_eq!(metadata["loader"].as_str(),Some(loader.as_str()));
            }
            assert!(jar.is_file());assert!(crate::fabric::game_jar(&installed).unwrap().is_file());
            let output_path=root.join("boot.log");let output=File::create(&output_path).map_err(|e|e.to_string())?;
            let mut command=Command::new(installed.java_path.as_ref().unwrap());
            #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
            let mut child=command.args(["-Xms512M","-Xmx1024M"])
                .arg(format!("-Dfabric.gameJarPath={}",crate::fabric::game_jar(&installed).unwrap().display()))
                .arg("-jar").arg(jar).arg("nogui").current_dir(&installed.directory)
                .stdin(Stdio::null()).stdout(output.try_clone().unwrap()).stderr(output).spawn().map_err(|e|e.to_string())?;
            let started=std::time::Instant::now();
            loop {
                if child.try_wait().map_err(|e|e.to_string())?.is_some(){break;}
                if started.elapsed()>std::time::Duration::from_secs(90) {let _=child.kill();let _=child.wait();return Err("Fabric did not reach the EULA gate in time".into());}
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            let log=fs::read_to_string(output_path).map_err(|e|e.to_string())?;
            eprintln!("{log}");
            assert!(log.contains("Loading Minecraft 1.21.4 with Fabric Loader"),"{log}");
            assert!(log.to_ascii_lowercase().contains("agree to the eula"),"{log}");
            assert!(!fs::read_to_string(Path::new(&installed.directory).join("eula.txt")).unwrap_or_default().lines().any(|line|line.trim()=="eula=true"));
            assert!(!Path::new(&installed.directory).join("world").exists());
            let inspection=storage::inspect_import(Path::new(&installed.directory))?;
            assert_eq!(inspection.detected_version.as_deref(),Some("1.21.4"));
            let selected_jar=inspection.jar_files.iter().find(|name|name.ends_with("fabric-server-launch.jar")).ok_or("Inspection did not find Fabric launcher")?;
            let imported=storage::import(&root,crate::model::ImportServer {path:installed.directory.clone(),name:"Fabric import validation".into(),version:"1.21.4".into(),platform:"fabric".into(),jar_file:selected_jar.clone(),copy:true,storage_target_gb:2,image_path:None})?;
            assert!(crate::fabric::game_jar(&imported).unwrap().is_file());
            assert!(Path::new(imported.jar_path.as_ref().unwrap()).is_file());
            assert!(jar.is_file());
            let imported_jar=imported.jar_path.clone().unwrap();let before=fs::read(&imported_jar).unwrap();
            install(&root,&imported,&task)?;
            assert_eq!(fs::read(&imported_jar).unwrap(),before);
            assert!(storage::get(&root,&imported.id)?.java_path.is_some());
            Ok(())
        })();
        let _=fs::remove_dir_all(&root);
        result.unwrap();
    }
    #[test]
    fn rejects_non_https_sources() {
        assert!(json(&client().unwrap(), "http://example.com").is_err());
    }
    #[test]
    fn accepts_a_matching_jdk_archive_when_java_16_has_no_jre() {
        let empty = serde_json::json!([]);
        assert!(temurin_package(&empty,16,"zip").is_none());
        let jdk = serde_json::json!([{ "version":{"major":16},"binary":{"image_type":"jdk","package":{
            "name":"OpenJDK16U-jdk_x64_windows_hotspot_16.0.2_7.zip","link":"https://github.com/adoptium/temurin16-binaries/releases/download/jdk-16.0.2%2B7/runtime.zip","checksum":"40191ffbafd8a6f9559352d8de31e8d22a56822fb41bbcf45f34e3fd3afa5f9e"}}}]);
        assert!(temurin_package(&jdk,16,"zip").is_some());
        assert!(temurin_package(&jdk,17,"zip").is_none());
        assert!(temurin_package(&jdk,16,"tar.gz").is_none());
    }
    #[test]
    #[ignore = "Downloads official Java 16 and Minecraft 1.17.1; requires internet"]
    fn installs_real_minecraft_1171_and_temurin16_without_accepting_eula() {
        let root = std::env::temp_dir().join(format!("tch-install-live-{}",Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let result = (|| -> Result<(),String> {
            storage::connect(&root)?;
            let server = storage::create(&root,crate::model::CreateServer {name:"Runtime validation".into(),version:"1.17.1".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None})?;
            let last = Arc::new(Mutex::new(String::new()));
            let task = DownloadTask::new(&server.id,"install",move |event| {
                let next = format!("{}: {}",event.stage,event.label);
                let mut previous = last.lock().unwrap();
                if *previous != next {*previous=next.clone();eprintln!("{next}");}
            })?;
            install(&root,&server,&task)?;
            let installed = storage::get(&root,&server.id)?;
            verify_java(Path::new(installed.java_path.as_ref().ok_or("Java path missing")?),16)?;
            assert!(fs::metadata(installed.jar_path.ok_or("Server jar missing")?).unwrap().len()>1_000_000);
            assert!(!fs::read_to_string(Path::new(&server.directory).join("eula.txt")).unwrap_or_default().lines().any(|line|line.trim()=="eula=true"));
            Ok(())
        })();
        let _=fs::remove_dir_all(&root);
        result.unwrap();
    }
}
