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
}

pub fn inspect(root: &Path,server: &Server) -> RuntimeInfo {
    let mut info = RuntimeInfo {path:server.java_path.clone(),version:None,distribution:None,managed:false,status:"missing".into(),error:None};
    let Some(path) = server.java_path.as_ref().map(Path::new) else {return info;};
    if !path.is_file() {info.error=Some("No se encuentra el ejecutable de Java. Reinstala Temurin.".into()); return info;}
    info.managed = path.canonicalize().ok().zip(root.join("runtimes").canonicalize().ok()).is_some_and(|(path,managed)|path.starts_with(managed));
    match version_output(path) {
        Ok(output) => {
            info.version = parse_version(&output);
            if info.version.is_none() {info.status="invalid".into();info.error=Some("Java no devolvió una versión reconocible.".into());return info;}
            info.distribution = Some(if output.contains("Temurin") {"Eclipse Temurin"} else if output.contains("OpenJDK") || output.contains("openjdk") {"OpenJDK"} else {"Java"}.into());
            info.status="ready".into();
        },
        Err(error) => {info.status="invalid".into();info.error=Some(error);},
    }
    info
}

pub fn parse_version(output: &str) -> Option<String> {
    let line = output.lines().find(|line|line.starts_with("java version ") || line.starts_with("openjdk version "))?;
    let version = line.split('"').nth(1)?;
    (!version.is_empty() && version.len() <= 64 && !version.chars().any(char::is_control)).then(||version.into())
}

pub fn version_output(path: &Path) -> Result<String,String> {
    // Use a temporary file rather than pipes: an unexpected executable cannot fill a pipe and block inspection.
    let output_path = std::env::temp_dir().join(format!("tch-java-{}.log",uuid::Uuid::new_v4()));
    let result = (|| {
        let output = fs::File::create(&output_path).map_err(|e|e.to_string())?;
        let mut command = Command::new(path);
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.arg("-version").stdin(Stdio::null()).stdout(output.try_clone().map_err(|e|e.to_string())?).stderr(output)
            .spawn().map_err(|e|format!("No se pudo ejecutar Java: {e}"))?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {return Err("Java no pudo completar la comprobación de versión.".into());}
                    use std::io::Read;
                    let mut text = String::new();
                    fs::File::open(&output_path).map_err(|e|e.to_string())?.take(16_384).read_to_string(&mut text).map_err(|e|e.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_temurin_and_legacy_java_versions_without_inventing_a_version() {
        assert_eq!(parse_version("openjdk version \"21.0.8\" 2025-07-15 LTS\nOpenJDK Runtime Environment Temurin-21.0.8+9"),Some("21.0.8".into()));
        assert_eq!(parse_version("java version \"1.8.0_462\""),Some("1.8.0_462".into()));
        assert_eq!(parse_version("Java has failed to start"),None);
    }
}
