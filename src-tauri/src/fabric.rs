use crate::{download::DownloadTask,model::Server};
use serde_json::Value;
use std::fs::{self,File};
use std::io::{Read,Write};
use std::path::{Path,PathBuf};
use uuid::Uuid;

pub fn validate_version(version:&str)->Result<(),String> {
    if version.is_empty() || version.len()>64 || version.starts_with('.') || !version.bytes().all(|c|c.is_ascii_alphanumeric() || b".-+_".contains(&c)) {
        return Err("Versión de Fabric no válida".into());
    }
    Ok(())
}

pub fn select_loader(meta:&Value)->Result<String,String> {
    let version=meta.as_array().ok_or("Catálogo Fabric no válido")?.iter()
        .find(|entry|entry["loader"]["stable"]==true)
        .and_then(|entry|entry["loader"]["version"].as_str())
        .ok_or("Fabric no ofrece un loader estable compatible con esta versión")?;
    validate_version(version)?;
    Ok(version.into())
}

pub fn requested_loader(meta:&Value,requested:Option<&str>)->Result<String,String> {
    let Some(version)=requested else {return select_loader(meta);};
    validate_version(version)?;
    if !meta.as_array().ok_or("Catálogo Fabric no válido")?.iter().any(|entry|entry["loader"]["version"].as_str()==Some(version)) {
        return Err(format!("Fabric {version} no está disponible para esta versión de Minecraft"));
    }
    Ok(version.into())
}

fn library_path(name:&str)->Result<String,String> {
    let parts=name.split(':').collect::<Vec<_>>();
    if parts.len()!=3 || name.len()>256 {return Err("Coordenada Maven no válida".into());}
    for part in &parts {validate_version(part)?;}
    if parts[0].split('.').any(|part|part.is_empty()) {return Err("Grupo Maven no válido".into());}
    Ok(format!("{}/{}/{}/{}-{}.jar",parts[0].replace('.',"/"),parts[1],parts[2],parts[1],parts[2]))
}

fn java_class(name:&str)->Result<(),String> {
    if name.len()>200 || name.split('.').any(|part|part.is_empty() || !part.bytes().all(|c|c.is_ascii_alphanumeric() || b"_$".contains(&c))) {
        return Err("Clase de arranque Fabric no válida".into());
    }
    Ok(())
}

fn manifest_attribute(output:&mut String,name:&str,value:&str) {
    let line=format!("{name}: {value}");
    // Values are ASCII; JAR manifest physical lines are limited to 72 bytes.
    for (index,chunk) in line.as_bytes().chunks(70).enumerate() {
        if index>0 {output.push(' ');}
        output.push_str(std::str::from_utf8(chunk).unwrap());output.push_str("\r\n");
    }
}

fn loader_main_class(path:&Path)->Result<String,String> {
    let mut jar=zip::ZipArchive::new(File::open(path).map_err(|e|e.to_string())?).map_err(|_|"El loader Fabric no es un JAR válido")?;
    let entry=jar.by_name("META-INF/MANIFEST.MF").map_err(|_|"Fabric no tiene manifiesto de arranque")?;
    if entry.size()>1_048_576 {return Err("Manifiesto Fabric demasiado grande".into());}
    let mut manifest=String::new();entry.take(1_048_577).read_to_string(&mut manifest).map_err(|e|e.to_string())?;
    let unfolded=manifest.replace("\r\n ","").replace("\n ","");
    let main=unfolded.lines().find_map(|line|line.strip_prefix("Main-Class: ")).ok_or("Fabric no declaró su clase de arranque")?;
    java_class(main)?;Ok(main.into())
}

fn launcher(path:&Path,main:&str,game_main:&str,libraries:&[String])->Result<(),String> {
    use zip::write::SimpleFileOptions;
    java_class(main)?;java_class(game_main)?;
    let mut manifest=String::new();
    manifest_attribute(&mut manifest,"Manifest-Version","1.0");
    manifest_attribute(&mut manifest,"Main-Class",main);
    manifest_attribute(&mut manifest,"Class-Path",&libraries.join(" "));
    manifest.push_str("\r\n");
    let mut jar=zip::ZipWriter::new(File::create(path).map_err(|e|e.to_string())?);
    jar.start_file("META-INF/MANIFEST.MF",SimpleFileOptions::default()).map_err(|e|e.to_string())?;
    jar.write_all(manifest.as_bytes()).map_err(|e|e.to_string())?;
    jar.start_file("fabric-server-launch.properties",SimpleFileOptions::default()).map_err(|e|e.to_string())?;
    jar.write_all(format!("launch.mainClass={game_main}\n").as_bytes()).map_err(|e|e.to_string())?;
    jar.finish().map_err(|e|e.to_string())?.sync_all().map_err(|e|e.to_string())
}

pub fn install(server:&Server,minecraft:&Value,task:&DownloadTask)->Result<PathBuf,String> {
    install_requested(server,minecraft,task,None)
}

pub fn install_requested(server:&Server,minecraft:&Value,task:&DownloadTask,requested:Option<&str>)->Result<PathBuf,String> {
    validate_version(&server.version)?;
    let catalog=task.json(&format!("https://meta.fabricmc.net/v2/versions/loader/{}",server.version),"Buscando Fabric compatible")?;
    let loader=requested_loader(&catalog,requested)?;
    let profile=task.json(&format!("https://meta.fabricmc.net/v2/versions/loader/{}/{loader}/server/json",server.version),"Consultando bibliotecas Fabric")?;
    if profile["inheritsFrom"].as_str()!=Some(&server.version) {return Err("El perfil Fabric no coincide con Minecraft".into());}
    let game_main=profile["mainClass"].as_str().ok_or("Fabric no declaró la clase principal")?;
    java_class(game_main)?;
    let entries=profile["libraries"].as_array().filter(|entries|!entries.is_empty() && entries.len()<=100).ok_or("Lista de bibliotecas Fabric no válida")?;
    let base=Path::new(&server.directory).join(".tch-fabric");
    fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    if fs::symlink_metadata(&base).map_err(|e|e.to_string())?.file_type().is_symlink() || base.canonicalize().map_err(|e|e.to_string())? != Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?.join(".tch-fabric") {return Err("La carpeta Fabric debe permanecer dentro del servidor".into());}
    let generation=Uuid::new_v4().to_string();
    let stage=base.join(format!(".stage-{generation}"));let target=base.join(&generation);
    fs::create_dir(&stage).map_err(|e|e.to_string())?;
    let result=(|| {
        let mut classpath=Vec::new();let mut main=None;
        for entry in entries {
            task.cancellation.check()?;
            let name=entry["name"].as_str().ok_or("Biblioteca Fabric sin nombre")?;
            let relative=library_path(name)?;
            if entry["url"].as_str()!=Some("https://maven.fabricmc.net/") {return Err("Biblioteca Fabric fuera del repositorio oficial".into());}
            let url=format!("https://maven.fabricmc.net/{relative}");
            let hash=if let Some(hash)=entry["sha256"].as_str() {hash.into()} else {task.text(&format!("{url}.sha256"),"Consultando checksum Fabric")?.trim().to_string()};
            if hash.len()!=64 || !hash.bytes().all(|c|c.is_ascii_hexdigit()) {return Err("Checksum Fabric no válido".into());}
            let relative=format!("libraries/{relative}");let destination=stage.join(&relative);
            fs::create_dir_all(destination.parent().unwrap()).map_err(|e|e.to_string())?;
            task.download(&url,&destination,Some(("sha256",&hash)),&format!("Fabric · {name}"))?;
            if name.starts_with("net.fabricmc:fabric-loader:") {
                if name!=format!("net.fabricmc:fabric-loader:{loader}") || main.is_some() {return Err("El perfil Fabric no corresponde al loader solicitado".into());}
                main=Some(loader_main_class(&destination)?);
            }
            classpath.push(relative);
        }
        let artifact=&minecraft["downloads"]["server"];
        task.download(artifact["url"].as_str().ok_or("Minecraft no proporciona servidor")?,&stage.join("vanilla.jar"),Some(("sha1",artifact["sha1"].as_str().ok_or("Minecraft no proporciona checksum")?)),"Servidor Minecraft para Fabric")?;
        task.report("Preparando arranque Fabric","verifying");
        launcher(&stage.join("fabric-server-launch.jar"),main.as_deref().ok_or("Falta Fabric Loader")?,game_main,&classpath)?;
        fs::write(stage.join("installation.json"),serde_json::to_vec_pretty(&serde_json::json!({"minecraft":server.version,"loader":loader,"libraries":classpath})).unwrap()).map_err(|e|e.to_string())?;
        task.cancellation.check()?;
        fs::rename(&stage,&target).map_err(|e|e.to_string())?;
        Ok(target.join("fabric-server-launch.jar"))
    })();
    if result.is_err() {let _=fs::remove_dir_all(&stage);}
    result
}

pub fn game_jar(server:&Server)->Option<PathBuf> {
    if server.platform!="fabric" {return None;}
    let parent=Path::new(server.jar_path.as_ref()?).parent()?;
    (parent.parent()?==Path::new(&server.directory).join(".tch-fabric")).then(||parent.join("vanilla.jar"))
}
