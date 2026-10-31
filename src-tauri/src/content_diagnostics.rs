//! Read-only checks. A descriptor or a startup-list entry never proves gameplay behavior.
use crate::{content_storage::{self,InstalledContent},model::Server};
use serde::Serialize;
use std::{collections::{HashMap,HashSet},fs,io::Read,path::Path};

#[derive(Clone,Default,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Descriptor {
    pub id:Option<String>,pub name:Option<String>,pub version:Option<String>,
    pub environment:Option<String>,pub minecraft:Option<String>,pub note:Option<String>,
    pub ids:Vec<String>,
    pub api_version:Option<String>,
    pub dependencies:Vec<Dependency>,
    #[serde(skip)]pub logo:Option<String>,
}

/// Preserve each declaration and its OR alternatives; no compatibility inference.
#[derive(Clone,Serialize)]
pub struct Dependency {pub id:String,pub kind:String,pub alternatives:Vec<String>}

fn fabric_id(id:&str)->bool {
    id.len()>=2&&id.len()<=64&&id.as_bytes()[0].is_ascii_lowercase()&&id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'-'|b'_'))
}

fn fabric_dependencies(value:&serde_json::Value)->Result<Vec<Dependency>,String>{
    let mut result=Vec::new();
    for kind in ["depends","recommends","suggests","conflicts","breaks"]{
        let Some(declarations)=value.get(kind) else{continue;};
        let declarations=declarations.as_object().ok_or_else(||format!("La declaración Fabric {kind} debe ser un objeto"))?;
        if declarations.len()>512||result.len()+declarations.len()>512{return Err("Demasiadas dependencias declaradas".into());}
        for(id,range)in declarations{
            if !fabric_id(id){return Err(format!("Identificador de dependencia Fabric no válido en {kind}"));}
            let ranges=match range{
                serde_json::Value::String(value)=>vec![value.as_str()],
                serde_json::Value::Array(values) if !values.is_empty()&&values.len()<=64=>values.iter().map(|value|value.as_str().ok_or("Rango de dependencia Fabric no válido")).collect::<Result<Vec<_>,_>>()?,
                _=>return Err(format!("Rango de dependencia Fabric no válido para {id}")),
            };
            if ranges.iter().any(|range|range.is_empty()||range.len()>256||range.chars().any(char::is_control)){return Err(format!("Rango de dependencia Fabric no válido para {id}"));}
            result.push(Dependency{id:id.clone(),kind:kind.into(),alternatives:ranges.into_iter().map(str::to_string).collect()});
        }
    }
    Ok(result)
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
    pub files:Vec<InstalledContent>,pub disabled:Vec<InstalledContent>,pub checks:Vec<Check>,
    pub log_modified_at:Option<u64>,pub log_notes:Vec<String>,
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
    if platform=="neoforge" {return inspect_neoforge(&data);}
    if platform=="paper" {return inspect_paper(&data);}
    let value:serde_json::Value=serde_json::from_slice(&data).map_err(|_|"fabric.mod.json no contiene JSON válido")?;
    inspect_fabric(&value)
}

pub(crate) fn inspect_fabric(value:&serde_json::Value)->Result<Descriptor,String> {
    if !value.is_object() {return Err("fabric.mod.json debe contener un objeto de metadatos".into());}
    match value.get("schemaVersion") {
        None=>return Ok(Descriptor {note:Some("Descriptor Fabric antiguo: revisa su carga en la consola".into()),..Default::default()}),
        Some(schema) if schema.as_u64()==Some(0)=>return Ok(Descriptor {note:Some("Descriptor Fabric antiguo: revisa su carga en la consola".into()),..Default::default()}),
        Some(schema) if schema.as_u64()==Some(1)=>{},
        _=>return Err("Formato Fabric desconocido; revisión manual necesaria".into()),
    }
    let id=text(&value,"id").filter(|id|fabric_id(id)).ok_or("Identificador Fabric no válido")?;
    let version=text(&value,"version").ok_or("Versión Fabric no válida")?;
    let environment=match value.get("environment") {
        None=>"*".to_string(),
        Some(v)=>v.as_str().ok_or("Entorno Fabric no válido")?.to_ascii_lowercase(),
    };
    let environment=match environment.as_str(){
        ""|"*"=>"*","client"=>"client","server"=>"server",
        _=>return Err("Entorno Fabric no válido".into()),
    };
    let minecraft=value.get("depends").and_then(|v|v.get("minecraft")).map(|v|v.as_str().map(str::to_string).unwrap_or_else(||v.to_string())).filter(|v|v.len()<=256);
    Ok(Descriptor {ids:vec![id.clone()],id:Some(id),name:text(&value,"name"),version:Some(version),environment:Some(environment.into()),minecraft,note:None,logo:None,api_version:None,dependencies:fabric_dependencies(&value)?})
}

fn inspect_neoforge(data:&[u8])->Result<Descriptor,String>{
    let source=std::str::from_utf8(data).map_err(|_|"El descriptor NeoForge no contiene UTF-8 válido")?;
    let value:toml::Value=toml::from_str(source).map_err(|_|"El descriptor NeoForge no contiene TOML válido")?;
    let mods=value.get("mods").and_then(toml::Value::as_array).filter(|mods|!mods.is_empty()&&mods.len()<=128).ok_or("No hay una lista de mods NeoForge válida")?;
    let field=|value:&toml::Value,key:&str|value.get(key).and_then(toml::Value::as_str).filter(|s|!s.is_empty()&&s.len()<=256&&!s.chars().any(char::is_control)).map(str::to_string);
    let mut ids=Vec::new();
    for entry in mods{
        let id=field(entry,"modId").filter(|id|id.len()>=2&&id.len()<=64&&id.as_bytes()[0].is_ascii_lowercase()&&id.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||c==b'_')).ok_or("Identificador NeoForge no válido")?;
        if ids.contains(&id){return Err("Identificador repetido dentro del JAR NeoForge".into());}ids.push(id);
    }
    let first=&mods[0];let id=ids[0].clone();
    let version=field(first,"version").filter(|v|!v.contains("${"));
    let minecraft=value.get("dependencies").and_then(|value|value.get(&id)).and_then(toml::Value::as_array).and_then(|entries|entries.iter().find(|entry|entry.get("modId").and_then(toml::Value::as_str)==Some("minecraft"))).and_then(|entry|field(entry,"versionRange"));
    Ok(Descriptor{id:Some(id),ids,name:field(first,"displayName"),version,environment:None,minecraft,note:None,logo:field(first,"logoFile").or_else(||field(&value,"logoFile")),api_version:None,dependencies:vec![]})
}

fn inspect_paper(data:&[u8])->Result<Descriptor,String>{
    use yaml_rust2::parser::{Event,Parser};
    let source=std::str::from_utf8(data).map_err(|_|"El descriptor del plugin no contiene UTF-8 válido")?;
    let mut parser=Parser::new_from_str(source);
    let(mut depth,mut documents,mut events)=(0usize,0usize,0usize);
    let mut pending:Option<String>=None;let mut fields=HashMap::new();let mut anchors=HashMap::new();
    loop{
        let(event,_)=parser.next_token().map_err(|_|"El descriptor del plugin no contiene YAML válido")?;
        events+=1;if events>20_000{return Err("Descriptor YAML demasiado complejo".into());}
        if let Event::Scalar(value,_,anchor,tag)=&event{if *anchor>0&&tag.is_none()&&value.len()<=256&&!value.chars().any(char::is_control){if anchors.len()>=256{return Err("Demasiados anclajes YAML".into());}anchors.insert(*anchor,value.clone());}}
        match event{
            Event::DocumentStart=>{documents+=1;if documents>1{return Err("El descriptor debe contener un solo documento YAML".into());}},
            Event::MappingStart(_,_)=>{if depth==1{let key=pending.take().ok_or("Las claves del descriptor deben ser texto")?;if matches!(key.as_str(),"name"|"version"|"main"|"api-version"){return Err("La identidad del plugin debe contener texto simple".into());}}depth+=1;},
            Event::SequenceStart(_,_)=>{if depth==0{return Err("El descriptor del plugin debe ser un objeto".into());}if depth==1{let key=pending.take().ok_or("Las claves del descriptor deben ser texto")?;if matches!(key.as_str(),"name"|"version"|"main"|"api-version"){return Err("La identidad del plugin debe contener texto simple".into());}}depth+=1;},
            Event::MappingEnd|Event::SequenceEnd=>{depth=depth.saturating_sub(1);},
            Event::Scalar(value,_,_,tag) if depth==1=>{
                if let Some(key)=pending.take(){
                    if matches!(key.as_str(),"name"|"version"|"main"|"api-version"){
                        if tag.is_some()||value.is_empty()||value.len()>256||value.chars().any(char::is_control){return Err("Metadatos de plugin no válidos".into());}
                        if fields.insert(key,value).is_some(){return Err("Campo de identidad repetido en el descriptor del plugin".into());}
                    }
                }else{pending=Some(value);}
            },
            Event::Scalar(_,_,_,_) if depth==0=>return Err("El descriptor del plugin debe ser un objeto".into()),
            Event::Alias(anchor) if depth==1=>{
                let key=pending.take().ok_or("Las claves del descriptor deben ser texto")?;
                if matches!(key.as_str(),"name"|"version"|"main"|"api-version"){
                    let value=anchors.get(&anchor).filter(|value|!value.is_empty()).ok_or("No se pudo comprobar un alias YAML de identidad")?.clone();
                    if fields.insert(key,value).is_some(){return Err("Campo de identidad repetido en el descriptor del plugin".into());}
                }
            },
            Event::StreamEnd=>break,
            _=>{},
        }
        if depth>64{return Err("Descriptor YAML demasiado profundo".into());}
    }
    let name=fields.remove("name").ok_or("El plugin no declara un nombre válido")?;
    if fields.remove("main").is_none(){return Err("El plugin no declara su clase principal".into());}
    let version=fields.remove("version").ok_or("El plugin no declara una versión")?;
    Ok(Descriptor{id:Some(name.clone()),ids:vec![name.clone()],name:Some(name),version:(!version.contains("${")).then_some(version),api_version:fields.remove("api-version"),..Default::default()})
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

/// Read a declared mod icon inside the JAR, never a path or URL outside it.
pub fn installed_icon(server:&Server,filename:&str)->Result<Option<String>,String> {
    if !matches!(server.platform.as_str(),"fabric"|"neoforge") {return Ok(None);}
    if filename.len()>180||filename.starts_with('.')||!filename.ends_with(".jar")||filename.chars().any(|c|c.is_control()||matches!(c,'/'|'\\'|':'|'<'|'>'|'"'|'|'|'?'|'*')) {return Err("Nombre de archivo no válido".into());}
    let base=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let folder=base.join("mods");let path=folder.join(filename);
    if fs::symlink_metadata(&folder).map_err(|e|e.to_string())?.file_type().is_symlink()||!folder.canonicalize().map_err(|e|e.to_string())?.starts_with(&base) {return Err("La carpeta de mods no es válida".into());}
    let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
    if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>512_000_000||!path.canonicalize().map_err(|e|e.to_string())?.starts_with(&base) {return Err("Archivo de mod no válido".into());}
    let mut archive=zip::ZipArchive::new(fs::File::open(&path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    if archive.len()>100_000 {return Ok(None);}
    let name=if server.platform=="fabric"{"fabric.mod.json"}else if archive.file_names().any(|name|name=="META-INF/neoforge.mods.toml"){"META-INF/neoforge.mods.toml"}else{"META-INF/mods.toml"};
    let bytes={
        let Ok(file)=archive.by_name(name) else{return Ok(None);};
        if file.size()>1_000_000{return Ok(None);}
        let mut bytes=Vec::new();file.take(1_000_001).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
        if bytes.len()>1_000_000{return Ok(None);}
        bytes
    };
    let name=if server.platform=="neoforge" {inspect_neoforge(&bytes)?.logo}else{
        let descriptor:serde_json::Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
        let icon=descriptor.get("icon");
        icon.and_then(|v|v.as_str()).or_else(||icon.and_then(|v|v.as_object()).and_then(|sizes|sizes.iter().filter_map(|(size,path)|Some((size.parse::<u32>().ok()?,path.as_str()?))).min_by_key(|(size,_)|size.abs_diff(64)).map(|(_,path)|path))).map(str::to_string)
    };
    let Some(name)=name.filter(|name|!name.is_empty()&&name.len()<=256&&!name.starts_with('/')&&!name.contains(['\\',':'])&&!name.split('/').any(|part|matches!(part,""|"."|".."))) else{return Ok(None);};
    let Ok(file)=archive.by_name(&name) else{return Ok(None);};
    if file.size()>262_144{return Ok(None);}
    let mut bytes=Vec::new();file.take(262_145).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len()>262_144{return Ok(None);}
    normalize_icon(&bytes).map(Some)
}

fn normalize_icon(bytes:&[u8])->Result<String,String> {
    use base64::Engine;
    let mut reader=image::ImageReader::new(std::io::Cursor::new(bytes));
    reader.set_format(image::ImageFormat::Png);
    let mut limits=image::Limits::default();limits.max_image_width=Some(512);limits.max_image_height=Some(512);limits.max_alloc=Some(4_000_000);reader.limits(limits);
    let image=reader.decode().map_err(|_|"Icono PNG no válido")?;
    if image.width()==0||image.height()==0 {return Err("Icono vacío".into());}
    let thumbnail=image.thumbnail(64,64);
    let mut output=std::io::Cursor::new(Vec::new());thumbnail.write_to(&mut output,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    Ok(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(output.into_inner())))
}

pub fn audit(server:&Server)->Result<Audit,String> {
    let files=content_storage::list(server)?;
    let base=Path::new(&server.directory);
    let version_parts:Vec<_>=server.version.split('.').collect();
    // Experimental Minecraft IDs need loader normalization; overrides can replace requirements.
    let review_minecraft=server.platform=="fabric"&&(2..=3).contains(&version_parts.len())
        &&version_parts.iter().all(|part|!part.is_empty()&&part.bytes().all(|c|c.is_ascii_digit())&&part.parse::<i32>().is_ok())
        &&matches!(fs::symlink_metadata(base.join("config/fabric_loader_dependencies.json")),Err(error) if error.kind()==std::io::ErrorKind::NotFound);
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
        if review_minecraft&&descriptor.environment.as_deref()!=Some("client") {
            if let Some(requirement)=descriptor.dependencies.iter().find(|dependency|dependency.id=="minecraft"&&dependency.kind=="depends") {
                if crate::fabric_version::matches(&server.version,&requirement.alternatives)==Some(false) {
                    issues.push(format!("Minecraft configurado {} no cumple el requisito del mod: {}. Revisa la versión del servidor o del archivo.",server.version,requirement.alternatives.join(" o ")));
                }
            }
        }
        if let Some(note)=&descriptor.note {issues.push(note.clone());}
        for id in &descriptor.ids {owners.entry(id.clone()).or_default().push(checks.len());}
        let modified=fs::metadata(&path).ok().and_then(|v|v.modified().ok()).and_then(|v|v.duration_since(std::time::UNIX_EPOCH).ok()).map(|v|v.as_millis() as u64);
        let listed_in_log=if !issues.is_empty()||modified.zip(log_modified_at).is_none_or(|(jar,log)|jar>log){None}else{descriptor.id.as_ref().and_then(|id|loaded.as_ref().map(|ids|ids.contains(id)))};
        checks.push(Check {filename:file.filename.clone(),descriptor,issues,listed_in_log});
    }
    for (id,indices) in owners {if indices.len()>1 {for i in indices {checks[i].issues.push(format!("Identificador duplicado: {id}. Hay más de un JAR de este {}",if server.platform=="paper"{"plugin"}else{"mod"}));checks[i].listed_in_log=None;}}}
    let mut log_notes=vec![log_note];
    if server.platform=="fabric" {
        let presence=crate::fabric_presence::review(base,&files);
        for check in &mut checks {
            if let Some(issues)=presence.issues.get(&check.filename) {
                check.issues.extend(issues.iter().cloned());
            }
        }
        log_notes.push(presence.note);
    }
    Ok(Audit {files,disabled:content_storage::list_disabled(server)?,checks,log_modified_at,log_notes})
}
