use crate::{download::DownloadTask, model::{Server, VersionInfo}};
use serde_json::Value;
use std::{collections::{BTreeMap, HashSet}, fs::{self, File}, io::{Read, Seek, SeekFrom}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, time::{Duration, Instant}};
use uuid::Uuid;

const CATALOG: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
pub fn catalog_url(minecraft:&str)->&'static str {
    if minecraft=="1.20.1" {"https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge"}else{CATALOG}
}
const MAX_TEXT: u64 = 128 * 1024;

use crate::jvm::java_path;

pub fn stable_loaders(data:&Value,minecraft:&str)->Result<Vec<String>,String> {
    Ok(crate::neoforge_release::stable(data,minecraft)?.into_iter().map(|release|release.version).collect())
}

pub fn requested_loader(data:&Value,minecraft:&str,requested:Option<&str>)->Result<String,String> {
    let Some(version)=requested else {
        return stable_loaders(data,minecraft)?.into_iter().next()
            .ok_or_else(||format!("NeoForge no tiene una versión estable para Minecraft {minecraft}"));
    };
    let release=crate::neoforge_release::Release::selected(version).ok_or("Versión NeoForge solicitada no compatible")?;
    let entries=data["versions"].as_array().ok_or("Catálogo NeoForge no válido")?;
    if entries.len()>100_000{return Err("El catálogo NeoForge supera el límite de versiones".into());}
    if release.minecraft!=minecraft||!entries.iter().any(|entry|entry.as_str()==Some(version)) {
        return Err(format!("NeoForge {version} no está disponible para Minecraft {minecraft}"));
    }
    Ok(version.into())
}

pub fn versions(task: &DownloadTask, minecraft: Vec<VersionInfo>) -> Result<Vec<VersionInfo>, String> {
    let data = task.json(CATALOG, "Consultando versiones NeoForge")?;
    let legacy=task.json(catalog_url("1.20.1"),"Consultando NeoForge histórico")?;
    let mut supported: HashSet<_> = crate::neoforge_release::releases(&data)?.map(|release|release.minecraft).collect();
    if !stable_loaders(&legacy,"1.20.1")?.is_empty(){supported.insert("1.20.1".into());}
    Ok(minecraft.into_iter().filter(|version|supported.contains(&version.id)).collect())
}

fn relative_path(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() || value.len() > 1024 || value.starts_with('/') || value.contains('\\')
        || value.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || !value.bytes().all(|c|c.is_ascii_alphanumeric() || b"/_-.+".contains(&c)) {
        return Err("NeoForge contiene una ruta de biblioteca no segura".into());
    }
    Ok(PathBuf::from(value))
}

fn coordinate(value: &str) -> Result<PathBuf,String> {
    let (name, extension) = value.split_once('@').unwrap_or((value,"jar"));
    let parts: Vec<_> = name.split(':').collect();
    if !(3..=4).contains(&parts.len()) || extension.len()>32 || extension.split('.').any(|part|part.is_empty() || !part.bytes().all(|c|c.is_ascii_alphanumeric())) {
        return Err("Coordenadas de biblioteca NeoForge no válidas".into());
    }
    for part in &parts {
        if part.is_empty() || part.contains("..") || !part.bytes().all(|c|c.is_ascii_alphanumeric() || b"_-.+".contains(&c)) {
            return Err("Coordenadas de biblioteca NeoForge no seguras".into());
        }
    }
    let classifier = if parts.len()==4 {format!("-{}",parts[3])} else {String::new()};
    relative_path(&format!("{}/{}/{}/{}-{}{}.{}",parts[0].replace('.',"/"),parts[1],parts[2],parts[1],parts[2],classifier,extension))
}

fn valid_hash(hash: &str, length: usize) -> Result<(),String> {
    if hash.len()!=length || !hash.bytes().all(|c|c.is_ascii_hexdigit()) { return Err("Checksum NeoForge no válido".into()); }
    Ok(())
}

fn official_library(url: &str) -> Result<(),String> {
    let url = reqwest::Url::parse(url).map_err(|_|"URL de biblioteca NeoForge no válida")?;
    if url.scheme() != "https" || ![Some("maven.neoforged.net"),Some("libraries.minecraft.net")].contains(&url.host_str())
        || !url.username().is_empty() || url.password().is_some() || url.port().is_some() {
        return Err("La biblioteca NeoForge no procede de un repositorio oficial admitido".into());
    }
    Ok(())
}

fn read_entry(archive: &mut zip::ZipArchive<File>, name: &str) -> Result<Vec<u8>,String> {
    let mut entry = archive.by_name(name).map_err(|_|format!("Falta {name} en el instalador NeoForge"))?;
    if entry.size()>5_000_000 {return Err("Los metadatos NeoForge superan el límite de tamaño".into());}
    let mut bytes=Vec::new();entry.by_ref().take(5_000_001).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len()>5_000_000 {return Err("Los metadatos NeoForge superan el límite de tamaño".into());}
    Ok(bytes)
}

// Use NeoForge's official fat-installer resource layout. Only add the verified Mojang
// mapping cache; retain the original profile, processor code and hash checks unchanged.
fn offline_installer(original: &Path, target: &Path, mapping: &Path, minecraft: &str, task: &DownloadTask) -> Result<(),String> {
    let mut source=zip::ZipArchive::new(File::open(original).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let mut output=zip::ZipWriter::new(File::create(target).map_err(|e|e.to_string())?);
    if source.len()>20_000 {return Err("El instalador NeoForge tiene demasiadas entradas".into());}
    let cache=format!("maven/minecraft/{minecraft}/server_mappings.txt");relative_path(&cache)?;
    let mut expanded=0u64;
    for index in 0..source.len() {
        task.cancellation.check()?;
        let entry=source.by_index(index).map_err(|e|e.to_string())?;
        expanded=expanded.saturating_add(entry.size());
        if expanded>100_000_000 || entry.name()==cache {return Err("La estructura del instalador NeoForge no es compatible".into());}
        output.raw_copy_file(entry).map_err(|e|e.to_string())?;
    }
    output.start_file(cache,zip::write::SimpleFileOptions::default()).map_err(|e|e.to_string())?;
    std::io::copy(&mut File::open(mapping).map_err(|e|e.to_string())?,&mut output).map_err(|e|e.to_string())?;
    output.finish().map_err(|e|e.to_string())?;
    Ok(())
}

struct InstallerChild(Child);
impl Drop for InstallerChild {fn drop(&mut self) {let _=self.0.kill();let _=self.0.wait();}}

fn diagnostic(path: &Path) -> String {
    (|| -> std::io::Result<String> {
        let mut file=File::open(path)?;let size=file.metadata()?.len();file.seek(SeekFrom::Start(size.saturating_sub(4096)))?;
        let mut bytes=Vec::new();file.take(4096).read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"))
    })().unwrap_or_default()
}

fn run_installer(java: &Path, installer: &Path, stage: &Path, task: &DownloadTask) -> Result<(),String> {
    task.cancellation.check()?;
    task.report("Preparando bibliotecas NeoForge", "extracting");
    let temp=stage.join("tmp");fs::create_dir(&temp).map_err(|e|e.to_string())?;
    let log=stage.join("prepare.log");let output=File::create(&log).map_err(|e|e.to_string())?;
    let mut command=Command::new(java);
    #[cfg(windows)] {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
    let mut child=InstallerChild(command.arg("-Djava.awt.headless=true").arg(format!("-Djava.io.tmpdir={}",java_path(&temp)))
        .arg("-jar").arg(java_path(installer)).arg("--installServer").arg(java_path(stage)).arg("--offline")
        .current_dir(stage).stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output)
        .spawn().map_err(|e|format!("No se pudo iniciar la preparación NeoForge: {e}"))?);
    let began=Instant::now();
    loop {
        task.cancellation.check()?;
        for path in [&log,&installer.with_extension("jar.log")] {
            if fs::metadata(path).is_ok_and(|meta|meta.len()>16_000_000) {return Err("El registro del instalador NeoForge supera 16 MB".into());}
        }
        if let Some(status)=child.0.try_wait().map_err(|e|e.to_string())? {
            if !status.success() {return Err(format!("NeoForge no pudo preparar sus bibliotecas. {}",diagnostic(&log)));}
            break;
        }
        if began.elapsed()>Duration::from_secs(900) {return Err("NeoForge tardó más de 15 minutos en prepararse. Puedes volver a intentarlo.".into());}
        std::thread::sleep(Duration::from_millis(50));
    }
    fs::remove_dir_all(temp).map_err(|e|e.to_string())?;
    Ok(())
}

pub fn install(server: &Server, minecraft: &Value, java: &Path, task: &DownloadTask) -> Result<PathBuf,String> {
    install_requested(server,minecraft,java,task,None)
}

pub fn install_requested(server:&Server,minecraft:&Value,java:&Path,task:&DownloadTask,requested:Option<&str>)->Result<PathBuf,String> {
    let data=task.json(catalog_url(&server.version),"Buscando NeoForge compatible")?;
    let loader=requested_loader(&data,&server.version,requested)?;
    let server_root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let base=server_root.join(".tch-neoforge");
    if fs::symlink_metadata(&base).is_ok_and(|meta|meta.file_type().is_symlink()) {return Err("La carpeta NeoForge no puede ser un enlace simbólico".into());}
    fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    if !base.canonicalize().map_err(|e|e.to_string())?.starts_with(&server_root) {return Err("La carpeta NeoForge está fuera del servidor".into());}
    let generation=Uuid::new_v4();let stage=base.join(format!(".stage-{generation}"));let target=base.join(generation.to_string());
    fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let result=(|| {
        let release=crate::neoforge_release::Release::selected(&loader).ok_or("Versión NeoForge no válida")?;
        let url=release.installer_url();
        let hash=task.text(&format!("{url}.sha256"),"Consultando checksum NeoForge")?.trim().to_owned();valid_hash(&hash,64)?;
        let installer=stage.join("original-installer.jar");
        task.download(&url,&installer,Some(("sha256",&hash)),&format!("NeoForge {loader}"))?;
        if fs::metadata(&installer).map_err(|e|e.to_string())?.len()>32_000_000 {return Err("El instalador NeoForge supera 32 MB".into());}
        let mut archive=zip::ZipArchive::new(File::open(&installer).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        let profile:Value=serde_json::from_slice(&read_entry(&mut archive,"install_profile.json")?).map_err(|e|e.to_string())?;
        let version:Value=serde_json::from_slice(&read_entry(&mut archive,"version.json")?).map_err(|e|e.to_string())?;
        if profile["minecraft"].as_str()!=Some(&server.version) || version["inheritsFrom"].as_str()!=Some(&server.version)
            || profile["version"].as_str()!=Some(&release.profile_version()) {
            return Err("El instalador NeoForge no corresponde a la versión Minecraft seleccionada".into());
        }
        let server_jar=profile["serverJarPath"].as_str().ok_or("NeoForge no indicó el servidor base")?
            .strip_prefix("{LIBRARY_DIR}/").ok_or("Ruta del servidor base NeoForge no compatible")?.replace("{MINECRAFT_VERSION}",&server.version);
        let server_jar=stage.join("libraries").join(relative_path(&server_jar)?);
        fs::create_dir_all(server_jar.parent().unwrap()).map_err(|e|e.to_string())?;
        let artifact=&minecraft["downloads"]["server"];
        task.download(artifact["url"].as_str().ok_or("Minecraft no ofrece servidor")?,&server_jar,
            Some(("sha1",artifact["sha1"].as_str().ok_or("Minecraft no indicó checksum")?)),"Minecraft para NeoForge")?;
        let mut libraries=BTreeMap::new();
        for metadata in [&profile,&version] {
            let entries=metadata["libraries"].as_array().ok_or("Bibliotecas NeoForge no válidas")?;
            if entries.len()>1000 {return Err("NeoForge declara demasiadas bibliotecas".into());}
            for entry in entries {
                let artifact=&entry["downloads"]["artifact"];
                let name=entry["name"].as_str().ok_or("Biblioteca NeoForge sin nombre")?;
                let path=relative_path(artifact["path"].as_str().ok_or("Biblioteca NeoForge sin ruta")?)?;
                if path!=coordinate(name)? {return Err("La ruta NeoForge no corresponde a sus coordenadas".into());}
                let url=artifact["url"].as_str().ok_or("Biblioteca NeoForge sin URL")?;official_library(url)?;
                let hash=artifact["sha1"].as_str().ok_or("Biblioteca NeoForge sin checksum")?;valid_hash(hash,40)?;
                if let Some(previous)=libraries.insert(path,(url.to_owned(),hash.to_owned(),name.to_owned())) {
                    if previous.1!=hash {return Err("NeoForge declara bibliotecas con checksums incompatibles".into());}
                }
            }
        }
        for (path,(url,hash,name)) in libraries {
            let target=stage.join("libraries").join(path);fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;
            task.download(&url,&target,Some(("sha1",&hash)),&format!("NeoForge · {name}"))?;
        }
        let mappings=&minecraft["downloads"]["server_mappings"];
        let mapping_hash=mappings["sha1"].as_str().ok_or("Minecraft no proporciona mappings de servidor")?;valid_hash(mapping_hash,40)?;
        let mapping=stage.join("server-mappings.txt");
        task.download(mappings["url"].as_str().ok_or("Minecraft no proporciona mappings")?,&mapping,Some(("sha1",mapping_hash)),"Mappings oficiales de Minecraft")?;
        drop(archive);
        let offline=stage.join("installer.jar");offline_installer(&installer,&offline,&mapping,&server.version,task)?;
        run_installer(java,&offline,&stage,task)?;
        let args=stage.join(release.library_directory()).join(if cfg!(windows){"win_args.txt"}else{"unix_args.txt"});
        // Validate both the entrypoint and every referenced library before publication.
        launch_file(&server_root,&stage,&args,&server.version)?;
        fs::write(stage.join("installation.json"),serde_json::to_vec_pretty(&serde_json::json!({"minecraft":server.version,"loader":loader,"installerSha256":hash})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        for path in [installer,offline,mapping] {fs::remove_file(path).map_err(|e|e.to_string())?;}
        task.cancellation.check()?;
        let relative=args.strip_prefix(&stage).map_err(|e|e.to_string())?.to_owned();
        fs::rename(&stage,&target).map_err(|e|e.to_string())?;
        Ok(target.join(relative))
    })();
    if result.is_err() {let _=fs::remove_dir_all(&stage);}
    result
}

// A bounded Java argument-file tokenizer; shell expansion and nested @files are never used.
fn tokens(text: &str) -> Result<Vec<String>,String> {
    let mut output=Vec::new();let mut word=String::new();let mut quote=None;let mut comment=false;
    for ch in text.chars() {
        if ch=='\0' {return Err("Archivo de arranque NeoForge no válido".into());}
        if comment {if ch=='\n' {comment=false;}continue;}
        if let Some(delimiter)=quote {
            if ch==delimiter {quote=None;} else {word.push(ch);}continue;
        }
        match ch {
            '"'|'\''=>quote=Some(ch),
            '#' if word.is_empty()=>comment=true,
            ch if ch.is_whitespace()=>{if !word.is_empty() {output.push(std::mem::take(&mut word));}},
            _=>word.push(ch),
        }
    }
    if quote.is_some() {return Err("Comillas sin cerrar en el arranque NeoForge".into());}
    if !word.is_empty() {output.push(word);}
    if output.len()>256 || output.iter().any(|word|word.len()>MAX_TEXT as usize || word.chars().any(char::is_control)) {return Err("Archivo de arranque NeoForge demasiado complejo".into());}
    Ok(output)
}

fn rooted_library(root: &Path, value: &str) -> Result<String,String> {
    let path=relative_path(value)?;
    if !value.starts_with("libraries/") {return Err("La ruta NeoForge debe estar dentro de libraries".into());}
    let library=root.join(path).canonicalize().map_err(|_|format!("Falta una biblioteca NeoForge: {value}"))?;
    if !library.is_file() || !library.starts_with(root.canonicalize().map_err(|e|e.to_string())?) {return Err("Una biblioteca NeoForge está fuera del servidor".into());}
    Ok(java_path(&library))
}

fn rooted_classpath(root: &Path, value: &str, separator: char) -> Result<String,String> {
    value.split(separator).map(|part|rooted_library(root,part)).collect::<Result<Vec<_>,_>>()
        .map(|paths|paths.join(if cfg!(windows){";"}else{":"}))
}

fn launch_file(server_root: &Path, installation: &Path, path: &Path, minecraft: &str) -> Result<Vec<String>,String> {
    if fs::metadata(path).map_err(|_|"Falta el archivo de arranque NeoForge")?.len()>MAX_TEXT {return Err("El archivo de arranque NeoForge supera 128 KB".into());}
    let path=path.canonicalize().map_err(|e|e.to_string())?;
    if !path.starts_with(server_root.canonicalize().map_err(|e|e.to_string())?) {return Err("El arranque NeoForge está fuera del servidor".into());}
    let text=fs::read_to_string(&path).map_err(|e|e.to_string())?;
    let parsed=tokens(&text)?;let separator=if path.file_name().is_some_and(|name|name=="win_args.txt") {';'} else {':'};
    let mut args=Vec::new();let mut index=0;let mut entrypoint=false;let mut version=false;let mut target=false;
    while index<parsed.len() {
        let arg=&parsed[index];
        let next=||parsed.get(index+1).ok_or("Falta un valor en el arranque NeoForge");
        if ["-p","--module-path","-cp","-classpath","--class-path"].contains(&arg.as_str()) && !entrypoint {
            args.extend([arg.clone(),rooted_classpath(installation,next()?,separator)?]);index+=2;continue;
        }
        if ["--add-modules","--add-opens","--add-exports","--enable-native-access"].contains(&arg.as_str()) && !entrypoint {
            let value=next()?;if value.starts_with('-') || value.contains(char::is_whitespace) {return Err("Opción modular NeoForge no válida".into());}
            args.extend([arg.clone(),value.clone()]);index+=2;continue;
        }
        if let Some(value)=arg.strip_prefix("-DlegacyClassPath=") {
            if entrypoint {return Err("Propiedad NeoForge fuera de la sección JVM".into());}
            args.push(format!("-DlegacyClassPath={}",rooted_classpath(installation,value,separator)?));
        } else if arg=="-DlibraryDirectory=libraries" && !entrypoint {
            args.push(format!("-DlibraryDirectory={}",java_path(&installation.join("libraries"))));
        } else if !entrypoint && ["-DignoreList=","-Dfml.pluginLayerLibraries=","-Dfml.gameLayerLibraries=","-Djava.net.preferIPv6Addresses="].iter().any(|prefix|arg.starts_with(prefix)) {
            args.push(arg.clone());
        } else if ["cpw.mods.bootstraplauncher.BootstrapLauncher","net.neoforged.fml.startup.Server"].contains(&arg.as_str()) && !entrypoint {
            // Modern FML identifies the server through its dedicated entrypoint.
            // Older BootstrapLauncher profiles still require --launchTarget.
            target=arg=="net.neoforged.fml.startup.Server";
            args.push(arg.clone());entrypoint=true;
        } else if entrypoint && ["--launchTarget","--fml.neoForgeVersion","--fml.forgeVersion","--fml.fmlVersion","--fml.mcVersion","--fml.neoFormVersion","--fml.mcpVersion"].contains(&arg.as_str()) {
            let value=next()?;
            if arg=="--fml.mcVersion" {if value!=minecraft {return Err("La versión Minecraft del arranque NeoForge no coincide".into());}version=true;}
            if arg=="--launchTarget" {if !["neoforgeserver","forgeserver"].contains(&value.as_str()) {return Err("NeoForge no tiene un arranque de servidor".into());}target=true;}
            if value.starts_with('-') || value.contains(char::is_whitespace) {return Err("Valor de arranque NeoForge no válido".into());}
            args.extend([arg.clone(),value.clone()]);index+=2;continue;
        } else {return Err(format!("Formato de arranque NeoForge no compatible: {arg}"));}
        index+=1;
    }
    if !entrypoint || !version || !target {return Err("El archivo NeoForge no identifica un servidor Minecraft válido".into());}
    args.push("nogui".into());Ok(args)
}

pub fn launch_arguments(server: &Server) -> Result<Vec<String>,String> {
    resolve_launch(Path::new(&server.directory),Path::new(server.jar_path.as_deref().ok_or("Instala NeoForge antes de iniciar")?),&server.version)
}

pub fn validate_import(root: &Path, relative: &str, minecraft: &str) -> Result<(),String> {
    resolve_launch(root,&root.join(relative_path(relative)?),minecraft).map(|_|())
}

fn resolve_launch(directory: &Path, configured: &Path, minecraft: &str) -> Result<Vec<String>,String> {
    let root=directory.canonicalize().map_err(|e|e.to_string())?;
    let file=configured.file_name().ok_or("Archivo de arranque NeoForge no válido")?;
    if file!="win_args.txt" && file!="unix_args.txt" {return Err("Selecciona el archivo de argumentos NeoForge, no un .jar o un script".into());}
    let path=configured.with_file_name(if cfg!(windows){"win_args.txt"}else{"unix_args.txt"}).canonicalize().map_err(|_|"Falta el archivo de arranque NeoForge para este sistema")?;
    let relative=path.strip_prefix(&root).map_err(|_|"El archivo NeoForge está fuera del servidor")?;
    let components:Vec<_>=relative.components().collect();
    let installation=if relative.starts_with(".tch-neoforge") {
        let generation=components.get(1).ok_or("Falta la generación NeoForge")?.as_os_str().to_str().ok_or("Generación NeoForge no válida")?;
        Uuid::parse_str(generation).map_err(|_|"Generación NeoForge no válida")?;root.join(".tch-neoforge").join(generation)
    } else {root.clone()};
    launch_file(&root,&installation,&path,minecraft)
}

pub fn discover(root: &Path) -> Vec<String> {
    let mut candidates=Vec::new();let mut bases=vec![root.to_owned()];
    if let Ok(generations)=fs::read_dir(root.join(".tch-neoforge")) {
        for entry in generations.take(1000).filter_map(Result::ok) {
            if Uuid::parse_str(&entry.file_name().to_string_lossy()).is_ok() {bases.push(entry.path());}
        }
    }
    for base in bases {
        for artifact in ["neoforge","forge"] {
            if let Ok(loaders)=fs::read_dir(base.join("libraries/net/neoforged").join(artifact)) {
                for entry in loaders.take(1000).filter_map(Result::ok) {
                    let name=entry.file_name().to_string_lossy().into_owned();
                    if !crate::neoforge_release::Release::selected(&name).is_some_and(|release|release.artifact==artifact){continue;}
                    let path=entry.path().join(if cfg!(windows){"win_args.txt"}else{"unix_args.txt"});
                    if path.is_file()&&path.canonicalize().is_ok_and(|path|path.starts_with(root)) {
                        if let Ok(relative)=path.strip_prefix(root){candidates.push(relative.to_string_lossy().replace('\\',"/"));}
                    }
                }
            }
        }
    }
    candidates
}

pub fn detected_version(path: &Path) -> Option<String> {
    if fs::metadata(path).ok()?.len()>MAX_TEXT {return None;}
    let parsed=tokens(&fs::read_to_string(path).ok()?).ok()?;
    let index=parsed.iter().position(|arg|arg=="--fml.mcVersion")?;
    let version=parsed.get(index+1)?;crate::fabric::validate_version(version).ok()?;Some(version.clone())
}
