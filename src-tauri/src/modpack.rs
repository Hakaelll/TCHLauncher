use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::{BTreeMap, HashMap, HashSet}, fs, io::{Read,Write}, path::{Path,PathBuf}};

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
    pub protected:bool,
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
    crate::download::validate_pack_url(raw,false)
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
        let entry=PackEntry {path:relative.into(),size:file.size(),source:layer.into(),server:"required".into(),selected:!protected(relative),replaces_common:false,protected:protected(relative)};
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
        entries.push(PackEntry {path:file.path.clone(),size:file.file_size,source:"download".into(),server:environment.into(),selected:environment=="required"&&!protected(&file.path),replaces_common:false,protected:protected(&file.path)});
    }
    for entry in override_entries.values() {
        if protected(&entry.path) {warnings.push(format!("Se conservará el archivo del servidor: {}",entry.path));}
        if let Some(previous)=entries.iter().position(|file|file.path.eq_ignore_ascii_case(&entry.path)) {entries.remove(previous);}
        entries.push(entry.clone());
    }
    entries.sort_by(|a,b|a.path.cmp(&b.path));
    let all:HashSet<_>=entries.iter().map(|entry|entry.path.to_lowercase()).collect();
    for path in &all {let mut parent=path.as_str();while let Some((prefix,_))=parent.rsplit_once('/') {if all.contains(prefix) {return Err(format!("El paquete usa la misma ruta como archivo y carpeta: {prefix}"));}parent=prefix;}}
    if loader=="vanilla" && entries.iter().any(|entry|entry.path.to_lowercase().starts_with("mods/")&&entry.server=="required") {blockers.push("El paquete incluye mods obligatorios pero no declara un loader.".into());}
    if entries.iter().any(|entry|entry.path.ends_with(".bat")||entry.path.ends_with(".sh")||entry.path.ends_with(".exe")) {warnings.push("El paquete incluye programas o scripts. El launcher no los ejecutará.".into());}
    let selected_bytes=entries.iter().filter(|entry|entry.selected).map(|entry|entry.size).sum();
    let inspection=PackInspection {path:path.canonicalize().map_err(|e|e.to_string())?.to_string_lossy().into(),archive_sha512,name:index.name.clone(),version_id:index.version_id.clone(),summary:index.summary.clone(),minecraft:index.dependencies["minecraft"].clone(),loader,loader_version,entries,excluded_client_files:excluded,selected_bytes,blockers,warnings};
    Ok(InspectedPack {index,inspection})
}

// Own a fresh staging directory. An unsuccessful preparation never leaves a
// partially extracted pack in a server directory or replaces existing files.
pub struct PreparedPack {pub directory:PathBuf,pub inspection:PackInspection}
impl Drop for PreparedPack {fn drop(&mut self) {let _=fs::remove_dir_all(&self.directory);}}

fn selection(pack:&PackInspection,selected:&[String])->Result<Vec<PackEntry>,String> {
    if !pack.blockers.is_empty() {return Err(pack.blockers.join("\n"));}
    let mut names=HashSet::new();
    for path in selected {
        if !names.insert(path.as_str()) {return Err("La selección repite un archivo".into());}
        let entry=pack.entries.iter().find(|entry|entry.path==*path).ok_or("El archivo seleccionado no pertenece al paquete")?;
        if protected(path) || client_path(path) || entry.server=="unsupported" {return Err(format!("Este archivo no se puede instalar en el servidor: {path}"));}
    }
    for entry in &pack.entries {
        if entry.selected && entry.server=="required" && !names.contains(entry.path.as_str()) {return Err(format!("Falta un archivo obligatorio: {}",entry.path));}
    }
    Ok(pack.entries.iter().filter(|entry|names.contains(entry.path.as_str())).cloned().collect())
}

pub fn prepare(path:&Path,expected_sha512:&str,selected:&[String],staging_root:&Path,task:&crate::download::DownloadTask)->Result<PreparedPack,String> {
    task.cancellation.check()?;
    let initial=inspect(path)?.inspection;
    let root=staging_root.canonicalize().map_err(|e|e.to_string())?;
    if !root.is_dir() || fs::symlink_metadata(staging_root).map_err(|e|e.to_string())?.file_type().is_symlink() {return Err("La carpeta temporal del modpack no es válida".into());}
    let directory=root.join(format!(".tch-pack-{}",uuid::Uuid::new_v4()));fs::create_dir(&directory).map_err(|e|e.to_string())?;
    let mut prepared=PreparedPack {directory,inspection:initial};
    let snapshot=prepared.directory.join("source.mrpack");
    // Inspect the exact private copy that will be extracted, so a changed source
    // cannot mix the reviewed index with a different ZIP during extraction.
    let mut source=fs::File::open(path).map_err(|e|e.to_string())?;let mut output=fs::File::create(&snapshot).map_err(|e|e.to_string())?;
    let mut buffer=[0u8;65536];let mut size=0u64;
    loop {task.cancellation.check()?;let n=source.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break;}size+=n as u64;if size>MAX_ARCHIVE {return Err("El modpack supera 512 MB".into());}output.write_all(&buffer[..n]).map_err(|e|e.to_string())?;}
    output.sync_all().map_err(|e|e.to_string())?;drop(output);
    let pack=inspect(&snapshot)?;
    if !pack.inspection.archive_sha512.eq_ignore_ascii_case(expected_sha512) {return Err("El modpack cambió después de la revisión. Vuelve a seleccionarlo.".into());}
    let chosen=selection(&pack.inspection,selected)?;
    let contents=prepared.directory.join("contents");fs::create_dir(&contents).map_err(|e|e.to_string())?;
    materialize(&pack,&chosen,&snapshot,&contents,task,|file,target,task| {
        let mut last="No se pudo descargar el archivo del modpack".to_string();
        for url in &file.downloads {
            task.cancellation.check()?;
            match task.download_pack(url,target,&file.hashes["sha512"],file.file_size,&file.path) {
                Ok(())=>return Ok(()),
                Err(error) if error==crate::download::CANCELLED=>return Err(error),
                Err(error)=>last=error,
            }
        }
        Err(last)
    })?;
    fs::remove_file(snapshot).map_err(|e|e.to_string())?;
    prepared.inspection=pack.inspection;prepared.inspection.path=path.to_string_lossy().into();
    let chosen_names:HashSet<_>=chosen.iter().map(|entry|entry.path.as_str()).collect();
    for entry in &mut prepared.inspection.entries {entry.selected=chosen_names.contains(entry.path.as_str());}
    prepared.inspection.selected_bytes=chosen.iter().map(|entry|entry.size).sum();
    task.cancellation.check()?;Ok(prepared)
}

fn materialize(pack:&InspectedPack,chosen:&[PackEntry],snapshot:&Path,contents:&Path,task:&crate::download::DownloadTask,
    mut download:impl FnMut(&PackFile,&Path,&crate::download::DownloadTask)->Result<(),String>)->Result<(),String> {
    let mut archive=zip::ZipArchive::new(fs::File::open(snapshot).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    for entry in chosen {
        task.cancellation.check()?;
        let target=contents.join(relative_path(&entry.path)?);fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;
        if entry.source=="download" {
            let file=pack.index.files.iter().find(|file|file.path==entry.path).ok_or("Falta el archivo en el índice del paquete")?;
            download(file,&target,task)?;
            let mut file_on_disk=fs::File::open(&target).map_err(|e|e.to_string())?;
            let mut sha1=sha1::Sha1::new();let mut sha512=Sha512::new();let mut size=0u64;let mut buffer=[0u8;65536];
            loop {task.cancellation.check()?;let n=file_on_disk.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break;}size+=n as u64;if size>file.file_size {return Err(format!("Tamaño del archivo no válido: {}",entry.path));}sha1.update(&buffer[..n]);sha512.update(&buffer[..n]);}
            if size!=file.file_size || !format!("{:x}",sha1.finalize()).eq_ignore_ascii_case(&file.hashes["sha1"]) || !format!("{:x}",sha512.finalize()).eq_ignore_ascii_case(&file.hashes["sha512"]) {return Err(format!("El archivo del paquete no coincide con sus hashes o tamaño: {}",entry.path));}
        } else {
            if !matches!(entry.source.as_str(),"overrides"|"server-overrides") {return Err("Capa del paquete no válida".into());}
            task.report(&format!("Preparando {}",entry.path),"extracting");
            let mut source=archive.by_name(&format!("{}/{}",entry.source,entry.path)).map_err(|e|e.to_string())?;
            let mut output=fs::File::create(&target).map_err(|e|e.to_string())?;let mut size=0u64;let mut buffer=[0u8;65536];
            loop {task.cancellation.check()?;let n=source.read(&mut buffer).map_err(|e|e.to_string())?;if n==0 {break;}size+=n as u64;if size>entry.size {return Err("El archivo extraído supera el tamaño declarado".into());}output.write_all(&buffer[..n]).map_err(|e|e.to_string())?;}
            if size!=entry.size {return Err("El archivo extraído está incompleto".into());}output.sync_all().map_err(|e|e.to_string())?;
        }
    }
    Ok(())
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

    #[test]
    fn selection_requires_dependencies_and_rejects_unreviewed_or_protected_files() {
        let mut data=index();data["files"]=serde_json::json!([file("mods/required.jar","required"),file("mods/optional.jar","optional")]);
        let pack=create(data,&[("server-overrides/eula.txt","eula=true")]);let review=inspect(&pack.0).unwrap().inspection;
        assert!(selection(&review,&[]).is_err());
        assert!(selection(&review,&["mods/required.jar".into(),"eula.txt".into()]).is_err());
        assert!(selection(&review,&["mods/required.jar".into(),"mods/unknown.jar".into()]).is_err());
        assert_eq!(selection(&review,&["mods/required.jar".into(),"mods/optional.jar".into()]).unwrap().len(),2);
    }

    #[test]
    fn prepares_server_layers_and_cleans_on_changed_archive_or_cancel() {
        let pack=create(index(),&[("overrides/config/value.txt","common"),("server-overrides/config/value.txt","server"),("server-overrides/eula.txt","eula=true"),("client-overrides/config/client.txt","client")]);
        let review=inspect(&pack.0).unwrap().inspection;let task=crate::download::DownloadTask::new("pack","install",|_|{}).unwrap();
        let root=std::env::temp_dir().join(format!("tch-pack-stage-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();
        let prepared=prepare(&pack.0,&review.archive_sha512,&["config/value.txt".into()],&root,&task).unwrap();
        assert_eq!(fs::read_to_string(prepared.directory.join("contents/config/value.txt")).unwrap(),"server");
        assert!(!prepared.directory.join("contents/eula.txt").exists());assert!(!prepared.directory.join("contents/config/client.txt").exists());
        let directory=prepared.directory.clone();drop(prepared);assert!(!directory.exists());
        assert!(prepare(&pack.0,&"0".repeat(128),&["config/value.txt".into()],&root,&task).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(),0);
        task.cancellation.cancel();assert_eq!(prepare(&pack.0,&review.archive_sha512,&["config/value.txt".into()],&root,&task).err().unwrap(),crate::download::CANCELLED);
        assert_eq!(fs::read_dir(&root).unwrap().count(),0);fs::remove_dir(root).unwrap();
    }

    #[test]
    fn verifies_both_hashes_and_declared_size_of_downloaded_content() {
        let body=b"real file bytes";let mut descriptor=file("mods/test.jar","required");
        descriptor["fileSize"]=(body.len() as u64).into();descriptor["hashes"]["sha1"]=format!("{:x}",sha1::Sha1::digest(body)).into();descriptor["hashes"]["sha512"]=format!("{:x}",Sha512::digest(body)).into();
        let mut data=index();data["files"]=serde_json::json!([descriptor]);let pack=create(data,&[]);let mut inspected=inspect(&pack.0).unwrap();
        let root=std::env::temp_dir().join(format!("tch-pack-hash-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();let task=crate::download::DownloadTask::new("pack","install",|_|{}).unwrap();
        let chosen=selection(&inspected.inspection,&["mods/test.jar".into()]).unwrap();
        materialize(&inspected,&chosen,&pack.0,&root,&task,|_,target,_|fs::write(target,body).map_err(|e|e.to_string())).unwrap();
        inspected.index.files[0].hashes.insert("sha1".into(),"0".repeat(40));
        assert!(materialize(&inspected,&chosen,&pack.0,&root,&task,|_,target,_|fs::write(target,body).map_err(|e|e.to_string())).is_err());
        inspected.index.files[0].file_size=1;
        assert!(materialize(&inspected,&chosen,&pack.0,&root,&task,|_,target,_|fs::write(target,body).map_err(|e|e.to_string())).is_err());fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore="Downloads an official Modrinth file into a staged mrpack; does not execute Java"]
    fn prepares_live_pack_download_with_hashes_and_server_overrides() {
        let task=crate::download::DownloadTask::new("pack","install",|_|{}).unwrap();
        let versions=task.json("https://api.modrinth.com/v2/project/gvQqBUqZ/version?game_versions=%5B%221.21.4%22%5D&loaders=%5B%22fabric%22%5D","Consultando Lithium").unwrap();
        let version=versions.as_array().unwrap().iter().find(|v|v["version_type"]=="release").unwrap();
        let artifact=version["files"].as_array().unwrap().iter().find(|file|file["primary"]==true).unwrap();
        let name=format!("mods/{}",artifact["filename"].as_str().unwrap());let mut data=index();
        data["files"]=serde_json::json!([{"path":name,"fileSize":artifact["size"],"hashes":artifact["hashes"],"downloads":[artifact["url"]],"env":{"client":"required","server":"required"}}]);
        let pack=create(data,&[("overrides/config/example.txt","common"),("server-overrides/config/example.txt","server")]);let review=inspect(&pack.0).unwrap().inspection;
        let root=std::env::temp_dir().join(format!("tch-pack-live-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();
        let prepared=prepare(&pack.0,&review.archive_sha512,&[name.clone(),"config/example.txt".into()],&root,&task).unwrap();
        assert!(prepared.directory.join("contents").join(name).is_file());assert_eq!(fs::read_to_string(prepared.directory.join("contents/config/example.txt")).unwrap(),"server");
        assert!(!prepared.directory.join("contents/eula.txt").exists());drop(prepared);assert_eq!(fs::read_dir(&root).unwrap().count(),0);fs::remove_dir(root).unwrap();
    }
}
