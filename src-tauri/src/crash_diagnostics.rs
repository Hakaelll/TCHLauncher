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
pub struct Report{pub server_id:String,pub session_id:String,pub ended_at:i64,pub exit_code:Option<i32>,pub stop_requested:bool,#[serde(default)]pub forced:bool,pub output_complete:bool,pub diagnoses:Vec<Diagnosis>,pub recent_files:Vec<ChangedFile>}
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
        let evidence:Vec<_>=lines.iter().filter(|line|{
            let parsed=crate::log::parse("",line);
            if matches!(parsed.level.as_str(),"chat"|"player"|"command"){return false;}
            let clean=crate::log::without_terminal_colors(&parsed.line);
            let lower=clean.to_ascii_lowercase();
            let diagnostic=match crate::log::header_level(&clean){
                Some(level)=>matches!(level,"WARN"|"WARNING"|"ERROR"|"SEVERE"|"FATAL"),
                None=>parsed.level=="error"||patterns.iter().any(|pattern|lower.trim_start().trim_start_matches("# ").starts_with(pattern)),
            };
            diagnostic&&patterns.iter().any(|pattern|lower.contains(pattern))
        }).take(4).map(|line|line.chars().take(1000).collect()).collect();
        if !evidence.is_empty(){result.push(Diagnosis{kind:kind.into(),title:title.into(),explanation:explanation.into(),recommendation:recommendation.into(),action:action.into(),evidence});}
    }
    if result.is_empty(){result.push(Diagnosis{kind:"unknown".into(),title:"El proceso terminó sin completar su ejecución".into(),explanation:"No se identificó una causa conocida en la salida conservada de este arranque.".into(),recommendation:"Abre la consola y revisa las últimas líneas antes de volver a iniciar.".into(),action:"console".into(),evidence:lines.iter().rev().take(6).cloned().collect::<Vec<_>>().into_iter().rev().collect()});}result
}
pub fn analyze_exit(lines:&[String],forced:bool)->Vec<Diagnosis>{
    let mut diagnoses=analyze(lines);
    if forced{
        diagnoses.retain(|diagnosis|diagnosis.kind!="unknown");
        diagnoses.insert(0,Diagnosis{kind:"forced_stop".into(),title:"Cierre forzado solicitado".into(),explanation:"El launcher terminó este proceso después de tu confirmación. El guardado del mundo no queda confirmado por este cierre.".into(),recommendation:"Revisa la consola y el mundo antes de continuar. No se generó una copia automática de parada ni un reinicio automático por este cierre.".into(),action:"console".into(),evidence:Vec::new()});
        diagnoses.truncate(6);
    }
    diagnoses
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
    persist(directory,report,"last-exit.json")?;
    if report.exit_code!=Some(0)||report.forced||report.diagnoses.iter().any(|diagnosis|diagnosis.kind=="process_monitor"){persist(directory,report,"last-failure.json")?;}
    Ok(())
}
fn persist(directory:&Path,report:&Report,name:&str)->Result<(),String>{
    let folder=directory.join("launcher-logs");let meta=fs::symlink_metadata(&folder).map_err(|e|e.to_string())?;
    if !meta.is_dir()||meta.file_type().is_symlink(){return Err("La carpeta de diagnóstico no es segura".into());}
    let temporary=folder.join(format!("diagnostic-{}.part",uuid::Uuid::new_v4()));
    let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;serde_json::to_writer(&mut file,report).map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);crate::download::replace_file(&temporary,&folder.join(name))})();
    if result.is_err(){let _=fs::remove_file(&temporary);}result
}
pub fn load(directory:&Path,id:&str,session:&str)->Result<Option<Report>,String>{
    read_report(directory,id,Some(session),"last-exit.json")
}
pub fn last_failure(directory:&Path,id:&str)->Result<Option<Report>,String>{
    // The previous format only stored the latest exit. Preserve access on upgrade.
    let report=match read_report(directory,id,None,"last-failure.json")?{
        Some(report)=>Some(report),None=>read_report(directory,id,None,"last-exit.json")?,
    };
    Ok(report.filter(|report|report.exit_code!=Some(0)||report.forced))
}
fn read_report(directory:&Path,id:&str,session:Option<&str>,name:&str)->Result<Option<Report>,String>{
    let folder=directory.join("launcher-logs");
    match fs::symlink_metadata(&folder){Ok(meta) if meta.is_dir()&&!meta.file_type().is_symlink()=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),_=>return Err("La carpeta de diagnóstico no es segura".into())}
    let path=folder.join(name);let meta=match fs::symlink_metadata(&path){Ok(meta)=>meta,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(None),Err(error)=>return Err(error.to_string())};
    if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>256_000{return Err("El diagnóstico guardado no es válido".into());}
    let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(256_001).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    let report:Report=serde_json::from_slice(&bytes).map_err(|_|"El diagnóstico guardado no es válido")?;
    if report.server_id!=id||session.is_some_and(|session|report.session_id!=session){return Ok(None);}
    if report.diagnoses.len()>6||report.recent_files.len()>32||report.diagnoses.iter().any(|value|value.evidence.len()>6||!matches!(value.action.as_str(),"settings"|"network"|"content"|"console")){return Err("El diagnóstico guardado no es válido".into());}Ok(Some(report))
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn forced_exit_is_durable_even_with_zero_code_and_legacy_reports_remain_readable(){
        let directory=std::env::temp_dir().join(format!("tch-forced-history-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();
        let mut report=Report{server_id:"server".into(),session_id:"forced".into(),ended_at:1000,exit_code:Some(0),stop_requested:true,forced:true,output_complete:true,diagnoses:analyze_exit(&[],true),recent_files:Vec::new()};
        save(&directory,&report).unwrap();
        let saved=last_failure(&directory,"server").unwrap().unwrap();assert!(saved.forced);assert_eq!(saved.exit_code,Some(0));assert_eq!(saved.diagnoses[0].kind,"forced_stop");assert!(!saved.diagnoses.iter().any(|diagnosis|diagnosis.kind=="unknown"));
        report.session_id="clean".into();report.forced=false;report.diagnoses=analyze_exit(&[],false);save(&directory,&report).unwrap();
        assert_eq!(last_failure(&directory,"server").unwrap().unwrap().session_id,"forced");
        let mut legacy=serde_json::to_value(&report).unwrap();legacy.as_object_mut().unwrap().remove("forced");
        fs::write(directory.join("launcher-logs/last-exit.json"),serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(!load(&directory,"server","clean").unwrap().unwrap().forced);
        let diagnoses=analyze_exit(&["java.lang.OutOfMemoryError: Java heap space".into()],true);assert_eq!(diagnoses[0].kind,"forced_stop");assert_eq!(diagnoses[1].kind,"heap");
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]fn rules_distinguish_memory_causes_and_never_diagnose_chat(){
        let heap=analyze(&["java.lang.OutOfMemoryError: Java heap space".into()]);assert_eq!(heap.len(),1);assert_eq!(heap[0].kind,"heap");
        let native=analyze(&["java.lang.OutOfMemoryError: unable to create native thread".into()]);assert_eq!(native[0].kind,"memory");
        for(raw,kind)in [("java.lang.UnsupportedClassVersionError: Main","java_version"),("[12:00:00 WARN]: **** FAILED TO BIND TO PORT!","bind"),("Incompatible mods found!","mods"),("Error occurred while enabling Example v1","plugins")]{assert_eq!(analyze(&[raw.into()])[0].kind,kind);}
        assert_eq!(analyze(&["[12:00:00] [Server thread/INFO]: <Player> java.lang.OutOfMemoryError: Java heap space".into()])[0].kind,"unknown");
        for text in ["UnsupportedClassVersionError is handled","Incompatible mods found! example","Error occurred while enabling Example v1","java.lang.OutOfMemoryError: Java heap space"]{
            for prefix in ["[12:00:00] [Server thread/INFO]: ","2026-10-05 12:00:00,001 main INFO "]{
                assert_eq!(analyze(&[format!("{prefix}{text}")])[0].kind,"unknown");
            }
        }
        assert_eq!(analyze(&["# Native memory allocation (malloc) failed to allocate 2048 bytes".into()])[0].kind,"memory");
        let mut output=Output::default();for _ in 0..100{output.push(&"x".repeat(9000));}assert_eq!(output.lines().len(),64);assert_eq!(output.lines()[0].len(),2048);
    }
    #[test]fn last_failure_survives_successful_sessions_and_rejects_other_servers(){
        let directory=std::env::temp_dir().join(format!("tch-diagnostic-history-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();
        let id=uuid::Uuid::new_v4().to_string();let session=uuid::Uuid::new_v4().to_string();
        let mut report=Report{server_id:id.clone(),session_id:session.clone(),ended_at:1000,exit_code:Some(1),stop_requested:false,forced:false,output_complete:true,diagnoses:analyze(&["UnsupportedClassVersionError".into()]),recent_files:Vec::new()};
        // Read the former single-file format after a launcher restart.
        persist(&directory,&report,"last-exit.json").unwrap();assert_eq!(last_failure(&directory,&id).unwrap().unwrap().session_id,session);
        save(&directory,&report).unwrap();report.session_id=uuid::Uuid::new_v4().to_string();report.ended_at=2000;report.exit_code=Some(0);save(&directory,&report).unwrap();
        assert_eq!(load(&directory,&id,&report.session_id).unwrap().unwrap().exit_code,Some(0));
        let previous=last_failure(&directory,&id).unwrap().unwrap();assert_eq!(previous.session_id,session);assert_eq!(previous.ended_at,1000);assert_eq!(previous.diagnoses[0].kind,"java_version");assert!(last_failure(&directory,"other").unwrap().is_none());
        report.session_id=uuid::Uuid::new_v4().to_string();report.ended_at=3000;report.exit_code=None;report.stop_requested=true;save(&directory,&report).unwrap();
        let latest=last_failure(&directory,&id).unwrap().unwrap();assert_eq!(latest.session_id,report.session_id);assert!(latest.stop_requested);assert_eq!(latest.exit_code,None);
        fs::write(directory.join("launcher-logs/last-failure.json"),b"broken").unwrap();assert!(last_failure(&directory,&id).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]fn persistence_matches_the_process_session_and_keeps_recent_changes_separate(){
        let directory=std::env::temp_dir().join(format!("tch-diagnostics-{}",uuid::Uuid::new_v4()));fs::create_dir_all(directory.join("launcher-logs")).unwrap();fs::create_dir_all(directory.join("mods")).unwrap();fs::write(directory.join("mods/example.jar"),b"jar").unwrap();fs::write(directory.join("mods/config.json"),b"config").unwrap();
        let time=chrono::Utc::now().timestamp_millis()+100;let files=recent_files(&directory,time);assert_eq!(files.len(),1);assert_eq!(files[0].name,"example.jar");
        let report=Report{server_id:"server".into(),session_id:"one".into(),ended_at:time,exit_code:Some(1),stop_requested:false,forced:false,output_complete:true,diagnoses:analyze(&["unknown failure".into()]),recent_files:files};save(&directory,&report).unwrap();assert_eq!(load(&directory,"server","one").unwrap().unwrap().exit_code,Some(1));assert!(load(&directory,"server","two").unwrap().is_none());assert!(load(&directory,"other","one").unwrap().is_none());
        fs::write(directory.join("launcher-logs/last-exit.json"),b"broken").unwrap();assert!(load(&directory,"server","one").is_err());fs::remove_dir_all(directory).unwrap();
    }
}
