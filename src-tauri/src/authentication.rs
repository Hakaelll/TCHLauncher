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
