use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<BTreeMap<String,String>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(raw.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { return None; }
        line.split_once('=').map(|(k,v)| (k.trim().into(),v.trim().into()))
    }).collect())
}

pub fn write(path: &Path, updates: BTreeMap<String,String>) -> Result<(), String> {
    for (key,value) in &updates {
        if key.is_empty() || key.len() > 80 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || value.contains(['\r','\n']) || value.len() > 1024 {
            return Err(format!("Propiedad no válida: {key}"));
        }
        match key.as_str() {
            "server-port" => check_range(key,value,1,65535)?,
            "max-players" => check_range(key,value,1,1000)?,
            "view-distance" | "simulation-distance" => check_range(key,value,2,32)?,
            "pvp" | "online-mode" | "white-list" | "hardcore" | "allow-flight" | "force-gamemode" | "enforce-whitelist" => {
                if value != "true" && value != "false" { return Err(format!("{key} debe ser true o false")); }
            }
            "gamemode" => {
                if !["survival","creative","adventure","spectator"].contains(&value.as_str()) { return Err("Modo de juego no válido".into()); }
            }
            "difficulty" => {
                if !["peaceful","easy","normal","hard"].contains(&value.as_str()) { return Err("Dificultad no válida".into()); }
            }
            _ => (),
        }
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut found = std::collections::HashSet::new();
    let mut lines = Vec::new();
    for line in raw.lines() {
        if let Some((key,_)) = line.split_once('=') {
            if let Some(value) = updates.get(key.trim()) {
                lines.push(format!("{}={}",key.trim(),value)); found.insert(key.trim().to_string()); continue;
            }
        }
        lines.push(line.into());
    }
    for (key,value) in updates { if !found.contains(&key) { lines.push(format!("{key}={value}")); } }
    let temp = path.with_extension("properties.part");
    fs::write(&temp, format!("{}\n",lines.join("\n"))).map_err(|e| e.to_string())?;
    let previous = path.with_extension(format!("previous-{}",uuid::Uuid::new_v4()));
    fs::rename(path,&previous).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&temp,path) {
        let _ = fs::rename(&previous,path);
        return Err(e.to_string());
    }
    let _ = fs::remove_file(previous);
    Ok(())
}

fn check_range(key: &str, value: &str, min: u32, max: u32) -> Result<(), String> {
    let number = value.parse::<u32>().map_err(|_| format!("{key} debe ser un número"))?;
    if number < min || number > max { return Err(format!("{key} debe estar entre {min} y {max}")); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_unedited_properties() {
        let path = std::env::temp_dir().join(format!("tch-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"# comment\nmax-players=20\ncustom=value\n").unwrap();
        write(&path,[("max-players".into(),"30".into())].into()).unwrap();
        assert_eq!(read(&path).unwrap()["custom"],"value");
        assert_eq!(read(&path).unwrap()["max-players"],"30");
        let _ = fs::remove_file(path);
    }
    #[test]
    fn rejects_invalid_port_without_changing_file() {
        let path = std::env::temp_dir().join(format!("tch-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"server-port=25565\n").unwrap();
        assert!(write(&path,[("server-port".into(),"70000".into())].into()).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(),"server-port=25565\n");
        let _ = fs::remove_file(path);
    }
}
