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
    if bytes.len() < 10 || bytes[0] != b'[' || bytes[9] != b']' || bytes[3] != b':' || bytes[6] != b':' {
        return None;
    }
    if [1, 2, 4, 5, 7, 8].iter().any(|&index| !bytes[index].is_ascii_digit()) {
        return None;
    }
    Some(line[1..9].to_string())
}

fn startup_complete(line:&str,message:&str)->bool {
    if !line.contains("[Server thread/INFO]") && !line.contains("[ServerMain/INFO]") {return false;}
    let Some(rest)=message.strip_prefix("Done (") else {return false;};
    let Some((duration,tail))=rest.split_once("s)!") else {return false;};
    duration.parse::<f64>().is_ok_and(|seconds|seconds.is_finite() && seconds>=0.0) &&
        (tail.trim().is_empty() || tail.trim().starts_with("For help, type "))
}

fn classify(line: &str) -> (&'static str, Option<&'static str>) {
    let lower = line.to_ascii_lowercase();
    let message = line.split_once("]: ").map(|(_, message)| message).unwrap_or(line);
    let message_lower = message.to_ascii_lowercase();
    if lower.contains("can't keep up") { return ("warn", Some("lag")); }
    if lower.contains("exception") || lower.contains("[severe]") || lower.contains("[fatal]") {
        return ("error", Some(if lower.contains("plugin") { "plugin_error" } else if lower.contains("mod") { "mod_error" } else { "exception" }));
    }
    if lower.contains("error") {
        return ("error", Some(if lower.contains("plugin") { "plugin_error" } else if lower.contains("mod") { "mod_error" } else { "error" }));
    }
    if message.ends_with(" joined the game") { return ("player", Some("join")); }
    if message.ends_with(" left the game") { return ("player", Some("leave")); }
    if message.starts_with('<') && message.contains("> ") { return ("chat", None); }
    if message_lower.contains("issued server command") { return ("command", None); }
    if lower.contains("warn") { return ("warn", Some("warning")); }
    if startup_complete(line,message) { return ("system", Some("startup")); }
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
    let (level, event) = classify(line);
    LogEvent {
        server_id: server_id.to_string(),
        level: level.to_string(),
        line: line.to_string(),
        message: line.split_once("]: ").map(|(_, message)| message).unwrap_or(line).to_string(),
        timestamp,
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
