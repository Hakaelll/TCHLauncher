use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::{BTreeMap, HashMap, HashSet}, fs, io::Read, path::Path};

const MAX_ARCHIVE:u64=512_000_000;
const MAX_EXPANDED:u64=2_000_000_000;
const MAX_INDEX:u64=5_000_000;

#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
pub struct PackIndex {
    pub format_version:u32,
    pub game:String,
    pub version_id:String,
    pub name:String,
    pub summary:Option<String>,
    pub files:Vec<PackFile>,
    pub dependencies:BTreeMap<String,String>,
}

#[derive(Clone,Debug,Deserialize,Serialize)]
pub struct Environment {pub client:String,pub server:String}

#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase")]
pub struct PackFile {
    pub path:String,
    pub hashes:BTreeMap<String,String>,
    pub env:Option<Environment>,
    pub downloads:Vec<String>,
    pub file_size:u64,
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct PackEntry {
    pub path:String,
    pub size:u64,
    pub source:String,
    pub server:String,
    pub selected:bool,
    pub replaces_common:bool,
}

#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct PackInspection {
    pub path:String,
    pub archive_sha512:String,
    pub name:String,
    pub version_id:String,
    pub summary:Option<String>,
    pub minecraft:String,
    pub loader:String,
    pub loader_version:Option<String>,
    pub entries:Vec<PackEntry>,
    pub excluded_client_files:usize,
    pub selected_bytes:u64,
    pub blockers:Vec<String>,
    pub warnings:Vec<String>,
}

pub struct InspectedPack {pub index:PackIndex,pub inspection:PackInspection}

// Validate names on every supported OS, before joining them onto an instance directory.
pub fn relative_path(raw:&str)->Result<String,String> {
    if raw.is_empty() || raw.len()>512 || raw.starts_with('/') || raw.contains('\\') {
        return Err(format!("Ruta de paquete no válida: {raw}"));
    }
    for part in raw.split('/') {
        if part.is_empty() || matches!(part,"."|"..") || part.len()>180 || part.ends_with('.') || part.ends_with(' ') ||
            part.chars().any(|c|c.is_control() || matches!(c,':'|'<'|'>'|'"'|'|'|'?'|'*')) {
            return Err(format!("Ruta de paquete no válida: {raw}"));
        }
        let stem=part.split('.').next().unwrap_or_default().to_ascii_uppercase();
        if matches!(stem.as_str(),"CON"|"PRN"|"AUX"|"NUL") ||
            (stem.len()==4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && matches!(stem.as_bytes()[3],b'1'..=b'9')) {
            return Err(format!("La ruta usa un nombre reservado: {raw}"));
        }
    }
    Ok(raw.into())
}

fn server_environment(file:&PackFile)->Result<&str,String> {
    let Some(environment)=&file.env else {return Ok("required");};
    if !matches!(environment.client.as_str(),"required"|"optional"|"unsupported") || !matches!(environment.server.as_str(),"required"|"optional"|"unsupported") {
        return Err(format!("Entorno de archivo no válido: {}",file.path));
    }
    Ok(&environment.server)
}

fn download_url(raw:&str)->Result<(),String> {
    if raw.chars().any(|c|c.is_whitespace() || c.is_control()) {return Err("La URL contiene caracteres sin codificar".into());}
    let url=reqwest::Url::parse(raw).map_err(|_|"URL de modpack no válida")?;
    if url.scheme()!="https" || !url.username().is_empty() || url.password().is_some() || url.port().is_some() ||
        !matches!(url.host_str(),Some("cdn.modrinth.com"|"github.com"|"raw.githubusercontent.com"|"gitlab.com")) {
        return Err(format!("La descarga no pertenece a un origen admitido: {raw}"));
    }
    Ok(())
}

fn index_valid(index:&PackIndex)->Result<(),String> {
    if index.format_version!=1 || index.game!="minecraft" {return Err("Este formato o juego de modpack no está admitido".into());}
    if index.name.trim().is_empty() || index.name.len()>256 || index.version_id.is_empty() || index.version_id.len()>128 || index.summary.as_ref().is_some_and(|s|s.len()>8192) || index.files.len()>4096 {
        return Err("Los metadatos del modpack no son válidos".into());
    }
    let minecraft=index.dependencies.get("minecraft").ok_or("El modpack no declara una versión de Minecraft")?;
    if !version_string(minecraft) {return Err("La versión de Minecraft del modpack no es válida".into());}
    let mut names=HashSet::new();let mut size=0u64;
    for file in &index.files {
        relative_path(&file.path)?;
        if !names.insert(file.path.to_lowercase()) {return Err(format!("El modpack repite una ruta: {}",file.path));}
        server_environment(file)?;
        for (algorithm,length) in [("sha1",40),("sha512",128)] {
            let hash=file.hashes.get(algorithm).ok_or_else(||format!("Falta {algorithm} en {}",file.path))?;
            if hash.len()!=length || !hash.bytes().all(|c|c.is_ascii_hexdigit()) {return Err(format!("Hash {algorithm} no válido en {}",file.path));}
        }
        if file.file_size>512_000_000 || file.downloads.is_empty() || file.downloads.len()>8 {return Err(format!("Tamaño o descargas no válidas: {}",file.path));}
        for url in &file.downloads {download_url(url)?;}
        size=size.checked_add(file.file_size).ok_or("El modpack es demasiado grande")?;
        if size>16_000_000_000 {return Err("El modpack declara más de 16 GB de descargas".into());}
    }
    // A path cannot be a regular file and the parent of another regular file.
    for name in &names {
        let mut parent=name.as_str();
        while let Some((prefix,_))=parent.rsplit_once('/') {if names.contains(prefix) {return Err(format!("Colisión entre archivo y carpeta: {prefix}"));}parent=prefix;}
    }
    Ok(())
}

fn version_string(value:&str)->bool { !value.is_empty() && value.len()<=128 && value.bytes().all(|c|c.is_ascii_alphanumeric() || matches!(c,b'.'|b'-'|b'_'|b'+')) }

fn protected(path:&str)->bool {
    let value=path.to_lowercase();
    let first=value.split('/').next().unwrap_or_default();
    value.split('/').any(|part|part.starts_with(".tch-")) || matches!(first,"eula.txt"|"server.properties"|"usercache.json"|"whitelist.json"|"ops.json"|"banned-players.json"|"banned-ips.json"|"server-icon.png"|"server.jar"|"libraries"|"logs"|"backups"|"world"|"world_nether"|"world_the_end")
}

fn client_path(path:&str)->bool {
    let value=path.to_lowercase();
    matches!(value.split('/').next().unwrap_or_default(),"resourcepacks"|"shaderpacks"|"screenshots") || matches!(value.as_str(),"options.txt"|"optionsof.txt"|"servers.dat")
}

pub fn inspect(path:&Path)->Result<InspectedPack,String> {
    let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len()>MAX_ARCHIVE || !path.extension().is_some_and(|e|e.eq_ignore_ascii_case("mrpack")) {
        return Err("Selecciona un archivo .mrpack de hasta 512 MB".into());
    }
    let mut archive_file=fs::File::open(path).map_err(|e|e.to_string())?;let mut digest=Sha512::new();let mut buffer=[0u8;65536];let mut received=0;
    loop {let n=archive_file.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break;}received+=n as u64;if received>MAX_ARCHIVE {return Err("El modpack supera 512 MB".into());}digest.update(&buffer[..n]);}
    let archive_sha512=format!("{:x}",digest.finalize());
    let mut archive=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|_|"El archivo no es un ZIP mrpack válido")?;
    if archive.len()>10000 {return Err("El modpack contiene demasiados archivos".into());}
    let mut expanded=0u64;let mut names=HashSet::new();let mut override_entries:HashMap<String,PackEntry>=HashMap::new();let mut server_entries=vec![];let mut excluded=0;
    let mut index_data=None;
    for offset in 0..archive.len() {
        let mut file=archive.by_index(offset).map_err(|e|e.to_string())?;
        let raw=file.name().trim_end_matches('/').to_string();let name=relative_path(&raw)?;
        if !names.insert(name.to_lowercase()) {return Err(format!("El ZIP repite una ruta: {name}"));}
        if file.unix_mode().is_some_and(|mode|!matches!(mode&0o170000,0|0o040000|0o100000)) {return Err("El modpack contiene enlaces o archivos especiales".into());}
        expanded=expanded.checked_add(file.size()).ok_or("El ZIP es demasiado grande")?;
        if expanded>MAX_EXPANDED || file.size()>512_000_000 {return Err("El ZIP declara más de 2 GB de contenido o un archivo demasiado grande".into());}
        if file.is_dir() {continue;}
        if name=="modrinth.index.json" {
            if file.size()>MAX_INDEX {return Err("El índice del modpack supera 5 MB".into());}
            let mut data=vec![];(&mut file).take(MAX_INDEX+1).read_to_end(&mut data).map_err(|e|e.to_string())?;
            if data.len() as u64>MAX_INDEX {return Err("El índice del modpack supera 5 MB".into());}index_data=Some(data);continue;
        }
        if name.starts_with("client-overrides/") {excluded+=1;continue;}
        let (layer,relative)=if let Some(relative)=name.strip_prefix("overrides/") {("overrides",relative)}
            else if let Some(relative)=name.strip_prefix("server-overrides/") {("server-overrides",relative)} else {continue;};
        relative_path(relative)?;
        if client_path(relative) {excluded+=1;continue;}
        let entry=PackEntry {path:relative.into(),size:file.size(),source:layer.into(),server:"required".into(),selected:!protected(relative),replaces_common:false};
        if layer=="server-overrides" {server_entries.push(entry);}else {override_entries.insert(relative.to_lowercase(),entry);}
    }
    let index:PackIndex=serde_json::from_slice(&index_data.ok_or("Falta modrinth.index.json en la raíz del paquete")?).map_err(|e|format!("Índice de modpack no válido: {e}"))?;
    index_valid(&index)?;
    for mut entry in server_entries {entry.replaces_common=override_entries.contains_key(&entry.path.to_lowercase());override_entries.insert(entry.path.to_lowercase(),entry);}
    let mut blockers=vec![];let mut warnings=vec![];
    let loader_dependencies:Vec<_>=index.dependencies.iter().filter(|(id,_)|id.as_str()!="minecraft").collect();
    let (loader,loader_version)=if loader_dependencies.is_empty() {("vanilla".to_string(),None)} else if loader_dependencies.len()!=1 {
        blockers.push("El modpack declara varios loaders o dependencias de motor desconocidas".into());("unknown".into(),None)
    } else {
        let (id,version)=loader_dependencies[0];if !version_string(version) {return Err("Versión de loader no válida".into());}
        let loader=match id.as_str() {"fabric-loader"=>"fabric","neoforge"=>"neoforge","forge"=>"forge","quilt-loader"=>"quilt",_=>"unknown"};
        if !matches!(loader,"fabric"|"neoforge") {blockers.push(format!("Este paquete necesita {id}; el motor todavía no puede prepararlo automáticamente."));}
        (loader.into(),Some(version.clone()))
    };
    let mut entries=vec![];
    for file in &index.files {
        let environment=server_environment(file)?;
        if environment=="unsupported" || client_path(&file.path) {excluded+=1;continue;}
        if protected(&file.path) {warnings.push(format!("Se conservará el archivo del servidor: {}",file.path));}
        entries.push(PackEntry {path:file.path.clone(),size:file.file_size,source:"download".into(),server:environment.into(),selected:environment=="required"&&!protected(&file.path),replaces_common:false});
    }
    for entry in override_entries.values() {
        if protected(&entry.path) {warnings.push(format!("Se conservará el archivo del servidor: {}",entry.path));}
        if let Some(previous)=entries.iter().position(|file|file.path.eq_ignore_ascii_case(&entry.path)) {entries.remove(previous);}
        entries.push(entry.clone());
    }
    entries.sort_by(|a,b|a.path.cmp(&b.path));
    let all:HashSet<_>=entries.iter().map(|entry|entry.path.to_lowercase()).collect();
    for path in &all {let mut parent=path.as_str();while let Some((prefix,_))=parent.rsplit_once('/') {if all.contains(prefix) {return Err(format!("El paquete usa la misma ruta como archivo y carpeta: {prefix}"));}parent=prefix;}}
    if loader=="vanilla" && entries.iter().any(|entry|entry.path.starts_with("mods/")&&entry.server=="required") {blockers.push("El paquete incluye mods obligatorios pero no declara un loader.".into());}
    if entries.iter().any(|entry|entry.path.ends_with(".bat")||entry.path.ends_with(".sh")||entry.path.ends_with(".exe")) {warnings.push("El paquete incluye programas o scripts. El launcher no los ejecutará.".into());}
    let selected_bytes=entries.iter().filter(|entry|entry.selected).map(|entry|entry.size).sum();
    let inspection=PackInspection {path:path.canonicalize().map_err(|e|e.to_string())?.to_string_lossy().into(),archive_sha512,name:index.name.clone(),version_id:index.version_id.clone(),summary:index.summary.clone(),minecraft:index.dependencies["minecraft"].clone(),loader,loader_version,entries,excluded_client_files:excluded,selected_bytes,blockers,warnings};
    Ok(InspectedPack {index,inspection})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    struct Pack(std::path::PathBuf);
    impl Drop for Pack {fn drop(&mut self) {let _=fs::remove_file(&self.0);}}
    fn create(index:serde_json::Value,files:&[(&str,&str)])->Pack {
        let path=std::env::temp_dir().join(format!("tch-mrpack-{}.mrpack",uuid::Uuid::new_v4()));let mut zip=zip::ZipWriter::new(fs::File::create(&path).unwrap());
        let options=zip::write::SimpleFileOptions::default();zip.start_file("modrinth.index.json",options).unwrap();zip.write_all(index.to_string().as_bytes()).unwrap();
        for (name,body) in files {zip.start_file(*name,options).unwrap();zip.write_all(body.as_bytes()).unwrap();}zip.finish().unwrap();Pack(path)
    }
    fn file(path:&str,server:&str)->serde_json::Value {serde_json::json!({"path":path,"hashes":{"sha1":"a".repeat(40),"sha512":"b".repeat(128)},"env":{"client":"required","server":server},"downloads":["https://cdn.modrinth.com/data/project/version/mod.jar"],"fileSize":123})}
    fn index()->serde_json::Value {serde_json::json!({"formatVersion":1,"game":"minecraft","versionId":"1.0","name":"Test pack","files":[],"dependencies":{"minecraft":"1.21.4","fabric-loader":"0.16.10"}})}
    #[test]
    fn rejects_traversal_absolute_paths_device_names_and_bad_sources() {
        for path in ["../escape","/absolute","C:/outside","config/../../outside","mods\\evil.jar","mods/CON.jar","mods/test.jar.","config//value"] {assert!(relative_path(path).is_err(),"{path}");}
        assert!(relative_path("config/example/config.json").is_ok());
        for url in ["http://cdn.modrinth.com/file","https://user:pass@github.com/file","https://localhost/file","https://example.com/file","https://github.com/a file"] {assert!(download_url(url).is_err());}
    }
    #[test]
    fn selects_required_server_files_preserves_settings_and_applies_server_layer() {
        let mut data=index();data["files"]=serde_json::json!([file("mods/required.jar","required"),file("mods/optional.jar","optional"),file("mods/client.jar","unsupported")]);
        let pack=create(data,&[("overrides/config/settings.json","common"),("server-overrides/config/settings.json","server"),("server-overrides/eula.txt","eula=true"),("client-overrides/options.txt","client")]);
        let result=inspect(&pack.0).unwrap();assert_eq!(result.index.dependencies["fabric-loader"],"0.16.10");assert!(result.inspection.blockers.is_empty());
        let entries=&result.inspection.entries;
        assert!(entries.iter().find(|entry|entry.path=="mods/required.jar").unwrap().selected);
        assert!(!entries.iter().find(|entry|entry.path=="mods/optional.jar").unwrap().selected);
        assert!(!entries.iter().any(|entry|entry.path=="mods/client.jar"));
        let server=entries.iter().find(|entry|entry.path=="config/settings.json").unwrap();assert_eq!(server.source,"server-overrides");assert!(server.replaces_common);
        assert!(!entries.iter().find(|entry|entry.path=="eula.txt").unwrap().selected);assert_eq!(result.inspection.excluded_client_files,2);
    }
    #[test]
    fn rejects_unsafe_zip_names_duplicate_paths_and_invalid_hashes() {
        let pack=create(index(),&[("overrides/../outside","unsafe")]);assert!(inspect(&pack.0).is_err());
        let pack=create(index(),&[("overrides/config/A.json","one"),("overrides/config/a.json","two")]);assert!(inspect(&pack.0).is_err());
        let mut data=index();let mut bad=file("mods/example.jar","required");bad["hashes"]["sha512"]="bad".into();data["files"]=serde_json::json!([bad]);let pack=create(data,&[]);assert!(inspect(&pack.0).is_err());
    }
    #[test]
    fn reports_unsupported_engine_without_claiming_server_compatibility() {
        let mut data=index();data["dependencies"]=serde_json::json!({"minecraft":"1.21.4","quilt-loader":"0.28.0"});let pack=create(data,&[]);
        let result=inspect(&pack.0).unwrap();assert_eq!(result.inspection.loader,"quilt");assert!(!result.inspection.blockers.is_empty());
        let mut data=index();data["dependencies"]=serde_json::json!({"minecraft":"1.21.4"});data["files"]=serde_json::json!([file("mods/required.jar","required")]);let pack=create(data,&[]);assert!(!inspect(&pack.0).unwrap().inspection.blockers.is_empty());
    }
}
