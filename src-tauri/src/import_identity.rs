//! Read-only launcher hints. Names alone are never platform or version evidence.
use serde::Serialize;
use std::{fs, io::Read, path::Path};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Launcher {
    pub filename: String,
    pub platform: Option<String>,
    pub version: Option<String>,
    pub note: Option<String>,
}

fn entry(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> Result<Option<String>, String> {
    let count = archive.file_names().filter(|value| *value == name).count();
    if count == 0 {
        return Ok(None);
    }
    if count != 1 {
        return Err("El JAR repite metadatos de arranque".into());
    }
    let item = archive
        .by_name(name)
        .map_err(|_| "No se pueden leer los metadatos del arranque")?;
    if item.size() > 131072 {
        return Err("Los metadatos de arranque superan 128 KB".into());
    }
    let mut text = String::new();
    item.take(131073)
        .read_to_string(&mut text)
        .map_err(|_| "Metadatos de arranque no válidos")?;
    if text.len() > 131072 {
        return Err("Los metadatos de arranque superan 128 KB".into());
    }
    Ok(Some(text))
}

fn unique_field(text: &str, key: &str, separator: &str) -> Result<Option<String>, String> {
    let prefix = format!("{key}{separator}");
    let mut values = text
        .lines()
        .take_while(|line| separator != ": " || !line.is_empty())
        .filter_map(|line| line.strip_prefix(&prefix));
    let first = values.next().map(str::to_owned);
    if values.next().is_some() {
        return Err("Los metadatos de arranque son ambiguos".into());
    }
    Ok(first)
}



fn jar(root: &Path, path: &Path) -> Result<(Option<String>, Option<String>), String> {
    crate::loader_identity::checked_file(root, path, 512 * 1024 * 1024)?;
    let mut archive = zip::ZipArchive::new(fs::File::open(path).map_err(|e| e.to_string())?)
        .map_err(|_| "El archivo no es un JAR legible; confirma sus datos")?;
    if archive.len() > 100_000 {
        return Err("El JAR tiene demasiadas entradas".into());
    }
    let manifest = entry(&mut archive, "META-INF/MANIFEST.MF")?
        .unwrap_or_default()
        .replace("\r\n ", "")
        .replace("\n ", "");
    let main = unique_field(&manifest, "Main-Class", ": ")?;
    let platform = match main.as_deref() {
        Some("io.papermc.paperclip.Paperclip" | "io.papermc.paperclip.Main") => Some("paper"),
        Some(
            "net.fabricmc.installer.ServerLauncher"
            | "net.fabricmc.loader.impl.launch.server.FabricServerLauncher"
            | "net.fabricmc.loader.launch.server.FabricServerLauncher"
            | "net.fabricmc.loader.impl.launch.knot.KnotServer",
        ) => Some("fabric"),
        Some(
            "net.minecraft.bundler.Main"
            | "net.minecraft.server.Main"
            | "net.minecraft.server.MinecraftServer",
        ) => Some("vanilla"),
        _ => None,
    };
    let mut versions = std::collections::HashSet::new();
    if let Some(text) = entry(&mut archive, "version.json")? {
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| "version.json del arranque no es válido")?;
        let id = value["id"]
            .as_str()
            .ok_or("version.json no declara una versión")?;
        crate::fabric::validate_version(id)?;
        versions.insert(id.to_owned());
    }
    if platform == Some("fabric") {
        if let Some(text) = entry(&mut archive, "install.properties")? {
            if let Some(id) = unique_field(&text, "game-version", "=")? {
                crate::fabric::validate_version(&id)?;
                versions.insert(id);
            }
        }
        if let Some(classpath) = unique_field(&manifest, "Class-Path", ": ")? {
            for reference in classpath.split_whitespace() {
                if let Some(rest) = reference.strip_prefix("libraries/net/fabricmc/intermediary/") {
                    if let Some((id, file)) = rest.split_once('/') {
                        if file == format!("intermediary-{id}.jar") {
                            crate::fabric::validate_version(id)?;
                            versions.insert(id.to_owned());
                        }
                    }
                }
            }
        }
    }
    if matches!(platform, Some("paper" | "vanilla")) {
        if let Some(text) = entry(&mut archive, "META-INF/versions.list")? {
            let rows = text
                .lines()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            if rows.len() != 1 {
                return Err("El arranque declara varias versiones de servidor".into());
            }
            let fields = rows[0].split('\t').collect::<Vec<_>>();
            if fields.len() != 3 {
                return Err("La lista de versiones del arranque no es válida".into());
            }
            crate::fabric::validate_version(fields[1])?;
            versions.insert(fields[1].to_owned());
        }
    }
    if versions.len() > 1 {
        return Err("El arranque declara versiones Minecraft contradictorias".into());
    }
    Ok((platform.map(str::to_owned), versions.into_iter().next()))
}

pub fn inspect(root: &Path, filename: &str) -> Launcher {
    let path = root.join(filename);
    let result = if filename.ends_with("win_args.txt") || filename.ends_with("unix_args.txt") {
        crate::loader_identity::checked_file(root, &path, 131072).and_then(|_| {
            let version = crate::neoforge::detected_version(&path)
                .ok_or("No se pudo identificar Minecraft en los argumentos NeoForge")?;
            crate::neoforge::validate_import(root, filename, &version)?;
            Ok((Some("neoforge".into()), Some(version)))
        })
    } else {
        jar(root, &path)
    };
    match result {
        Ok((platform, version)) => {
            let note = if platform.is_none() {
                Some(
                    "Tipo no identificado: confirma el cargador y la versión antes de importar"
                        .into(),
                )
            } else if version.is_none() {
                Some("Versión Minecraft no identificada: introdúcela antes de importar".into())
            } else {
                None
            };
            Launcher {
                filename: filename.into(),
                platform,
                version,
                note,
            }
        }
        Err(note) => Launcher {
            filename: filename.into(),
            platform: None,
            version: None,
            note: Some(note),
        },
    }
}
