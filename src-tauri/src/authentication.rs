//! Read-only evidence from Paper/Spigot files. Never expose proxy secrets.
use serde::Serialize;
use std::{collections::{HashMap,HashSet},fs,io::Read,path::Path};
use yaml_rust2::parser::{Event,Parser};

#[derive(Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Check {
    pub file:String,
    pub key:String,
    pub enabled:Option<bool>,
    pub problem:Option<String>,
}

struct Frame {path:Vec<String>,mapping:bool,pending:Option<String>,keys:HashSet<String>}

fn child_path(stack:&mut [Frame])->Result<Vec<String>,String>{
    match stack.last_mut(){
        None=>Ok(vec![]),
        Some(frame)if !frame.mapping=>{let mut path=frame.path.clone();path.push("[]".into());Ok(path)},
        Some(frame)=>{let key=frame.pending.take().ok_or("Las claves YAML deben ser texto simple")?;
            let mut path=frame.path.clone();path.push(key);Ok(path)},
    }
}
fn target_path(path:&[String],targets:&[&str])->bool {targets.iter().any(|target|path.iter().map(String::as_str).eq(target.split('.')))}

fn booleans(source:&str,targets:&[&str])->Result<HashMap<String,bool>,String>{
    let mut parser=Parser::new_from_str(source);
    let mut stack:Vec<Frame>=vec![];
    let mut values=HashMap::new();let mut documents=0;
    for _ in 0..20_000 {
        let(event,_)=parser.next_token().map_err(|_|"No se pudo interpretar el archivo YAML")?;
        match event {
            Event::DocumentStart=>{documents+=1;if documents>1{return Err("El archivo contiene varios documentos YAML".into());}},
            Event::MappingStart(_,_)|Event::SequenceStart(_,_)=>{
                let mapping=matches!(event,Event::MappingStart(_, _));
                if stack.is_empty()&&!mapping{return Err("La configuración debe ser un objeto YAML".into());}
                let path=child_path(&mut stack)?;
                if target_path(&path,targets){return Err("El ajuste debe ser true o false".into());}
                stack.push(Frame{path,mapping,pending:None,keys:HashSet::new()});
                if stack.len()>64{return Err("El archivo YAML es demasiado profundo".into());}
            },
            Event::MappingEnd|Event::SequenceEnd=>{stack.pop();},
            Event::Scalar(value,_,_,tag)=>{
                let Some(frame)=stack.last_mut() else{return Err("La configuración debe ser un objeto YAML".into());};
                if !frame.mapping{continue;}
                if frame.pending.is_none(){
                    if tag.is_some()||value.len()>256||!frame.keys.insert(value.clone()){return Err("Claves YAML repetidas o no compatibles".into());}
                    frame.pending=Some(value);
                }else{
                    let path=child_path(&mut stack)?;
                    if target_path(&path,targets) {
                        if tag.is_some(){return Err("El ajuste tiene una etiqueta YAML no compatible".into());}
                        let enabled=match value.as_str(){"true"=>true,"false"=>false,_=>return Err("El ajuste debe ser true o false".into())};
                        values.insert(path.join("."),enabled);
                    }
                }
            },
            Event::Alias(_)=>return Err("No se pudo comprobar una configuración con alias YAML".into()),
            Event::StreamEnd=>return Ok(values),
            _=>{},
        }
    }
    Err("El archivo YAML es demasiado complejo".into())
}

fn read(directory:&Path,name:&str,targets:&[&str])->Vec<Check>{
    let result=(||{
        let path=directory.join(name);
        let metadata=match fs::symlink_metadata(&path){Ok(value)=>value,Err(error)if error.kind()==std::io::ErrorKind::NotFound=>return Ok(HashMap::new()),Err(_)=>return Err("No se pudo leer el archivo".into())};
        if !metadata.is_file()||metadata.file_type().is_symlink(){return Err("El archivo no es un archivo normal".into());}
        let base=directory.canonicalize().map_err(|_|"No se pudo comprobar la carpeta del servidor")?;
        if !path.canonicalize().map_err(|_|"No se pudo comprobar el archivo")?.starts_with(base){return Err("El archivo apunta fuera del servidor".into());}
        if metadata.len()>512_000{return Err("El archivo supera el límite de 512 KB".into());}
        let mut data=Vec::new();fs::File::open(path).map_err(|_|"No se pudo leer el archivo")?.take(512_001).read_to_end(&mut data).map_err(|_|"No se pudo leer el archivo")?;
        if data.len()>512_000{return Err("El archivo supera el límite de 512 KB".into());}
        let text=std::str::from_utf8(&data).map_err(|_|"El archivo no contiene UTF-8 válido")?;
        booleans(text,targets)
    })();
    targets.iter().map(|key|Check{file:name.into(),key:(*key).into(),enabled:result.as_ref().ok().map(|values|values.get(*key).copied().unwrap_or(false)),problem:result.as_ref().err().cloned()}).collect()
}

pub fn inspect(server:&crate::model::Server)->Vec<Check>{
    if server.platform!="paper"{return vec![];}
    let directory=Path::new(&server.directory);
    let mut checks=read(directory,"spigot.yml",&["settings.bungeecord"]);
    let mut version=server.version.split('.').filter_map(|part|part.parse::<u32>().ok());
    let major=version.next().unwrap_or(1);let minor=version.next().unwrap_or(0);
    let modern=major>1||minor>=19;
    checks.extend(if modern{read(directory,"config/paper-global.yml",&["proxies.velocity.enabled","proxies.proxy-protocol"])}else{read(directory,"paper.yml",&["settings.velocity-support.enabled","settings.use-proxy-protocol"])});
    checks
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_nested_and_flow_maps_without_matching_unrelated_or_secret_fields(){
        let text="proxies:\n  velocity: {enabled: true, secret: private-token}\n  proxy-protocol: false\nother:\n  enabled: true\n";
        let values=booleans(text,&["proxies.velocity.enabled","proxies.proxy-protocol"]).unwrap();
        assert_eq!(values.len(),2);assert_eq!(values["proxies.velocity.enabled"],true);assert_eq!(values["proxies.proxy-protocol"],false);
        assert!(!format!("{values:?}").contains("private-token"));
        assert!(booleans("'proxies.velocity.enabled': true",&["proxies.velocity.enabled"]).unwrap().is_empty());
    }
    #[test]
    fn ambiguity_and_bad_types_remain_unknown_instead_of_becoming_disabled(){
        for source in ["settings: {bungeecord: maybe}","settings: {bungeecord: true, bungeecord: false}","settings: {bungeecord: [true]}","settings: &s {bungeecord: true}\nother: *s","settings: [broken","---\nsettings: {}\n---\nsettings: {}"] {
            assert!(booleans(source,&["settings.bungeecord"]).is_err(),"{source}");
        }
        assert!(booleans("settings: {unrelated: [a, b]}",&["settings.bungeecord"]).unwrap().is_empty());
    }
    #[test]
    fn file_checks_bound_input_and_never_modify_configurations(){
        let root=std::env::temp_dir().join(format!("tch-auth-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        assert_eq!(read(&root,"spigot.yml",&["settings.bungeecord"])[0].enabled,Some(false));
        let raw="settings:\n  bungeecord: true\nsecret: do-not-display\n";fs::write(root.join("spigot.yml"),raw).unwrap();
        let checks=read(&root,"spigot.yml",&["settings.bungeecord"]);assert_eq!(checks[0].enabled,Some(true));
        assert!(!serde_json::to_string(&checks).unwrap().contains("do-not-display"));assert_eq!(fs::read_to_string(root.join("spigot.yml")).unwrap(),raw);
        fs::write(root.join("spigot.yml"),vec![b'x';512_001]).unwrap();assert!(read(&root,"spigot.yml",&["settings.bungeecord"])[0].enabled.is_none());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn paper_versions_select_their_real_config_location_and_other_loaders_are_not_inspected(){
        let root=std::env::temp_dir().join(format!("tch-auth-version-{}",uuid::Uuid::new_v4()));fs::create_dir_all(root.join("config")).unwrap();
        fs::write(root.join("paper.yml"),"settings: {velocity-support: {enabled: true}}\n").unwrap();
        fs::write(root.join("config/paper-global.yml"),"proxies: {velocity: {enabled: false}, proxy-protocol: true}\n").unwrap();
        let mut server:crate::model::Server=serde_json::from_value(serde_json::json!({"id":"auth-fixture","name":"fixture","version":"1.18.2","platform":"paper","directory":root.to_str().unwrap(),"jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        let legacy=inspect(&server);assert_eq!(legacy[1].file,"paper.yml");assert_eq!(legacy[1].enabled,Some(true));
        for version in ["1.19","1.21.11","26.1"]{server.version=version.into();let checks=inspect(&server);assert_eq!(checks[1].file,"config/paper-global.yml");assert_eq!(checks[1].enabled,Some(false));assert_eq!(checks[2].enabled,Some(true));}
        server.platform="fabric".into();assert!(inspect(&server).is_empty());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn yaml_work_is_bounded_before_collecting_a_whole_document(){
        let deep=format!("{}false{}","[".repeat(70),"]".repeat(70));assert!(booleans(&format!("unused: {deep}"),&["settings.bungeecord"]).is_err());
        let wide=(0..11_000).map(|index|format!("key{index}: false\n")).collect::<String>();assert!(booleans(&wide,&["settings.bungeecord"]).is_err());
    }
}
