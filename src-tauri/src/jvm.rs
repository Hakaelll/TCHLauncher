use crate::model::Server;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

// Windows canonicalization returns a verbatim path. Java's JAR URL classloader
// cannot resolve that prefix, even when the native executable opens the JAR.
pub(crate) fn java_path(path: &Path) -> String {
    let value=path.to_string_lossy();
    if cfg!(windows) {
        if let Some(rest)=value.strip_prefix("\\\\?\\UNC\\") {return format!("\\\\{rest}");}
        if let Some(rest)=value.strip_prefix("\\\\?\\") {return rest.to_owned();}
    }
    value.into_owned()
}


const RECOMMENDED: [&str; 2] = ["-XX:+UseG1GC", "-XX:+ParallelRefProcEnabled"];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Settings { pub mode: String, pub arguments: String }

impl Default for Settings {
    fn default()->Self {Self {mode:"recommended".into(),arguments:String::new()}}
}

pub fn options(settings:&Settings)->Result<Vec<String>,String> {
    if settings.mode=="recommended" {return Ok(RECOMMENDED.iter().map(|s|s.to_string()).collect());}
    if settings.mode!="custom" {return Err("Perfil JVM no válido".into());}
    if settings.arguments.len()>8192 {return Err("Los argumentos no pueden superar 8192 bytes".into());}
    let mut result=Vec::new();
    for (index,line) in settings.arguments.lines().enumerate() {
        let argument=line.trim();
        if argument.is_empty(){continue;}
        let invalid=||format!("Revisa la línea {}: escribe una opción JVM por línea, sin comillas exteriores.",index+1);
        if argument.len()>1024 || argument.chars().any(char::is_control) || !argument.starts_with('-') {return Err(invalid());}
        let lower=argument.to_ascii_lowercase();
        if ["-x","-xinternalversion"].contains(&lower.as_str()) {return Err("Las opciones que muestran ayuda o terminan Java no son argumentos de arranque.".into());}
        if lower.starts_with("-xms") || lower.starts_with("-xmx") || lower.starts_with("-xx:initialheapsize") || lower.starts_with("-xx:maxheapsize") {
            return Err(format!("Línea {}: configura la RAM en Capacidad, para mantener sus límites sincronizados.",index+1));
        }
        if lower.starts_with("-dfabric.") || ["-djava.class.path=","-djava.home=","-duser.dir=","-dlibrarydirectory=","-dlegacyclasspath=","-dignorelist=","-dfml."].iter().any(|prefix|lower.starts_with(prefix)) {
            return Err(format!("Línea {}: esa propiedad está gestionada por el launcher.",index+1));
        }
        // Accept VM options only. Launcher options, argument files and replacement entrypoints
        // must not override the selected server, classpath or the version verification command.
        if !(argument.starts_with("-XX:") || argument.starts_with("-D") || argument.starts_with("-X") || ["-ea","-da","-esa","-dsa"].contains(&argument) || argument.starts_with("-ea:") || argument.starts_with("-da:") || argument.starts_with("-javaagent:") || argument.starts_with("-agentlib:") || argument.starts_with("-agentpath:")) {
            return Err(invalid());
        }
        if lower.starts_with("-xx:flags=") || lower.starts_with("-xx:vmoptionsfile=") {return Err("Los archivos externos de argumentos no están permitidos; pega sus opciones aquí.".into());}
        if argument.contains(char::is_whitespace) && !argument.starts_with("-D") && !argument.starts_with("-javaagent:") && !argument.starts_with("-agentpath:") {return Err(invalid());}
        result.push(argument.to_string());
        if result.len()>64 {return Err("Usa como máximo 64 argumentos JVM".into());}
    }
    Ok(result)
}

pub fn get(root:&Path,id:&str)->Result<Settings,String> {
    crate::storage::get(root,id)?;
    crate::storage::connect(root)?.query_row("SELECT mode,arguments FROM jvm_settings WHERE server_id=?1",[id],|row|Ok(Settings{mode:row.get(0)?,arguments:row.get(1)?})).optional().map_err(|e|e.to_string()).map(|value|value.unwrap_or_default())
}

pub fn save(root:&Path,id:&str,settings:&Settings)->Result<(),String> {
    crate::storage::get(root,id)?;
    options(settings)?;
    crate::storage::connect(root)?.execute("INSERT INTO jvm_settings(server_id,mode,arguments) VALUES(?1,?2,?3) ON CONFLICT(server_id) DO UPDATE SET mode=excluded.mode,arguments=excluded.arguments",params![id,settings.mode,if settings.mode=="recommended" {""} else {&settings.arguments}]).map_err(|e|e.to_string())?;
    Ok(())
}

pub fn verify(server:&Server,settings:&Settings)->Result<(),String> {
    let options=options(settings)?;
    let java=server.java_path.as_deref().ok_or("Instala Java antes de comprobar argumentos personalizados")?;
    crate::runtime::version_output_with_args(Path::new(java),&options).map_err(|error|format!("Java no aceptó los argumentos. {error}"))?;
    Ok(())
}

pub fn save_checked(root:&Path,id:&str,settings:&Settings)->Result<(),String> {
    let server=crate::storage::get(root,id)?;
    if settings.mode=="custom" {
        crate::runtime::validate_configured(root,&server)?;
        verify(&server,settings)?;
    }
    save(root,id,settings)
}

pub fn vm_arguments(server:&Server,settings:&Settings)->Result<Vec<String>,String> {
    let mut args=vec![format!("-Xms{}M",server.memory_min_mb),format!("-Xmx{}M",server.memory_max_mb)];
    args.extend(options(settings)?);
    if let Some(game)=crate::fabric::game_jar(server) {
        if !game.is_file(){return Err("Falta el servidor Minecraft de Fabric. Comprueba la instalación.".into());}
        args.push(format!("-Dfabric.gameJarPath={}",java_path(&game)));
    }
    Ok(args)
}

pub fn launch_arguments(server:&Server,settings:&Settings)->Result<Vec<String>,String> {
    let mut args=vm_arguments(server,settings)?;
    if server.platform=="neoforge" {args.extend(crate::neoforge::launch_arguments(server)?);}
    else {args.extend(["-jar".into(),java_path(Path::new(server.jar_path.as_deref().ok_or("Instala primero el servidor")?)),"nogui".into()]);}
    Ok(args)
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Preview {pub arguments:Vec<String>,pub installed:bool}

pub fn preview(server:&Server,settings:&Settings)->Result<Preview,String> {
    let installed=server.jar_path.is_some();
    Ok(Preview{arguments:if installed {launch_arguments(server,settings)?} else {vm_arguments(server,settings)?},installed})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn custom(arguments:&str)->Settings {Settings{mode:"custom".into(),arguments:arguments.into()}}
    #[test]
    fn preserves_arguments_and_rejects_launcher_and_memory_overrides() {
        assert_eq!(options(&custom("-XX:+UseG1GC\n-Dexample.path=C:\\My Server\\data\n\n-ea")).unwrap(),vec!["-XX:+UseG1GC","-Dexample.path=C:\\My Server\\data","-ea"]);
        for invalid in ["-X","-Xinternalversion","-Xmx8G","-Xms1G","-XX:MaxHeapSize=1234","-jar other.jar","@flags.txt","--class-path=x","-version","-Dfabric.gameJarPath=other.jar","-DlibraryDirectory=other","-DlegacyClassPath=other","-Dfml.pluginLayerLibraries=other","-Duser.dir=x","-XX:Flags=flags.txt","-XX:+UseG1GC -XX:+UseZGC","-Dkey=a\u{0}b"] {assert!(options(&custom(invalid)).is_err(),"accepted {invalid}");}
        assert!(options(&custom(&"-ea\n".repeat(65))).is_err());
        assert!(options(&Settings{mode:"unknown".into(),arguments:String::new()}).is_err());
    }
    #[test]
    fn persists_per_server_and_keeps_old_settings_after_invalid_input() {
        let root=std::env::temp_dir().join(format!("tch-jvm-{}",uuid::Uuid::new_v4()));
        let server=crate::storage::create(&root,crate::model::CreateServer{name:"JVM test".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        assert_eq!(get(&root,&server.id).unwrap(),Settings::default());
        let settings=custom("-Dexample.name=A server");save(&root,&server.id,&settings).unwrap();
        assert!(save(&root,&server.id,&custom("-Xmx8G")).is_err());
        assert_eq!(get(&root,&server.id).unwrap(),settings);
        let mut installed=server.clone();installed.jar_path=Some("C:\\Server Folder\\server.jar".into());
        let args=launch_arguments(&installed,&settings).unwrap();
        assert_eq!(args,vec!["-Xms512M","-Xmx1024M","-Dexample.name=A server","-jar","C:\\Server Folder\\server.jar","nogui"]);
        if cfg!(windows) {
            installed.jar_path=Some(r"\\?\C:\Server Folder\server.jar".into());
            assert_eq!(launch_arguments(&installed,&settings).unwrap(),args);
            assert_eq!(java_path(Path::new(r"\\?\UNC\host\share\server.jar")),r"\\host\share\server.jar");
        }
        save(&root,&server.id,&Settings::default()).unwrap();
        assert_eq!(get(&root,&server.id).unwrap(),Settings::default());
        crate::storage::delete(&root,&server.id).unwrap();
        assert_eq!(crate::storage::connect(&root).unwrap().query_row("SELECT count(*) FROM jvm_settings",[],|row|row.get::<_,i64>(0)).unwrap(),0);
        let _=std::fs::remove_dir_all(root);
    }
    #[test]
    #[ignore="Requires TCH_TEST_JAVA pointing to an installed Java executable"]
    fn checks_profiles_with_real_java() {
        let executable=std::env::var("TCH_TEST_JAVA").expect("Set TCH_TEST_JAVA");
        let args=options(&Settings::default()).unwrap();
        crate::runtime::version_output_with_args(Path::new(&executable),&args).unwrap();
        let args=options(&custom("-Dexample.name=A server\n-XX:+UseG1GC")).unwrap();
        crate::runtime::version_output_with_args(Path::new(&executable),&args).unwrap();
        assert!(crate::runtime::version_output_with_args(Path::new(&executable),&["-XX:NotARealOption=1".into()]).is_err());
        let root=std::env::temp_dir().join(format!("tch-jvm-java-{}",uuid::Uuid::new_v4()));
        let server=crate::storage::create(&root,crate::model::CreateServer{name:"Java options test".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        crate::storage::update_java(&root,&server.id,Path::new(&executable)).unwrap();
        let valid=custom("-Dexample.name=A server\n-XX:+UseG1GC");
        save_checked(&root,&server.id,&valid).unwrap();
        assert!(save_checked(&root,&server.id,&custom("-XX:NotARealOption=1")).is_err());
        assert_eq!(get(&root,&server.id).unwrap(),valid);
        save_checked(&root,&server.id,&Settings::default()).unwrap();
        assert_eq!(get(&root,&server.id).unwrap(),Settings::default());
        let _=std::fs::remove_dir_all(root);
    }
}
