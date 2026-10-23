use crate::model::LogEvent;
use chrono::SecondsFormat;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub fn stamp(line: &str) -> String {
    format!("{}\t{line}", chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
}

fn minecraft_time(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    if bytes.len() < 10 || bytes[0] != b'[' || !matches!(bytes[9], b']' | b' ') || bytes[3] != b':' || bytes[6] != b':' {
        return None;
    }
    if [1, 2, 4, 5, 7, 8].iter().any(|&index| !bytes[index].is_ascii_digit()) {
        return None;
    }
    Some(line[1..9].to_string())
}

pub(crate) fn without_terminal_colors(line:&str)->String {
    let mut clean=String::with_capacity(line.len());
    let mut chars=line.chars().peekable();
    while let Some(character)=chars.next() {
        if character=='\u{1b}' && chars.peek()==Some(&'[') {
            chars.next();
            for code in chars.by_ref() { if ('@'..='~').contains(&code) {break;} }
        } else {clean.push(character);}
    }
    clean
}

// JLine may repeat its cleared prompt after commands. Only strip it before a header.
pub(crate) fn without_terminal_prompt(line:&str)->&str {
    let mut rest=line;
    while let Some(tail)=rest.strip_prefix("> ") {rest=tail.trim_start();}
    if rest.starts_with('[') {rest} else {line}
}

fn server_info(line:&str)->bool {
    // Paper's console formatter omits the thread and uses [HH:mm:ss INFO].
    let paper_prefix = line.as_bytes().get(..16).is_some_and(|prefix|
        prefix[0] == b'[' && prefix[3] == b':' && prefix[6] == b':' && &prefix[9..] == b" INFO]:"
        && [1,2,4,5,7,8].iter().all(|&index|prefix[index].is_ascii_digit()));
    let prefix=line.split_once("]: ").map(|(prefix,_)|prefix).unwrap_or("");
    paper_prefix||prefix.ends_with("[Server thread/INFO")||prefix.contains("[Server thread/INFO]")||prefix.ends_with("[ServerMain/INFO")||prefix.contains("[ServerMain/INFO]")
}

pub(crate) fn header_level(line:&str)->Option<&str> {
    for (prefix,level) in [("ERROR:","ERROR"),("Error:","ERROR"),("WARN:","WARN"),("WARNING:","WARN")] {if line.starts_with(prefix){return Some(level);}}
    if !line.starts_with('[') {
        let mut words=line.split_whitespace();
        let date=words.next()?;let time=words.next()?;let thread=words.next()?;let level=words.next()?;
        if date.len()==10&&chrono::NaiveDate::parse_from_str(date,"%Y-%m-%d").is_ok()&&time.len()==12&&time.as_bytes()[8]==b','&&time.as_bytes()[9..].iter().all(u8::is_ascii_digit)&&chrono::NaiveTime::parse_from_str(&time[..8],"%H:%M:%S").is_ok()&&thread.len()<=128&&matches!(level,"TRACE"|"DEBUG"|"INFO"|"WARN"|"ERROR"|"FATAL") {return Some(level);}
        return None;
    }
    let rest=line.strip_prefix('[')?;
    let (first,tail)=rest.split_once(']')?;
    let recognized=|value:&str|matches!(value,"TRACE"|"DEBUG"|"INFO"|"WARN"|"WARNING"|"ERROR"|"SEVERE"|"FATAL");
    if minecraft_time(line).is_some() {
        if let Some((_,level))=first.rsplit_once(' ') {if recognized(level){return Some(level);}}
        let rest=tail.trim_start().strip_prefix('[')?;
        let (thread,_)=rest.split_once(']')?;
        let (_,level)=thread.rsplit_once('/')?;
        return recognized(level).then_some(level);
    }
    // Some launcher/loader formatters omit the clock but retain [thread/LEVEL].
    first.rsplit_once('/').and_then(|(_,level)|recognized(level).then_some(level))
}

fn throwable(message:&str)->bool {
    if message.starts_with("Exception in thread \""){return true;}
    let text=message.trim_start().strip_prefix("Caused by: ").unwrap_or(message.trim_start());
    let name=text.split([':', ' ']).next().unwrap_or("");
    name.contains('.')&&(name.ends_with("Exception")||name.ends_with("Error"))&&name.bytes().all(|byte|byte.is_ascii_alphanumeric()||b".$_".contains(&byte))
}

fn startup_complete(line:&str,message:&str)->bool {
    if !server_info(line){return false;}
    let Some(rest)=message.strip_prefix("Done (") else {return false;};
    let Some((duration,tail))=rest.split_once("s)!") else {return false;};
    duration.parse::<f64>().is_ok_and(|seconds|seconds.is_finite() && seconds>=0.0) &&
        (tail.trim().is_empty() || tail.trim().starts_with("For help, type "))
}

fn message_text(line:&str)->&str{
    // Remove only a leading console header. A delimiter written inside chat
    // must remain player text, including chat without a console timestamp.
    if line.starts_with('['){
        if let Some((prefix,message))=line.split_once("]: "){
            if !prefix.contains('<'){return message;}
        }
    }
    line
}

fn classify(line: &str) -> (&'static str, Option<&'static str>) {
    let lower = line.to_ascii_lowercase();
    let message = message_text(line);
    let message_lower = message.to_ascii_lowercase();
    let authenticator=line.split_once("]: ").and_then(|(prefix,_)|prefix.rsplit('[').next())
        .and_then(|thread|thread.strip_prefix("User Authenticator #"))
        .and_then(|thread|thread.strip_suffix("/INFO"))
        .is_some_and(|number|!number.is_empty()&&number.bytes().all(|byte|byte.is_ascii_digit()));
    if (server_info(line)||authenticator)&&crate::players::message_profile(message).is_some() {return ("player",Some("identity"));}
    if let Some((_,joined))=crate::players::message_player_change(message){return ("player",Some(if joined{"join"}else{"leave"}));}
    let chat=message.strip_prefix("[Not Secure] ").unwrap_or(message);
    if chat.starts_with('<') && chat.contains("> ") { return ("chat", None); }
    if message.split_once(" issued server command: ").is_some_and(|(name,_)|!name.is_empty()&&name.len()<=16&&name.bytes().all(|byte|byte.is_ascii_alphanumeric()||byte==b'_')) { return ("command", None); }
    if startup_complete(line,message) { return ("system", Some("startup")); }
    if server_info(line){
        if message.starts_with("Preparing level \"")&&message.ends_with('"'){return ("system",Some("world_load"));}
        let region=message.strip_prefix("Preparing start region for dimension ").is_some_and(|dimension|!dimension.is_empty()&&dimension.bytes().all(|byte|byte.is_ascii_alphanumeric()||b"_:/.-".contains(&byte)));
        let area=message.strip_prefix("Preparing spawn area: ").and_then(|rest|rest.strip_suffix('%')).and_then(|value|value.parse::<u8>().ok()).is_some_and(|value|value<=100);
        if region||area{return ("system",Some("spawn_prepare"));}
    }
    let level=header_level(line);
    if server_info(line){
        if message=="Stopping server"||message=="Stopping the server" {return ("system",Some("shutdown"));}
        if message=="Saving the game (this may take a moment!)"||message=="Saving worlds"||message.starts_with("Saving chunks for level '") {return ("system",Some("save_started"));}
        if message=="Saved the game"||message=="ThreadedAnvilChunkStorage: All dimensions are saved" {return ("system",Some("save"));}
        if message=="Failed to save the game" {return ("error",Some("save_failed"));}
        if message.starts_with("ThreadedAnvilChunkStorage (")&&message.ends_with("): All chunks are saved") {return ("system",Some("save_dimension"));}
    }
    if matches!(level,Some("WARN"|"WARNING"))&&message.starts_with("Can't keep up!") {return ("warn",Some("lag"));}
    if matches!(level,Some("ERROR"|"SEVERE"|"FATAL"))&&["Failed to save the game","Failed to save level ","Failed to save chunk ","Couldn't save chunk ","Error saving chunk "].iter().any(|prefix|message.starts_with(prefix)) {return ("error",Some("save_failed"));}
    if matches!(level,Some("ERROR"|"SEVERE"|"FATAL"))||level.is_none()&&throwable(message) {
        return ("error", Some(if lower.contains("plugin") { "plugin_error" } else if lower.contains("mod") { "mod_error" } else if throwable(message)||message.contains("Exception") {"exception"}else {"error"}));
    }
    if matches!(level,Some("WARN"|"WARNING"))||level.is_none()&&(message.starts_with("WARNING:")||message.starts_with("Warning:")||message.starts_with("*** Warning,")) {return ("warn",Some("warning"));}
    if message_lower.starts_with("proceso finalizado:") { return ("system", Some("exit")); }
    ("info", None)
}

pub fn parse(server_id: &str, record: &str) -> LogEvent {
    let (timestamp, line) = match record.split_once('\t') {
        Some((prefix, raw)) if chrono::DateTime::parse_from_rfc3339(prefix).is_ok() => (Some(prefix.to_string()), raw),
        _ => (minecraft_time(record), record),
    };
    let clean=without_terminal_colors(line);
    let clean=without_terminal_prompt(&clean);
    let (level, event) = classify(clean);
    LogEvent {
        server_id: server_id.to_string(),
        level: level.to_string(),
        line: line.to_string(),
        message: message_text(clean).to_string(),
        timestamp:timestamp.or_else(||minecraft_time(clean)),
        event: event.map(str::to_string),
    }
}

pub fn append(file: &mut File, server_id: &str, line: &str) -> Result<LogEvent, String> {
    let record = stamp(line);
    writeln!(file, "{record}").map_err(|e| format!("No se pudo guardar el log: {e}"))?;
    Ok(parse(server_id, &record))
}

pub fn recent(path: &Path, server_id: &str) -> Result<Vec<LogEvent>, String> {
    if !path.is_file() { return Ok(Vec::new()); }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    let offset = length.saturating_sub(512_000);
    file.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
    // Freeze the observed extent. A busy server appending output cannot grow this read.
    let mut bytes = recent_bytes(file,length-offset)?;
    if offset > 0 {
        if let Some(first_newline) = bytes.iter().position(|&byte| byte == b'\n') {
            bytes.drain(..=first_newline);
        } else { return Ok(Vec::new()); }
    }
    // A record still being written is not evidence of a completed operation.
    if bytes.last()!=Some(&b'\n') {
        if let Some(last_newline)=bytes.iter().rposition(|&byte|byte==b'\n') {bytes.truncate(last_newline+1);}else{return Ok(Vec::new());}
    }
    let content = String::from_utf8_lossy(&bytes);
    let mut records: Vec<_> = content.lines().rev().take(1000).map(|line| parse(server_id, line.trim_end_matches('\r'))).collect();
    records.reverse();
    Ok(records)
}

fn recent_bytes(reader:impl Read,length:u64)->Result<Vec<u8>,String> {
    let mut bytes=Vec::new();
    reader.take(length.min(512_000)).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    Ok(bytes)
}

pub fn export_snapshot(source: &Path, destination: &Path) -> Result<u64, String> {
    let input = File::open(source).map_err(|e| format!("No se pudo leer el log: {e}"))?;
    if destination.exists() && source.canonicalize().ok() == destination.canonicalize().ok() {
        return Err("Elige un archivo diferente del log original".into());
    }
    let length = input.metadata().map_err(|e| e.to_string())?.len();
    let temporary = destination.with_extension(format!("tchlauncher-{}.part", uuid::Uuid::new_v4()));
    let result = (|| -> Result<u64, String> {
        let mut output = File::create(&temporary).map_err(|e| format!("No se pudo crear el archivo: {e}"))?;
        let copied = std::io::copy(&mut input.take(length), &mut output).map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        if copied != length { return Err("No se pudo exportar el log completo".into()); }
        drop(output);
        if destination.exists() {
            let previous = destination.with_extension(format!("tchlauncher-{}.previous", uuid::Uuid::new_v4()));
            fs::rename(destination, &previous).map_err(|e| e.to_string())?;
            if let Err(error) = fs::rename(&temporary, destination) {
                let _ = fs::rename(&previous, destination);
                return Err(error.to_string());
            }
            let _ = fs::remove_file(previous);
        } else {
            fs::rename(&temporary, destination).map_err(|e| e.to_string())?;
        }
        Ok(copied)
    })();
    if result.is_err() { let _ = fs::remove_file(temporary); }
    result
}
