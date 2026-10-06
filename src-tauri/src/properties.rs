use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<BTreeMap<String,String>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(raw.lines().filter_map(|line| {
        let line = line.trim_start();
        if line.is_empty() || line.starts_with('#') { return None; }
        line.split_once('=').map(|(k,v)| (k.trim().into(),if k.trim()=="level-seed" {decode_seed(v)}else{v.trim().into()}))
    }).collect())
}

pub fn validate_seed(seed:&str)->Result<(),String>{
    if seed.len()>1024||seed.chars().any(char::is_control){return Err("La semilla debe tener hasta 1024 bytes y no incluir saltos de línea ni caracteres de control".into());}Ok(())
}
fn encode_seed(seed:&str)->String {
    let mut out=String::new();
    for unit in seed.encode_utf16(){match unit{32=>out.push_str("\\ "),92=>out.push_str("\\\\"),33..=126=>out.push(char::from_u32(unit as u32).unwrap()),_=>out.push_str(&format!("\\u{unit:04x}"))}}
    out
}
fn decode_seed(raw:&str)->String {
    let mut chars=raw.trim_start().chars();let mut units=Vec::new();
    while let Some(c)=chars.next(){
        if c!='\\'{units.extend(c.encode_utf16(&mut [0;2]).iter().copied());continue;}
        match chars.next(){
            Some('u')=>{let digits:String=chars.by_ref().take(4).collect();match u16::from_str_radix(&digits,16){Ok(unit)=>units.push(unit),Err(_)=>return raw.into()}},
            Some(c)=>{let decoded=match c{'t'=>'\t','n'=>'\n','r'=>'\r','f'=>'\u{000c}',_=>c};units.extend(decoded.encode_utf16(&mut [0;2]).iter().copied());},
            None=>return raw.into(),
        }
    }
    String::from_utf16(&units).unwrap_or_else(|_|raw.into())
}

/// Editor defaults fill absent controls without modifying imported files on read.
pub fn editor_defaults(version:&str)->BTreeMap<String,String>{
    let minor=version.strip_prefix("1.").and_then(|v|v.split('.').next()).and_then(|v|v.parse::<u32>().ok());
    let legacy=minor.is_some_and(|minor|minor<13);
    let mut values:BTreeMap<String,String>=[("gamemode",if legacy{"0"}else{"survival"}),("difficulty",if legacy{"1"}else{"easy"}),("pvp","true"),("force-gamemode","false"),("hardcore","false"),("allow-flight","false"),("enable-command-block","false"),("spawn-protection","16"),("view-distance","10"),("player-idle-timeout","0"),("level-seed",""),("generate-structures","true"),("white-list","false"),("enforce-whitelist","false")].into_iter().map(|(k,v)|(k.into(),v.into())).collect();
    if minor.is_none_or(|minor|minor>=18){values.insert("simulation-distance".into(),"10".into());}
    if minor.is_some_and(|minor|minor<8){for key in ["force-gamemode","enforce-whitelist"]{values.remove(key);}}
    values
}
pub fn read_for_editor(path:&Path,version:&str)->Result<BTreeMap<String,String>,String>{
    let mut values=editor_defaults(version);values.extend(read(path)?);Ok(values)
}

pub(crate) fn prepare(path: &Path, mut updates: BTreeMap<String,String>) -> Result<String, String> {
    let current = read(path)?;
    // Preserve original imported seed escapes byte-for-byte on unrelated edits.
    // A decoded unchanged seed may legitimately contain escaped control characters.
    if current.get("level-seed").is_some_and(|seed|updates.get("level-seed")==Some(seed)) {
        updates.remove("level-seed");
    }
    for (key,value) in &updates {
        if key.is_empty() || key.len() > 80 || !key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c,'-'|'.'|'_')) || value.contains(['\r','\n']) || value.len() > 1024 {
            return Err(format!("Propiedad no válida: {key}"));
        }
        // Imported legacy values must not block an unrelated edit. Validate values
        // the user changes, while preserving all unchanged server-owned settings.
        if current.get(key) == Some(value) { continue; }
        match key.as_str() {
            "level-seed" => validate_seed(value)?,
            "server-port" | "query.port" | "rcon.port" => check_range(key,value,1,65535)?,
            "max-players" => check_range(key,value,1,1000)?,
            "view-distance" | "simulation-distance" => check_range(key,value,2,32)?,
            "max-world-size" => check_range(key,value,1,29_999_984)?,
            "spawn-protection" | "player-idle-timeout" => check_range(key,value,0,i32::MAX as u32)?,
            "pvp" | "online-mode" | "white-list" | "hardcore" | "allow-flight" | "force-gamemode" | "enforce-whitelist" | "generate-structures" | "enable-command-block" | "allow-nether" | "spawn-animals" | "spawn-monsters" | "spawn-npcs" => {
                if value != "true" && value != "false" { return Err(format!("{key} debe ser true o false")); }
            }
            "gamemode" => {
                if !["survival","creative","adventure","spectator","0","1","2","3"].contains(&value.as_str()) { return Err("Modo de juego no válido".into()); }
            }
            "difficulty" => {
                if !["peaceful","easy","normal","hard","0","1","2","3"].contains(&value.as_str()) { return Err("Dificultad no válida".into()); }
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
                lines.push(format!("{}={}",key.trim(),if key.trim()=="level-seed"{encode_seed(value)}else{value.clone()})); found.insert(key.trim().to_string()); continue;
            }
        }
        lines.push(line.into());
    }
    for (key,value) in updates { if !found.contains(&key) { lines.push(format!("{key}={}",if key=="level-seed"{encode_seed(&value)}else{value})); } }
    Ok(format!("{}\n",lines.join("\n")))
}

pub fn write(path: &Path, updates: BTreeMap<String,String>) -> Result<(), String> {
    let prepared=prepare(path,updates)?;
    let temp = path.with_extension("properties.part");
    fs::write(&temp, prepared).map_err(|e| e.to_string())?;
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
    fn unchanged_imported_seed_escapes_do_not_block_or_rewrite_an_unrelated_edit(){
        let path=std::env::temp_dir().join(format!("tch-imported-seed-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"level-seed=old\\u0020seed\\ntext\npvp=true\n").unwrap();
        let mut updates=read(&path).unwrap();assert_eq!(updates["level-seed"],"old seed\ntext");updates.insert("pvp".into(),"false".into());
        write(&path,updates).unwrap();assert!(fs::read_to_string(&path).unwrap().starts_with("level-seed=old\\u0020seed\\ntext\n"));assert_eq!(read(&path).unwrap()["pvp"],"false");
        fs::remove_file(path).unwrap();
    }
    #[test]
    #[ignore="Requires TCH_TEST_JAVA and TCH_TEST_PROPERTIES_CLASSES compiled from SeedPropertiesProbe.java; never runs Minecraft"]
    fn real_java_properties_reads_exact_seed_strings(){
        use base64::Engine;
        let java=std::env::var("TCH_TEST_JAVA").expect("Set TCH_TEST_JAVA");
        let classes=std::env::var("TCH_TEST_PROPERTIES_CLASSES").expect("Compile the checked-in Java probe and set its classes directory");
        let root=std::env::temp_dir().join(format!("tch-java-seed-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let path=root.join("server.properties");fs::write(&path,"online-mode=true\n").unwrap();
        for seed in ["-9223372036854775808","9223372036854775807","Mundo ñ 🌋"," leading and trailing ",r"path\seed",r"literal\u0041","#!:=seed",""]{
            write(&path,[("level-seed".into(),seed.into())].into()).unwrap();
            let mut command=std::process::Command::new(&java);
            #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
            let output=command.args(["-Xms16m","-Xmx64m","-cp",&classes,"SeedPropertiesProbe"]).arg(&path).current_dir(&root).stdin(std::process::Stdio::null()).output().unwrap();
            assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
            let encoded=String::from_utf8(output.stdout).unwrap();let actual=base64::engine::general_purpose::STANDARD.decode(encoded.trim()).unwrap();
            assert_eq!(actual,seed.as_bytes());assert_eq!(read(&path).unwrap()["level-seed"],seed);
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn seed_text_is_lossless_escaped_and_cannot_inject_properties(){
        let path=std::env::temp_dir().join(format!("tch-seed-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"server-port=25565\nonline-mode=true\n").unwrap();
        for seed in ["-9223372036854775808","9223372036854775807","Mundo ñ 🌋"," leading and trailing ",r"path\seed",""]{
            write(&path,[("level-seed".into(),seed.into())].into()).unwrap();assert_eq!(read(&path).unwrap()["level-seed"],seed);
            assert_eq!(read(&path).unwrap()["online-mode"],"true");
        }
        assert_eq!(encode_seed("ñ🌋"),r"\u00f1\ud83c\udf0b");
        assert_eq!(encode_seed(r"a\b"),r"a\\b");
        let before=fs::read(&path).unwrap();
        for seed in ["a\nonline-mode=false".into(),"\t".into(),"x".repeat(1025)]{
            assert!(write(&path,[("level-seed".into(),seed)].into()).is_err());assert_eq!(fs::read(&path).unwrap(),before);
        }
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn editor_exposes_preboot_rules_preserves_real_values_and_never_writes_on_read(){
        let path=std::env::temp_dir().join(format!("tch-editor-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"# Before first boot\nserver-port=25565\ndifficulty=hard\ncustom=value\n").unwrap();let before=fs::read(&path).unwrap();
        let values=read_for_editor(&path,"1.21.11").unwrap();assert_eq!(values["gamemode"],"survival");assert_eq!(values["difficulty"],"hard");assert_eq!(values["simulation-distance"],"10");assert_eq!(values["custom"],"value");
        assert_eq!(fs::read(&path).unwrap(),before);
        let mut updates=values;updates.insert("gamemode".into(),"creative".into());write(&path,updates).unwrap();assert_eq!(read(&path).unwrap()["gamemode"],"creative");
        let old=editor_defaults("1.16.5");assert_eq!(old["gamemode"],"survival");assert!(!old.contains_key("simulation-distance"));assert_eq!(editor_defaults("1.8.8")["difficulty"],"1");
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn rules_and_generation_edits_preserve_legacy_values_and_reject_invalid_changes() {
        let path=std::env::temp_dir().join(format!("tch-rules-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"# Imported Minecraft properties\ngamemode=0\nview-distance=48\ncustom=value\nlevel-seed=old seed\nlevel-type=minecraft\\:normal\nspawn-protection=16\nplayer-idle-timeout=0\nmax-world-size=29999984\ngenerate-structures=true\nenable-command-block=false\nallow-nether=true\nspawn-animals=true\nspawn-monsters=true\nspawn-npcs=true\n").unwrap();
        let mut update=read(&path).unwrap();update.insert("level-seed".into(),"a new seed".into());update.insert("spawn-protection".into(),"0".into());update.insert("player-idle-timeout".into(),"15".into());update.insert("generate-structures".into(),"false".into());update.insert("enable-command-block".into(),"true".into());write(&path,update).unwrap();
        let saved=read(&path).unwrap();assert_eq!(saved["gamemode"],"0");assert_eq!(saved["view-distance"],"48");assert_eq!(saved["level-type"],"minecraft\\:normal");assert_eq!(saved["custom"],"value");assert_eq!(saved["level-seed"],"a new seed");assert_eq!(saved["spawn-protection"],"0");assert_eq!(saved["player-idle-timeout"],"15");assert_eq!(saved["generate-structures"],"false");assert_eq!(saved["enable-command-block"],"true");
        let before=fs::read(&path).unwrap();
        for (key,value) in [("view-distance","49"),("gamemode","4"),("max-world-size","30000000"),("max-world-size","0"),("spawn-protection","-1"),("player-idle-timeout","1.5"),("player-idle-timeout","2147483648"),("generate-structures","yes"),("enable-command-block","1"),("allow-nether",""),("spawn-animals","TRUE"),("spawn-monsters","yes"),("spawn-npcs","0"),("query.port","70000"),("rcon.port","0")] {
            let mut update=saved.clone();update.insert(key.into(),value.into());assert!(write(&path,update).is_err(),"{key}");assert_eq!(fs::read(&path).unwrap(),before);
        }
        assert!(fs::read_to_string(&path).unwrap().starts_with("# Imported Minecraft properties\n"));fs::remove_file(path).unwrap();
    }
    #[test]
    fn saving_all_minecraft_properties_accepts_dotted_names_and_rejects_injection(){
        let path=std::env::temp_dir().join(format!("tch-dotted-props-{}",uuid::Uuid::new_v4()));
        fs::write(&path,"# Minecraft properties\nserver-port=25565\nenable-query=false\nquery.port=25565\nenable-rcon=false\nrcon.port=25575\nrcon.password=\npvp=true\nonline-mode=true\ncustom_setting=value\n").unwrap();
        let mut updates=read(&path).unwrap();updates.insert("pvp".into(),"false".into());write(&path,updates).unwrap();
        let saved=read(&path).unwrap();assert_eq!(saved["pvp"],"false");assert_eq!(saved["query.port"],"25565");assert_eq!(saved["rcon.port"],"25575");assert_eq!(saved["rcon.password"],"");assert_eq!(saved["online-mode"],"true");assert_eq!(saved["enable-query"],"false");assert!(fs::read_to_string(&path).unwrap().starts_with("# Minecraft properties\n"));
        let before=fs::read(&path).unwrap();
        for(key,value)in [("query.port\nserver-port","1"),("query.port=server-port","1"),("query.port","25565\nonline-mode=false"),("../other","value"),("query port","25565")]{assert!(write(&path,[(key.into(),value.into())].into()).is_err());assert_eq!(fs::read(&path).unwrap(),before);}
        fs::remove_file(path).unwrap();
    }
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
