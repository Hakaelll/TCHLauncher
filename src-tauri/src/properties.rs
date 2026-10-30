use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<BTreeMap<String,String>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(parse(&raw))
}

fn property_space(c:char)->bool { matches!(c,' '|'\t'|'\u{000c}') }
fn continued(line:&str)->bool { line.bytes().rev().take_while(|c|*c==b'\\').count()%2==1 }

// Keep each logical property's original bytes so unrelated edits preserve its layout.
fn entries(raw:&str)->impl Iterator<Item=(&str,Option<(String,String)>)> {
    let mut offset=0;
    std::iter::from_fn(move || {
        if offset==raw.len(){return None;}
        let start=offset;
        let mut logical=String::new();
        let mut first=true;
        loop {
            let length=raw[offset..].find(['\r','\n']).unwrap_or(raw.len()-offset);
            let end=offset+length;
            let line=raw[offset..end].trim_start_matches(property_space);
            offset=end;
            if raw.as_bytes().get(offset)==Some(&b'\r'){offset+=1;}
            if raw.as_bytes().get(offset)==Some(&b'\n'){offset+=1;}
            if first&&(line.is_empty()||line.starts_with(['#','!'])){
                return Some((&raw[start..offset],None));
            }
            first=false;
            logical.push_str(line);
            let more=continued(&logical);
            if more {logical.pop();}
            if !more||offset==raw.len(){break;}
        }
        let mut escaped=false;
        let split=logical.char_indices().find_map(|(index,c)|{
            if !escaped&&(matches!(c,'='|':')||property_space(c)){return Some(index);}
            escaped=!escaped&&c=='\\';None
        }).unwrap_or(logical.len());
        let key=decode_value(&logical[..split]);
        let mut value=logical[split..].trim_start_matches(property_space);
        if value.starts_with(['=',':']){value=&value[1..];}
        value=value.trim_start_matches(property_space);
        let value=if matches!(key.as_str(),"level-seed"|"level-name"){decode_value(value)}else{value.into()};
        Some((&raw[start..offset],Some((key,value))))
    })
}

pub(crate) fn parse(raw:&str)->BTreeMap<String,String>{
    entries(raw).filter_map(|(_,entry)|entry).collect()
}

pub fn validate_seed(seed:&str)->Result<(),String>{
    if seed.len()>1024||seed.chars().any(char::is_control){return Err("La semilla debe tener hasta 1024 bytes y no incluir saltos de línea ni caracteres de control".into());}Ok(())
}
fn encode_value(seed:&str)->String {
    let mut out=String::new();
    for unit in seed.encode_utf16(){match unit{32=>out.push_str("\\ "),92=>out.push_str("\\\\"),33..=126=>out.push(char::from_u32(unit as u32).unwrap()),_=>out.push_str(&format!("\\u{unit:04x}"))}}
    out
}
fn decode_value(raw:&str)->String {
    let mut chars=raw.chars();let mut units=Vec::new();
    while let Some(c)=chars.next(){
        if c!='\\'{units.extend(c.encode_utf16(&mut [0;2]).iter().copied());continue;}
        match chars.next(){
            Some('u')=>{let digits:String=chars.by_ref().take(4).collect();match u16::from_str_radix(&digits,16){Ok(unit) if digits.len()==4=>units.push(unit),_=>return raw.into()}},
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
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let current = parse(&raw);
    // Preserve unchanged source values, including escapes and duplicate entries.
    updates.retain(|key,value|current.get(key)!=Some(value));
    if updates.is_empty() { return Ok(raw); }
    for (key,value) in &updates {
        if key.is_empty() || key.len() > 80 || !key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c,'-'|'.'|'_')) || value.contains(['\r','\n']) || value.len() > 1024 {
            return Err(format!("Propiedad no válida: {key}"));
        }
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
    let newline=if raw.contains("\r\n") {"\r\n"}else{"\n"};
    let mut found = std::collections::HashSet::new();
    let mut output=String::with_capacity(raw.len());
    for (source,entry) in entries(&raw) {
        if let Some((key,_))=entry {
            if let Some(value)=updates.get(&key) {
                let ending=if source.ends_with("\r\n"){"\r\n"}else if source.ends_with('\n'){"\n"}else if source.ends_with('\r'){"\r"}else{""};
                output.push_str(&format!("{key}={}{ending}",if matches!(key.as_str(),"level-seed"|"level-name"){encode_value(value)}else{value.clone()}));
                if !ending.is_empty()&&!matches!(key.as_str(),"level-seed"|"level-name")&&continued(value){output.push_str(ending);}
                found.insert(key);continue;
            }
        }
        output.push_str(source);
    }
    for (key,value) in updates {
        if !found.contains(&key) {
            if !output.is_empty()&&!output.ends_with(['\r','\n']){
                let more=continued(&output);output.push_str(newline);
                if more {output.push_str(newline);}
            }
            let value=if matches!(key.as_str(),"level-seed"|"level-name"){encode_value(&value)}else{value};
            output.push_str(&format!("{key}={value}{newline}"));
            if continued(&value){output.push_str(newline);}
        }
    }
    Ok(output)
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
