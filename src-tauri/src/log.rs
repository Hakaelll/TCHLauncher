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

fn server_info(line:&str)->bool {
    // Paper's console formatter omits the thread and uses [HH:mm:ss INFO].
    let paper_prefix = line.as_bytes().get(..16).is_some_and(|prefix|
        prefix[0] == b'[' && prefix[3] == b':' && prefix[6] == b':' && &prefix[9..] == b" INFO]:"
        && [1,2,4,5,7,8].iter().all(|&index|prefix[index].is_ascii_digit()));
    let prefix=line.split_once("]: ").map(|(prefix,_)|prefix).unwrap_or("");
    paper_prefix||prefix.ends_with("[Server thread/INFO")||prefix.contains("[Server thread/INFO]")||prefix.ends_with("[ServerMain/INFO")||prefix.contains("[ServerMain/INFO]")
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
    if lower.contains("can't keep up") { return ("warn", Some("lag")); }
    if lower.contains("exception") || lower.contains("[severe]") || lower.contains("[fatal]") {
        return ("error", Some(if lower.contains("plugin") { "plugin_error" } else if lower.contains("mod") { "mod_error" } else { "exception" }));
    }
    if lower.contains("error") {
        return ("error", Some(if lower.contains("plugin") { "plugin_error" } else if lower.contains("mod") { "mod_error" } else { "error" }));
    }
    if lower.contains("warn") { return ("warn", Some("warning")); }
    if message_lower.contains("stopping server") { return ("system", Some("shutdown")); }
    if message_lower.contains("saving chunks") || message_lower.contains("saved the game") { return ("system", Some("save")); }
    if message_lower.starts_with("proceso finalizado:") { return ("system", Some("exit")); }
    ("info", None)
}

pub fn parse(server_id: &str, record: &str) -> LogEvent {
    let (timestamp, line) = match record.split_once('\t') {
        Some((prefix, raw)) if chrono::DateTime::parse_from_rfc3339(prefix).is_ok() => (Some(prefix.to_string()), raw),
        _ => (minecraft_time(record), record),
    };
    let clean=without_terminal_colors(line);
    let (level, event) = classify(&clean);
    LogEvent {
        server_id: server_id.to_string(),
        level: level.to_string(),
        line: line.to_string(),
        message: message_text(&clean).to_string(),
        timestamp:timestamp.or_else(||minecraft_time(&clean)),
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
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if offset > 0 {
        if let Some(first_newline) = bytes.iter().position(|&byte| byte == b'\n') {
            bytes.drain(..=first_newline);
        } else { return Ok(Vec::new()); }
    }
    let content = String::from_utf8_lossy(&bytes);
    let mut records: Vec<_> = content.lines().rev().take(1000).map(|line| parse(server_id, line.trim_end_matches('\r'))).collect();
    records.reverse();
    Ok(records)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_text_cannot_impersonate_error_or_warning_tags(){
        for message in ["<Player> java.lang.OutOfMemoryError: Java heap space","<Player> WARNING: error exception"]{
            let parsed=parse("server",&format!("[12:00:00 INFO]: {message}"));assert_eq!(parsed.level,"chat");assert!(parsed.event.is_none());
        }
        assert_eq!(parse("server","[12:00:00 ERROR]: Plugin exception when player issued server command: /help").level,"error");
        assert_eq!(parse("server","[12:00:00 INFO]: Steve issued server command: /help").level,"command");
        let parsed=parse("server","[12:00:00 INFO]: Exception joined the game");assert_eq!(parsed.level,"player");assert_eq!(parsed.event.as_deref(),Some("join"));
    }
    use uuid::Uuid;

    #[test]
    fn readiness_requires_a_complete_server_startup_message() {
        for message in ["Done (1.2s)!", "Done (0.013s)! For help, type \"help\"", "Done (23.9s)! For help, type \"help\" or \"?\""] {
            let line=format!("[10:00:00] [Server thread/INFO]: {message}");
            assert_eq!(parse("server",&line).event.as_deref(),Some("startup"));
        }
        for message in ["<Alex> Done (1.2s)!", "Plugin task Done (1.2s)!", "Done (NaNs)!", "Done (-1s)!", "Done (1.2s)", "Done (1.2s)! Still preparing"] {
            let line=format!("[10:00:00] [Server thread/INFO]: {message}");
            assert_ne!(parse("server",&line).event.as_deref(),Some("startup"));
        }
        assert_ne!(parse("server","[10:00:00] [Worker/INFO]: Done (1.2s)!").event.as_deref(),Some("startup"));
        assert_eq!(parse("server","[10:00:00 INFO]: Done (1.2s)! For help, type \"help\"").event.as_deref(),Some("startup"));
        assert_ne!(parse("server","[10:00:00 INFO]: <Alex> Done (1.2s)!").event.as_deref(),Some("startup"));
        assert_ne!(parse("server","[10:00:00 WARN]: Done (1.2s)!").event.as_deref(),Some("startup"));
        let colored=parse("server","\u{1b}[32m[10:00:00 INFO]: Done (1.2s)! For help, type \"help\"\u{1b}[0m");
        assert_eq!(colored.event.as_deref(),Some("startup"));
        assert_eq!(colored.timestamp.as_deref(),Some("10:00:00"));
        assert!(!colored.message.contains('\u{1b}'));
        // Exact formatter and launcher timestamp from the reported Paper 1.17.1 startup.
        let imported_paper=parse("server","2026-10-03T00:26:04.484Z\t[02:26:04 INFO]: Done (2.992s)! For help, type \"help\"");
        assert_eq!(imported_paper.event.as_deref(),Some("startup"));
        assert_eq!(imported_paper.timestamp.as_deref(),Some("2026-10-03T00:26:04.484Z"));
    }
    #[test]
    fn classifies_server_events_and_preserves_time() {
        let joined = parse("server", "[21:04:31] [Server thread/INFO]: Alex joined the game");
        assert_eq!(joined.timestamp.as_deref(), Some("21:04:31"));
        assert_eq!(joined.level, "player");
        assert_eq!(joined.event.as_deref(), Some("join"));
        assert_eq!(parse("server", "[21:04:32] [Server thread/WARN]: Can't keep up!").event.as_deref(), Some("lag"));
        assert_eq!(parse("server", "[21:04:33] [Server thread/ERROR]: Plugin example failed").event.as_deref(), Some("plugin_error"));
        assert_eq!(parse("server", "[21:04:34] [Server thread/INFO]: <Alex> Hola").level, "chat");
        assert_eq!(parse("server", "[21:04:35] [Server thread/INFO]: Alex wrote ]: hello").message, "Alex wrote ]: hello");
    }

    #[test]
    fn reads_legacy_and_stamped_records_and_exports_whole_file() {
        let root = std::env::temp_dir().join(format!("tch-log-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("latest.log");
        let destination = root.join("export.txt");
        let mut file = File::create(&source).unwrap();
        writeln!(file, "[10:00:00] [Server thread/INFO]: Starting minecraft server").unwrap();
        let live = append(&mut file, "server", "[10:00:01] [Server thread/INFO]: Done (1.2s)!").unwrap();
        assert!(live.timestamp.as_deref().unwrap().contains('T'));
        drop(file);
        let records = recent(&source, "server").unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].event.as_deref(), Some("startup"));
        assert!(export_snapshot(&source, &source).is_err());
        fs::write(&destination, "old export").unwrap();
        assert_eq!(export_snapshot(&source, &destination).unwrap(), fs::metadata(&source).unwrap().len());
        assert_eq!(fs::read(&source).unwrap(), fs::read(&destination).unwrap());
        let _ = fs::remove_dir_all(root);
    }
}
