//! Read-only checks. A descriptor or a startup-list entry never proves gameplay behavior.
use crate::{content_storage::{self,InstalledContent},model::Server};
use serde::Serialize;
use std::{collections::{HashMap,HashSet},fs,io::Read,path::Path};

#[derive(Clone,Default,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Descriptor {
    pub id:Option<String>,pub name:Option<String>,pub version:Option<String>,
    pub environment:Option<String>,pub minecraft:Option<String>,pub note:Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Check {
    pub filename:String,pub descriptor:Descriptor,pub issues:Vec<String>,
    /// None means no trustworthy comparison, not an unloaded mod.
    pub listed_in_log:Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Audit {
    pub files:Vec<InstalledContent>,pub checks:Vec<Check>,
    pub log_modified_at:Option<u64>,pub log_note:String,
}

fn text(value:&serde_json::Value,key:&str)->Option<String> {
    value.get(key).and_then(|v|v.as_str()).filter(|s|!s.is_empty()&&s.len()<=256&&!s.chars().any(char::is_control)).map(str::to_string)
}

pub(crate) fn inspect(path:&Path,platform:&str)->Result<Descriptor,String> {
    let metadata=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    if !metadata.is_file()||metadata.file_type().is_symlink()||metadata.len()>512_000_000 {return Err("Archivo JAR no válido".into());}
    let mut archive=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|_|"El archivo no es un JAR legible".to_string())?;
    if archive.is_empty()||archive.len()>100_000 {return Err("Contenido JAR no válido".into());}
    let names:&[&str]=match platform {"fabric"=>&["fabric.mod.json"],"neoforge"=>&["META-INF/neoforge.mods.toml","META-INF/mods.toml"],"paper"=>&["paper-plugin.yml","plugin.yml"],_=>return Err("Cargador no admitido".into())};
    let name=names.iter().find(|name|archive.file_names().any(|entry|entry==**name)).ok_or_else(||format!("No incluye un descriptor de {platform}; puede ser de otro cargador"))?;
    let file=archive.by_name(name).map_err(|_|"Descriptor no legible")?;
    if file.size()>1_000_000 {return Err("Descriptor demasiado grande".into());}
    let mut data=Vec::new();file.take(1_000_001).read_to_end(&mut data).map_err(|_|"No se pudo leer el descriptor")?;
    if data.len()>1_000_000 {return Err("Descriptor demasiado grande".into());}
    // Other descriptor formats are only identified; their dependencies aren't inferred.
    if platform!="fabric" {return Ok(Descriptor::default());}
    let value:serde_json::Value=serde_json::from_slice(&data).map_err(|_|"fabric.mod.json no contiene JSON válido")?;
    if !value.is_object() {return Err("fabric.mod.json debe contener un objeto de metadatos".into());}
    match value.get("schemaVersion") {
        None=>return Ok(Descriptor {note:Some("Descriptor Fabric antiguo: revisa su carga en la consola".into()),..Default::default()}),
        Some(schema) if schema.as_u64()==Some(0)=>return Ok(Descriptor {note:Some("Descriptor Fabric antiguo: revisa su carga en la consola".into()),..Default::default()}),
        Some(schema) if schema.as_u64()==Some(1)=>{},
        _=>return Err("Formato Fabric desconocido; revisión manual necesaria".into()),
    }
    let id=text(&value,"id").filter(|id|id.len()>=2&&id.len()<=64&&id.as_bytes()[0].is_ascii_lowercase()&&id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'-'|b'_'))).ok_or("Identificador Fabric no válido")?;
    let version=text(&value,"version").ok_or("Versión Fabric no válida")?;
    let environment=match value.get("environment") {
        None=>"*",
        Some(v) if v.is_array()=>{
            let values=v.as_array().unwrap();
            if values.is_empty()||values.len()>3||values.iter().any(|v|!matches!(v.as_str(),Some("*"|"client"|"server"))) {return Err("Entorno Fabric no válido".into());}
            let client=values.iter().any(|v|v.as_str()==Some("client"));let server=values.iter().any(|v|v.as_str()==Some("server"));
            if values.iter().any(|v|v.as_str()==Some("*"))||(client&&server) {"*"}else if server {"server"}else{"client"}
        },
        Some(v)=>v.as_str().filter(|v|matches!(*v,"*"|"client"|"server")).ok_or("Entorno Fabric no válido")?,
    };
    let minecraft=value.get("depends").and_then(|v|v.get("minecraft")).map(|v|v.as_str().map(str::to_string).unwrap_or_else(||v.to_string())).filter(|v|v.len()<=256);
    Ok(Descriptor {id:Some(id),name:text(&value,"name"),version:Some(version),environment:Some(environment.into()),minecraft,note:None})
}

fn listed_mods(log:&str)->Option<HashSet<String>> {
    let mut ids=None;let mut collecting=false;let mut expected=0;
    for line in log.lines() {
        // Only Fabric's INFO header opens a list; player chat cannot open one.
        if let Some((prefix,message))=line.split_once("]: ") {
            if prefix.ends_with("/INFO") {
                if let Some(count)=message.strip_prefix("Loading ").and_then(|v|v.strip_suffix(" mods:")) {
                    if let Ok(count)=count.parse::<usize>() {if count<=100_000 {ids=Some(HashSet::new());expected=count;collecting=true;continue;}}
                }
            }
        }
        if line.starts_with('[') {collecting=false;continue;}
        if let Some(ids)=ids.as_mut().filter(|_|collecting) {
            let row=line.trim_start();
            if row.starts_with(['-','|','\\']) {
                let value=row.trim_start_matches(|c:char|c.is_ascii_whitespace()||matches!(c,'-'|'|'|'\\'));
                if let Some(id)=value.split_whitespace().next().filter(|id|id.len()<=64&&id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'-'|b'_'))) {ids.insert(id.into());}
            }
        }
    }
    ids.filter(|v|!v.is_empty()&&v.len()==expected)
}

fn read_log(base:&Path)->Result<(String,u64),String> {
    let path=base.join("logs/latest.log");
    let canonical=path.canonicalize().map_err(|_|"No hay un latest.log legible todavía")?;
    if !canonical.starts_with(base.canonicalize().map_err(|e|e.to_string())?) {return Err("El log apunta fuera del servidor".into());}
    if fs::symlink_metadata(base.join("logs")).map_err(|e|e.to_string())?.file_type().is_symlink() {return Err("La carpeta del log es un enlace".into());}
    let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
    if !meta.is_file()||meta.file_type().is_symlink() {return Err("El log no es un archivo regular".into());}
    // Startup lists appear at the beginning. Truncation never becomes negative evidence.
    if meta.len()>2_000_000 {return Err("El log supera 2 MB; consulta la consola para comprobar la carga".into());}
    let mut data=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(2_000_001).read_to_end(&mut data).map_err(|e|e.to_string())?;
    if data.len()>2_000_000 {return Err("El log cambió mientras se leía; vuelve a revisar".into());}
    let modified=meta.modified().ok().and_then(|v|v.duration_since(std::time::UNIX_EPOCH).ok()).map(|v|v.as_millis() as u64).unwrap_or(0);
    Ok((String::from_utf8_lossy(&data).into_owned(),modified))
}

pub fn audit(server:&Server)->Result<Audit,String> {
    let files=content_storage::list(server)?;
    let base=Path::new(&server.directory);
    let (loaded,log_modified_at,log_note)=if server.platform=="fabric" {
        match read_log(base) {
            Ok((log,at))=>match listed_mods(&log) {
                Some(ids)=>(Some(ids),Some(at),"Comparación con la lista Fabric del último log; no confirma el funcionamiento dentro del mundo.".into()),
                None=>(None,Some(at),"El último log no incluye una lista Fabric reconocible. La carga queda sin confirmar.".into()),
            },
            Err(error)=>(None,None,error),
        }
    }else{(None,None,"Descriptor reconocido; consulta la consola para comprobar la carga de este motor.".into())};
    let mut checks=vec![];let mut owners:HashMap<String,Vec<usize>>=HashMap::new();
    for file in &files {
        let path=base.join(if server.platform=="paper"{"plugins"}else{"mods"}).join(&file.filename);
        let mut issues=vec![];
        let descriptor=match inspect(&path,&server.platform){Ok(value)=>value,Err(error)=>{issues.push(error);Descriptor::default()}};
        if descriptor.environment.as_deref()==Some("client") {issues.push("Solo cliente: Fabric no lo carga en un servidor dedicado".into());}
        if let Some(note)=&descriptor.note {issues.push(note.clone());}
        if let Some(id)=&descriptor.id {owners.entry(id.clone()).or_default().push(checks.len());}
        let modified=fs::metadata(&path).ok().and_then(|v|v.modified().ok()).and_then(|v|v.duration_since(std::time::UNIX_EPOCH).ok()).map(|v|v.as_millis() as u64);
        let listed_in_log=if !issues.is_empty()||modified.zip(log_modified_at).is_none_or(|(jar,log)|jar>log){None}else{descriptor.id.as_ref().and_then(|id|loaded.as_ref().map(|ids|ids.contains(id)))};
        checks.push(Check {filename:file.filename.clone(),descriptor,issues,listed_in_log});
    }
    for (id,indices) in owners {if indices.len()>1 {for i in indices {checks[i].issues.push(format!("Identificador duplicado: {id}. Hay más de un JAR de este mod"));checks[i].listed_in_log=None;}}}
    Ok(Audit {files,checks,log_modified_at,log_note})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn jar(path:&Path,name:&str,data:&[u8]) {
        let mut zip=zip::ZipWriter::new(fs::File::create(path).unwrap());
        zip.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();zip.write_all(data).unwrap();zip.finish().unwrap();
    }
    #[test]
    fn descriptor_detects_client_wrong_loader_and_invalid_json_without_executing() {
        let root=std::env::temp_dir().join(format!("tch-descriptor-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let path=root.join("mod.jar");
        jar(&path,"fabric.mod.json",br#"{"schemaVersion":1,"id":"backpack","version":"1.0","environment":"client","depends":{"minecraft":">=1.21"}}"#);
        let found=inspect(&path,"fabric").unwrap();assert_eq!(found.environment.as_deref(),Some("client"));assert_eq!(found.minecraft.as_deref(),Some(">=1.21"));assert!(inspect(&path,"neoforge").is_err());
        jar(&path,"fabric.mod.json",b"invalid");assert!(inspect(&path,"fabric").is_err());
        jar(&path,"fabric.mod.json",b"123");assert!(inspect(&path,"fabric").is_err());
        jar(&path,"fabric.mod.json",&vec![b' ';1_000_001]);assert!(inspect(&path,"fabric").is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn loader_list_reads_nested_ids_and_ignores_chat_and_unrelated_headers() {
        assert!(listed_mods("[12:00:00] [Server thread/INFO]: <player> Loading 5 mods:\n - fake 1").is_none());
        let log="[12:00:00] [main/INFO]: Loading 3 mods:\n - backpack 1.0\n   |-- library 2.0\n   \\-- helper 1.0\n[12:00:01] [main/INFO]: Done\n - unrelated 1\n";
        let ids=listed_mods(log).unwrap();assert!(ids.contains("backpack")&&ids.contains("library")&&ids.contains("helper"));
        assert!(!ids.contains("unrelated"));
        assert!(listed_mods("[12:00:00] [main/INFO]: Loading 3 mods:\n - backpack 1.0\n").is_none());
    }
    #[test]
    fn audit_preserves_files_flags_duplicates_and_only_compares_observed_ids() {
        let root=std::env::temp_dir().join(format!("tch-audit-{}",uuid::Uuid::new_v4()));fs::create_dir_all(root.join("mods")).unwrap();fs::create_dir_all(root.join("logs")).unwrap();
        let server:Server=serde_json::from_value(serde_json::json!({"id":"audit","name":"Audit","version":"1.21.11","platform":"fabric","directory":root.to_string_lossy(),"jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        for name in ["first.jar","duplicate.jar"] {jar(&root.join("mods").join(name),"fabric.mod.json",br#"{"schemaVersion":1,"id":"backpack","version":"1.0"}"#);}
        jar(&root.join("mods/client.jar"),"fabric.mod.json",br#"{"schemaVersion":1,"id":"clientmod","version":"1.0","environment":"client"}"#);
        jar(&root.join("mods/structures.jar"),"fabric.mod.json",br#"{"schemaVersion":1,"id":"structures","version":"1.0"}"#);
        fs::write(root.join("logs/latest.log"),"[12:00:00] [main/INFO]: Loading 2 mods:\n - backpack 1.0\n - structures 1.0\n").unwrap();
        let result=audit(&server).unwrap();assert_eq!(result.files.len(),4);
        for name in ["first.jar","duplicate.jar"] {let row=result.checks.iter().find(|c|c.filename==name).unwrap();assert!(row.issues.iter().any(|s|s.contains("duplicado")));assert_eq!(row.listed_in_log,None);}
        assert!(result.checks.iter().find(|c|c.filename=="client.jar").unwrap().issues[0].contains("Solo cliente"));
        assert_eq!(result.checks.iter().find(|c|c.filename=="structures.jar").unwrap().listed_in_log,Some(true));
        assert!(root.join("mods/client.jar").exists());
        fs::remove_file(root.join("logs/latest.log")).unwrap();assert!(audit(&server).unwrap().checks.iter().all(|c|c.listed_in_log.is_none()));
        fs::remove_dir_all(root).unwrap();
    }
}
