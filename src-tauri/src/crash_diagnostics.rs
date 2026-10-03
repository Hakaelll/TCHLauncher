//! Deterministic diagnostics from bounded output of one native process session.
use serde::{Deserialize,Serialize};
use std::{collections::VecDeque,fs,io::Read,path::Path};

#[derive(Default)]
pub struct Output{lines:VecDeque<String>}
impl Output{
    pub fn push(&mut self,line:&str){
        let parsed=crate::log::parse("",line);
        if matches!(parsed.level.as_str(),"chat"|"player"|"command"){return;}
        let line=crate::log::without_terminal_colors(line).chars().take(2048).collect();
        if self.lines.len()==64{self.lines.pop_front();}self.lines.push_back(line);
    }
    pub fn lines(&self)->Vec<String>{self.lines.iter().cloned().collect()}
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Diagnosis{pub kind:String,pub title:String,pub explanation:String,pub recommendation:String,pub action:String,pub evidence:Vec<String>}
#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Report{pub server_id:String,pub session_id:String,pub ended_at:i64,pub exit_code:Option<i32>,pub stop_requested:bool,pub output_complete:bool,pub diagnoses:Vec<Diagnosis>,pub recent_files:Vec<ChangedFile>}
#[derive(Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ChangedFile{pub name:String,pub folder:String,pub modified_at:i64}

pub fn analyze(lines:&[String])->Vec<Diagnosis>{
    let rules:[(&str,&str,&str,&str,&str,&[&str]);6]=[
        ("java_version","Java no es compatible","Java no reconoce la versión de una clase que el servidor intenta cargar.","Comprueba el runtime requerido por Minecraft y por tus mods o plugins.","settings",&["unsupportedclassversionerror"]),
        ("heap","Se agotó la memoria de Java","La salida indica que se agotó el espacio para objetos de Java.","Revisa la RAM máxima y los mods. Aumenta la memoria solo si el equipo tiene recursos libres.","settings",&["outofmemoryerror: java heap space","outofmemoryerror: gc overhead limit exceeded"]),
        ("memory","Java no pudo obtener memoria","Java informó de un fallo de memoria; puede afectar al sistema, a los hilos o a los metadatos de clases.","Revisa el detalle del error y los argumentos JVM. No todos estos fallos se resuelven aumentando la RAM máxima.","settings",&["outofmemoryerror","could not reserve enough space","native memory allocation (malloc) failed"]),
        ("bind","No se pudo usar la dirección del servidor","La salida indica que Java no pudo abrir su dirección o puerto.","Comprueba el puerto en Conexión, otros servidores abiertos y la dirección configurada.","network",&["failed to bind to port","java.net.bindexception","bind(..) failed"]),
        ("mods","Falló la carga de mods","El loader informó de un problema al preparar los mods.","Revisa las versiones y dependencias indicadas en el detalle antes de volver a iniciar.","content",&["incompatible mods found!","modresolutionexception","loadingfailedevent","mod loading has failed","failed to load mods","modloadingexception"]),
        ("plugins","Un plugin informó de un fallo de carga","La salida contiene un error durante la carga o activación de un plugin; puede ser un fallo adicional al cierre.","Revisa el plugin y su compatibilidad con esta versión de Paper.","content",&["invalidpluginexception","error occurred while enabling","could not load 'plugins/"]),
    ];
    let mut result=Vec::new();
    for(kind,title,explanation,recommendation,action,patterns)in rules{
        if kind=="memory"&&result.iter().any(|item:&Diagnosis|item.kind=="heap"){continue;}
        let evidence:Vec<_>=lines.iter().filter(|line|{let parsed=crate::log::parse("",line);!matches!(parsed.level.as_str(),"chat"|"player"|"command")&&patterns.iter().any(|pattern|line.to_ascii_lowercase().contains(pattern))}).take(4).map(|line|line.chars().take(1000).collect()).collect();
        if !evidence.is_empty(){result.push(Diagnosis{kind:kind.into(),title:title.into(),explanation:explanation.into(),recommendation:recommendation.into(),action:action.into(),evidence});}
    }
    if result.is_empty(){result.push(Diagnosis{kind:"unknown".into(),title:"El proceso terminó sin completar su ejecución".into(),explanation:"No se identificó una causa conocida en la salida conservada de este arranque.".into(),recommendation:"Abre la consola y revisa las últimas líneas antes de volver a iniciar.".into(),action:"console".into(),evidence:lines.iter().rev().take(6).cloned().collect::<Vec<_>>().into_iter().rev().collect()});}result
}
pub fn recent_files(directory:&Path,time:i64)->Vec<ChangedFile>{
    let mut files=Vec::new();
    for folder in ["mods","plugins"]{
        let path=directory.join(folder);let Ok(meta)=fs::symlink_metadata(&path)else{continue;};if !meta.is_dir()||meta.file_type().is_symlink(){continue;}
        let Ok(entries)=fs::read_dir(path)else{continue;};
        for entry in entries.flatten(){
            let name=entry.file_name().to_string_lossy().into_owned();if name.len()>255||!name.to_ascii_lowercase().ends_with(".jar"){continue;}
            let Ok(meta)=fs::symlink_metadata(entry.path())else{continue;};if !meta.is_file()||meta.file_type().is_symlink(){continue;}
            let Some(modified)=meta.modified().ok().and_then(|value|value.duration_since(std::time::UNIX_EPOCH).ok())else{continue;};let modified_at=modified.as_millis().min(i64::MAX as u128) as i64;
            if modified_at>=time-86_400_000&&modified_at<=time{files.push(ChangedFile{name,folder:folder.into(),modified_at});}
        }
    }
    files.sort_by(|a,b|b.modified_at.cmp(&a.modified_at).then(a.name.cmp(&b.name)));files.truncate(32);files
}
pub fn save(directory:&Path,report:&Report)->Result<(),String>{
    let folder=directory.join("launcher-logs");let meta=fs::symlink_metadata(&folder).map_err(|e|e.to_string())?;
    if !meta.is_dir()||meta.file_type().is_symlink(){return Err("La carpeta de diagnóstico no es segura".into());}
    let temporary=folder.join(format!("diagnostic-{}.part",uuid::Uuid::new_v4()));
    let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;serde_json::to_writer(&mut file,report).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);crate::download::replace_file(&temporary,&folder.join("last-exit.json"))})();
    if result.is_err(){let _=fs::remove_file(&temporary);}result
}
pub fn load(directory:&Path,id:&str,session:&str)->Result<Option<Report>,String>{
    let folder=directory.join("launcher-logs");
    match fs::symlink_metadata(&folder){Ok(meta) if meta.is_dir()&&!meta.file_type().is_symlink()=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),_=>return Err("La carpeta de diagnóstico no es segura".into())}
    let path=folder.join("last-exit.json");let meta=match fs::symlink_metadata(&path){Ok(meta)=>meta,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string())};
    if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>256_000{return Err("El diagnóstico guardado no es válido".into());}
    let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(256_001).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    let report:Report=serde_json::from_slice(&bytes).map_err(|_|"El diagnóstico guardado no es válido")?;
    if report.server_id!=id||report.session_id!=session{return Ok(None);}
    if report.diagnoses.len()>6||report.recent_files.len()>32||report.diagnoses.iter().any(|value|value.evidence.len()>6||!matches!(value.action.as_str(),"settings"|"network"|"content"|"console")){return Err("El diagnóstico guardado no es válido".into());}Ok(Some(report))
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn rules_distinguish_memory_causes_and_never_diagnose_chat(){
        let heap=analyze(&["java.lang.OutOfMemoryError: Java heap space".into()]);assert_eq!(heap.len(),1);assert_eq!(heap[0].kind,"heap");
        let native=analyze(&["java.lang.OutOfMemoryError: unable to create native thread".into()]);assert_eq!(native[0].kind,"memory");
        for(raw,kind)in [("java.lang.UnsupportedClassVersionError: Main","java_version"),("[12:00:00 WARN]: **** FAILED TO BIND TO PORT!","bind"),("Incompatible mods found!","mods"),("Error occurred while enabling Example v1","plugins")]{assert_eq!(analyze(&[raw.into()])[0].kind,kind);}
        assert_eq!(analyze(&["[12:00:00] [Server thread/INFO]: <Player> java.lang.OutOfMemoryError: Java heap space".into()])[0].kind,"unknown");
        let mut output=Output::default();for _ in 0..100{output.push(&"x".repeat(9000));}assert_eq!(output.lines().len(),64);assert_eq!(output.lines()[0].len(),2048);
    }
    #[test]fn persistence_matches_the_process_session_and_keeps_recent_changes_separate(){
        let directory=std::env::temp_dir().join(format!("tch-diagnostics-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();fs::create_dir_all(directory.join("mods")).unwrap();fs::write(directory.join("mods/example.jar"),b"jar").unwrap();fs::write(directory.join("mods/config.json"),b"config").unwrap();
        let time=chrono::Utc::now().timestamp_millis()+100;let files=recent_files(&directory,time);assert_eq!(files.len(),1);assert_eq!(files[0].name,"example.jar");
        let report=Report{server_id:"server".into(),session_id:"one".into(),ended_at:time,exit_code:Some(1),stop_requested:false,output_complete:true,diagnoses:analyze(&["unknown failure".into()]),recent_files:files};save(&directory,&report).unwrap();assert_eq!(load(&directory,"server","one").unwrap().unwrap().exit_code,Some(1));assert!(load(&directory,"server","two").unwrap().is_none());assert!(load(&directory,"other","one").unwrap().is_none());
        fs::write(directory.join("launcher-logs/last-exit.json"),b"broken").unwrap();assert!(load(&directory,"server","one").is_err());fs::remove_dir_all(directory).unwrap();
    }
}
