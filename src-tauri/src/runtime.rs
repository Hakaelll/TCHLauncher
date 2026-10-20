use crate::model::Server;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct RuntimeInfo {
    pub path: Option<String>,
    pub version: Option<String>,
    pub distribution: Option<String>,
    pub managed: bool,
    pub status: String,
    pub error: Option<String>,
    pub required_major: Option<u64>,
    pub compatibility: String,
}

pub fn inspect(root: &Path,server: &Server) -> RuntimeInfo {
    let required=cached_requirement(root,&server.version);
    let mut info = RuntimeInfo {path:server.java_path.clone(),version:None,distribution:None,managed:false,status:"missing".into(),error:None,required_major:required,compatibility:"unknown".into()};
    let Some(path) = server.java_path.as_ref().map(Path::new) else {return info;};
    if !path.is_file() {info.error=Some("No se encuentra el ejecutable de Java. Reinstala Temurin.".into()); return info;}
    info.managed = path.canonicalize().ok().zip(root.join("runtimes").canonicalize().ok()).is_some_and(|(path,managed)|path.starts_with(managed));
    match version_output(path) {
        Ok(output) => {
            info.version = parse_version(&output);
            if info.version.is_none() {info.status="invalid".into();info.error=Some("Java no devolvió una versión reconocible.".into());return info;}
            info.distribution = Some(if output.contains("Temurin") {"Eclipse Temurin"} else if output.contains("OpenJDK") || output.contains("openjdk") {"OpenJDK"} else {"Java"}.into());
            info.status="ready".into();
            if let Some(required)=required {
                if check_compatibility(info.version.as_deref().unwrap(),required).is_ok() {info.compatibility="compatible".into();}
                else {info.compatibility="incompatible".into();info.error=Some(format!("Minecraft {} requiere Java {required}. El ejecutable seleccionado usa Java {}.",server.version,info.version.as_deref().unwrap()));}
            }
        },
        Err(error) => {info.status="invalid".into();info.error=Some(error);},
    }
    info
}

pub fn check_compatibility(version:&str,required:u64)->Result<(),String> {
    let major=if version.starts_with("1.") {version.split('.').nth(1)} else {version.split('.').next()}.and_then(|part|part.parse::<u64>().ok());
    if major!=Some(required) {return Err(format!("Este servidor requiere Java {required}; el ejecutable elegido devuelve Java {version}. El runtime actual se conservará."));}
    Ok(())
}

pub fn validate_configured(root:&Path,server:&Server)->Result<(),String> {
    let path=server.java_path.as_deref().ok_or("Instala o selecciona Java antes de iniciar el servidor")?;
    let output=version_output(Path::new(path))?;
    let version=parse_version(&output).ok_or("El runtime Java no devolvió una versión reconocible. Compruébalo en Configuración.")?;
    if let Some(required)=cached_requirement(root,&server.version) {check_compatibility(&version,required)?;}
    Ok(())
}

fn requirement_path(root:&Path,version:&str)->Option<std::path::PathBuf> {
    if version.is_empty() || version.len()>64 || !version.bytes().all(|c|c.is_ascii_alphanumeric() || b".-".contains(&c)) {return None;}
    Some(root.join("metadata").join("java-requirements").join(format!("{version}.json")))
}

pub fn cached_requirement(root:&Path,version:&str)->Option<u64> {
    let path=requirement_path(root,version)?;
    if fs::metadata(&path).ok()?.len()>1024 {return None;}
    let data:serde_json::Value=serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if data["minecraft"].as_str()!=Some(version) {return None;}
    data["javaMajor"].as_u64().filter(|major|(8..=99).contains(major))
}

pub fn remember_requirement(root:&Path,version:&str,major:u64)->Result<(),String> {
    if !(8..=99).contains(&major) {return Err("Versión requerida de Java no válida".into());}
    let path=requirement_path(root,version).ok_or("Versión Minecraft no válida")?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e|e.to_string())?;
    let temporary=path.with_extension(format!("part-{}",uuid::Uuid::new_v4()));
    fs::write(&temporary,serde_json::to_vec(&serde_json::json!({"minecraft":version,"javaMajor":major})).unwrap()).map_err(|e|e.to_string())?;
    let result=crate::download::replace_file(&temporary,&path);
    if result.is_err(){let _=fs::remove_file(temporary);}
    result
}

pub fn candidate(root:&Path,server:&Server,path:&Path,required:u64)->Result<RuntimeInfo,String> {
    let name=path.file_name().and_then(|name|name.to_str()).ok_or("Selecciona el ejecutable Java")?;
    if !(name=="java" || (cfg!(windows) && name.eq_ignore_ascii_case("java.exe"))) {return Err("Selecciona java.exe en Windows o el ejecutable java en Linux, dentro de la carpeta bin del runtime.".into());}
    let canonical=path.canonicalize().map_err(|e|format!("No se pudo abrir el ejecutable Java: {e}"))?;
    if !canonical.is_file() {return Err("La ruta elegida no es un ejecutable Java".into());}
    let mut selected=server.clone();selected.java_path=Some(canonical.to_string_lossy().into());
    let info=inspect(root,&selected);
    if info.status!="ready" {return Err(info.error.unwrap_or_else(||"No se pudo verificar Java".into()));}
    check_compatibility(info.version.as_deref().unwrap(),required)?;
    Ok(RuntimeInfo{required_major:Some(required),compatibility:"compatible".into(),..info})
}

pub fn parse_version(output: &str) -> Option<String> {
    let line = output.lines().find(|line|line.starts_with("java version ") || line.starts_with("openjdk version "))?;
    let version = line.split('"').nth(1)?;
    (!version.is_empty() && version.len() <= 64 && !version.chars().any(char::is_control)).then(||version.into())
}

pub fn version_output(path: &Path) -> Result<String,String> {
    version_output_with_args(path,&[])
}

pub fn version_output_with_args(path: &Path,args:&[String]) -> Result<String,String> {
    // Use a temporary file rather than pipes: an unexpected executable cannot fill a pipe and block inspection.
    let output_path = std::env::temp_dir().join(format!("tch-java-{}.log",uuid::Uuid::new_v4()));
    let result = (|| {
        let output = fs::File::create(&output_path).map_err(|e|e.to_string())?;
        let mut command = Command::new(path);
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.args(args).arg("-version").stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output)
            .spawn().map_err(|e|format!("No se pudo ejecutar Java: {e}"))?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    use std::io::Read;
                    let mut text = String::new();
                    fs::File::open(&output_path).map_err(|e|e.to_string())?.take(16_384).read_to_string(&mut text).map_err(|e|e.to_string())?;
                    if !status.success() {
                        let detail=text.lines().take(2).collect::<Vec<_>>().join(" ").chars().take(400).collect::<String>();
                        return Err(format!("Java no pudo completar la comprobación de versión. {detail}"));
                    }
                    return Ok(text);
                },
                Ok(None) if started.elapsed() < Duration::from_secs(5) => std::thread::sleep(Duration::from_millis(25)),
                Ok(None) => {let _=child.kill();let _=child.wait();return Err("Java no respondió en cinco segundos. Reinstala el runtime.".into());},
                Err(error) => {let _=child.kill();let _=child.wait();return Err(format!("No se pudo comprobar Java: {error}"));},
            }
        }
    })();
    let _=fs::remove_file(output_path);
    result
}
