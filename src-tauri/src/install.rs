use crate::model::{Progress, Server, VersionInfo};
use crate::storage;
use reqwest::blocking::Client;
use serde_json::Value;
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;
use walkdir::WalkDir;

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

fn client() -> Result<Client, String> {
    Client::builder().user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION")," (https://github.com/Hakaelll/TCHLauncher)"))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" { attempt.error("HTTPS required") }
            else if attempt.previous().len() > 10 { attempt.error("too many redirects") }
            else { attempt.follow() }
        }))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build().map_err(|e| e.to_string())
}

fn json(client: &Client, url: &str) -> Result<Value, String> {
    if !url.starts_with("https://") { return Err("La fuente de descarga no utiliza HTTPS".into()); }
    client.get(url).timeout(std::time::Duration::from_secs(30)).send().and_then(|r| r.error_for_status()).and_then(|r| r.json())
        .map_err(|e| format!("No se pudo consultar {url}: {e}"))
}

pub fn versions() -> Result<Vec<VersionInfo>, String> {
    let data = json(&client()?, MANIFEST)?;
    let items = data["versions"].as_array().ok_or("Respuesta de versiones no válida")?;
    Ok(items.iter().filter_map(|v| {
        let kind = v["type"].as_str()?;
        if kind != "release" { return None; }
        Some(VersionInfo { id: v["id"].as_str()?.into(), kind: kind.into() })
    }).take(50).collect())
}

fn version_metadata(client: &Client, version: &str) -> Result<Value, String> {
    let manifest = json(client, MANIFEST)?;
    let entry = manifest["versions"].as_array().ok_or("Respuesta de Mojang no válida")?
        .iter().find(|v| v["id"].as_str() == Some(version))
        .ok_or_else(|| format!("Minecraft {version} no está en el catálogo oficial"))?;
    let url = entry["url"].as_str().ok_or("Falta la URL de la versión")?;
    json(client, url)
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

fn replace_file(part: &Path, target: &Path) -> Result<(), String> {
    if !target.exists() { return fs::rename(part,target).map_err(|e| e.to_string()); }
    let previous = target.with_extension(format!("previous-{}",Uuid::new_v4()));
    fs::rename(target,&previous).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(part,target) {
        let _ = fs::rename(&previous,target);
        return Err(e.to_string());
    }
    let _ = fs::remove_file(previous);
    Ok(())
}

fn download(client: &Client, url: &str, target: &Path, expected_hash: Option<(&str, &str)>, app: &AppHandle, server_id: &str, label: &str) -> Result<(), String> {
    if !url.starts_with("https://") { return Err("La descarga debe usar HTTPS".into()); }
    let mut response = client.get(url).send().and_then(|r| r.error_for_status())
        .map_err(|e| format!("No se pudo descargar {label}: {e}"))?;
    let total = response.content_length();
    if total.is_some_and(|n| n > 2_000_000_000) { return Err("La descarga supera el límite de 2 GB".into()); }
    let part = target.with_extension("part");
    let result = (|| -> Result<(), String> {
        let mut file = File::create(&part).map_err(|e| e.to_string())?;
        let mut sha1 = Sha1::new();
        let mut sha256 = Sha256::new();
        let mut received = 0;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = response.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 { break; }
            if received + n as u64 > 2_000_000_000 { return Err("La descarga supera el límite de 2 GB".into()); }
            file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            sha1.update(&buffer[..n]); sha256.update(&buffer[..n]);
            received += n as u64;
            let _ = app.emit("install-progress", Progress { server_id:server_id.into(), label:label.into(), received, total });
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        if let Some((kind, expected)) = expected_hash {
            let actual = match kind { "sha1" => hex(&sha1.finalize()), "sha256" => hex(&sha256.finalize()), _ => return Err("Algoritmo de hash no compatible".into()) };
            if !actual.eq_ignore_ascii_case(expected) { return Err(format!("La suma de comprobación de {label} no coincide")); }
        }
        replace_file(&part,target)?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&part); }
    result
}

fn platform_archive() -> Result<(&'static str, &'static str, &'static str), String> {
    let os = match std::env::consts::OS { "windows" => "windows", "linux" => "linux", _ => return Err("La instalación automática de Java no está disponible en este sistema".into()) };
    let arch = match std::env::consts::ARCH { "x86_64" => "x64", "aarch64" => "aarch64", _ => return Err("Arquitectura no compatible con Java gestionado".into()) };
    Ok((os, arch, if os == "windows" { "zip" } else { "tar.gz" }))
}

fn extract_archive(archive: &Path, destination: &Path, format: &str) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    if format == "zip" {
        let mut zip = zip::ZipArchive::new(File::open(archive).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        if zip.len() > 20_000 { return Err("El archivo Java tiene demasiadas entradas".into()); }
        let mut unpacked_size = 0u64;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            unpacked_size = unpacked_size.saturating_add(entry.size());
            if unpacked_size > 2_000_000_000 { return Err("El archivo Java extraído supera 2 GB".into()); }
            let relative = entry.enclosed_name().ok_or("Ruta no segura dentro del archivo Java")?;
            if entry.unix_mode().map(|m| m & 0o170000 == 0o120000).unwrap_or(false) {
                return Err("El archivo Java contiene un enlace simbólico no permitido".into());
            }
            let path = destination.join(relative);
            if entry.is_dir() { fs::create_dir_all(path).map_err(|e| e.to_string())?; }
            else {
                if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
                let mut out = File::create(path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
    } else {
        let gz = flate2::read::GzDecoder::new(File::open(archive).map_err(|e| e.to_string())?);
        let mut tar = tar::Archive::new(gz);
        let mut unpacked_size = 0u64;
        let mut entry_count = 0usize;
        for item in tar.entries().map_err(|e| e.to_string())? {
            let mut entry = item.map_err(|e| e.to_string())?;
            entry_count += 1;
            unpacked_size = unpacked_size.saturating_add(entry.size());
            if entry_count > 20_000 || unpacked_size > 2_000_000_000 { return Err("El archivo Java extraído supera los límites de seguridad".into()); }
            if !matches!(entry.header().entry_type(), tar::EntryType::Regular | tar::EntryType::Directory) { continue; }
            if !entry.unpack_in(destination).map_err(|e| e.to_string())? { return Err("Ruta no segura en el archivo Java".into()); }
        }
    }
    Ok(())
}

fn java_executable(root: &Path) -> Option<PathBuf> {
    let name = if cfg!(windows) { "java.exe" } else { "java" };
    WalkDir::new(root).max_depth(5).into_iter().filter_map(Result::ok)
        .find(|e| e.file_type().is_file() && e.file_name() == name && e.path().parent().and_then(|p| p.file_name()).is_some_and(|p| p == "bin"))
        .map(|e| e.into_path())
}

fn verify_java(path: &Path, major: u64) -> Result<(), String> {
    let output = Command::new(path).arg("-version").output().map_err(|e| format!("No se pudo ejecutar Java: {e}"))?;
    let version = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() || !version.contains(&format!("\"{major}.")) && !(major == 8 && version.contains("\"1.8.")) {
        return Err(format!("Java no devolvió la versión {major} esperada: {version}"));
    }
    Ok(())
}

fn ensure_java(root: &Path, major: u64, client: &Client, app: &AppHandle, server_id: &str) -> Result<PathBuf, String> {
    let target = root.join("runtimes").join(format!("temurin-{major}-{}-{}", std::env::consts::OS, std::env::consts::ARCH));
    if let Some(path) = java_executable(&target) { if verify_java(&path, major).is_ok() { return Ok(path); } }
    let (os, arch, format) = platform_archive()?;
    let meta = json(client, &format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture={arch}&image_type=jre&os={os}&vendor=eclipse"))?;
    let item = meta.as_array().and_then(|a| a.first()).ok_or("No se encontró un runtime Temurin compatible")?;
    let package = &item["binary"]["package"];
    let url = package["link"].as_str().ok_or("Temurin no proporcionó descarga")?;
    let checksum = package["checksum"].as_str().ok_or("Temurin no proporcionó checksum")?;
    let stage = root.join("runtimes").join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    let archive = stage.join(format!("java.{format}"));
    let result = (|| {
        download(client, url, &archive, Some(("sha256", checksum)), app, server_id, &format!("Java {major}"))?;
        let unpacked = stage.join("unpacked");
        extract_archive(&archive, &unpacked, format)?;
        let path = java_executable(&unpacked).ok_or("El paquete Java no contiene java ejecutable")?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).map_err(|e| e.to_string())?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;
        }
        verify_java(&path, major)?;
        let previous = target.with_extension(format!("previous-{}",Uuid::new_v4()));
        if target.exists() { fs::rename(&target,&previous).map_err(|e| e.to_string())?; }
        if let Err(e) = fs::rename(&unpacked, &target) {
            if previous.exists() { let _ = fs::rename(&previous,&target); }
            return Err(e.to_string());
        }
        if previous.exists() { let _ = fs::remove_dir_all(previous); }
        java_executable(&target).ok_or_else(|| "Java instalado pero no localizable".into())
    })();
    let _ = fs::remove_dir_all(&stage);
    result
}

pub fn install(root: &Path, server: &Server, app: &AppHandle) -> Result<(), String> {
    let client = client()?;
    let meta = version_metadata(&client, &server.version)?;
    let major = meta["javaVersion"]["majorVersion"].as_u64().ok_or("Mojang no indicó la versión Java necesaria")?;
    if let Some(jar) = &server.jar_path {
        if !Path::new(jar).is_file() { return Err("El archivo .jar importado ya no existe".into()); }
        let java = ensure_java(root,major,&client,app,&server.id)?;
        storage::update_java(root,&server.id,&java)?;
        return Ok(());
    }
    let (url, checksum): (String, Option<(&str, String)>) = if server.platform == "vanilla" {
        let artifact = &meta["downloads"]["server"];
        let url = artifact["url"].as_str().ok_or("Esta versión no tiene servidor Vanilla")?;
        let hash = artifact["sha1"].as_str().ok_or("Mojang no proporcionó checksum")?;
        (url.into(), Some(("sha1", hash.into())))
    } else {
        let builds = json(&client, &format!("https://fill.papermc.io/v3/projects/paper/versions/{}/builds", server.version))?;
        let stable = builds.as_array().ok_or("Respuesta de Paper no válida")?.iter()
            .find(|b| b["channel"].as_str() == Some("STABLE"))
            .ok_or("Paper no tiene una versión estable para esta versión de Minecraft")?;
        let download = &stable["downloads"]["server:default"];
        let url = download["url"].as_str().ok_or("Paper no proporcionó URL")?;
        let hash = download["checksums"]["sha256"].as_str().map(|s| ("sha256",s.into()));
        (url.into(), hash)
    };
    let java = ensure_java(root, major, &client, app, &server.id)?;
    let jar = Path::new(&server.directory).join("server.jar");
    download(&client, &url, &jar, checksum.as_ref().map(|(a,b)| (*a,b.as_str())), app, &server.id, "Servidor Minecraft")?;
    storage::update_install(root, &server.id, &jar, &java)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_non_https_sources() {
        assert!(json(&client().unwrap(), "http://example.com").is_err());
    }
}
