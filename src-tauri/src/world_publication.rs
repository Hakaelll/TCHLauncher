use crate::{model::Server, storage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, io::{Read, Write}, path::{Path, PathBuf}, sync::Mutex};
use uuid::Uuid;

static PUBLICATION: Mutex<()> = Mutex::new(());
const PREFIX: &str = "world-copy:";

pub(crate) struct Part { pub stage: PathBuf, pub target: PathBuf }

#[derive(Debug)]
pub(crate) enum Failure { Unprepared(String), Pending(String) }

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CopyPart { stage: String, target: String, digest: String }

#[derive(Serialize, Deserialize, PartialEq)]
enum Phase { Prepared, Committed }

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    version: u8,
    token: String,
    server_id: String,
    parent: PathBuf,
    parts: Vec<CopyPart>,
    reserved: Vec<String>,
    phase: Phase,
}

fn plain(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() { return false; }
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return false; }
    }
    true
}

fn metadata(path: &Path) -> Result<Option<fs::Metadata>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) => Ok(Some(meta)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn checked_parent(parent: &Path) -> Result<(), String> {
    if !parent.is_absolute() { return Err("La publicación del mundo requiere una carpeta absoluta".into()); }
    for ancestor in parent.ancestors().filter(|path| path.file_name().is_some()) {
        let meta = fs::symlink_metadata(ancestor).map_err(|e| e.to_string())?;
        if !meta.is_dir() || !plain(&meta) { return Err("La carpeta del mundo contiene enlaces o redirecciones".into()); }
    }
    if parent.canonicalize().map_err(|e| e.to_string())? != parent {
        return Err("La carpeta de publicación del mundo cambió".into());
    }
    Ok(())
}

fn leaf(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    !name.is_empty() && name.chars().count() <= 72 && !name.starts_with(['.', ' '])
        && !name.ends_with(['.', ' '])
        && name.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.' | '(' | ')'))
        && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn validate(key: &str, plan: &Plan) -> Result<(), String> {
    let invalid = "Registro de publicación de mundo no válido";
    if plan.version != 1 || Uuid::parse_str(&plan.token).is_err() || key != format!("{PREFIX}{}", plan.token)
        || Uuid::parse_str(&plan.server_id).is_err() || !plan.parent.is_absolute()
        || plan.parts.is_empty() || plan.parts.len() > 3 || !matches!(plan.reserved.len(), 1 | 3) {
        return Err(invalid.into());
    }
    let base = &plan.reserved[0];
    if !leaf(base) || base.chars().count() > 64 || (plan.reserved.len() == 3
        && plan.reserved != [base.clone(), format!("{base}_nether"), format!("{base}_the_end")]) {
        return Err(invalid.into());
    }
    let mut stages = HashSet::new();
    let mut targets = HashSet::new();
    for part in &plan.parts {
        let stage_uuid = part.stage.strip_prefix(".tch-world-").and_then(|name| name.strip_suffix(".part"));
        if stage_uuid.is_none_or(|id| Uuid::parse_str(id).is_err()) || !stages.insert(part.stage.to_lowercase())
            || !leaf(&part.target) || !plan.reserved.contains(&part.target) || !targets.insert(part.target.to_lowercase())
            || part.digest.len() != 64 || !part.digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid.into());
        }
    }
    if plan.parts[0].target != *base { return Err(invalid.into()); }
    Ok(())
}

fn marker(plan: &Plan) -> String { format!(".tch-world-owner-{}", plan.token) }

fn checked_marker(folder: &Path, plan: &Plan, required: bool) -> Result<(), String> {
    let path = folder.join(marker(plan));
    let Some(meta) = metadata(&path)? else {
        return if required { Err("No se puede confirmar la propiedad de la copia del mundo".into()) } else { Ok(()) };
    };
    if !meta.is_file() || !plain(&meta) || meta.len() != plan.token.len() as u64
        || fs::read_to_string(path).map_err(|e| e.to_string())? != plan.token {
        return Err("El identificador de la copia del mundo cambió".into());
    }
    Ok(())
}

fn fingerprint(folder: &Path, own_marker: &str) -> Result<String, String> {
    let mut hash = Sha256::new();
    for entry in walkdir::WalkDir::new(folder).follow_links(false).sort_by_file_name() {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path.strip_prefix(folder).map_err(|e| e.to_string())?;
        if relative == Path::new(own_marker) { continue; }
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !plain(&meta) || (!meta.is_dir() && !meta.is_file()) {
            return Err("La copia del mundo contiene enlaces o archivos especiales".into());
        }
        let name = relative.to_str().ok_or("El mundo contiene una ruta no válida")?;
        hash.update([if meta.is_dir() { b'd' } else { b'f' }]);
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        if meta.is_file() {
            hash.update(meta.len().to_le_bytes());
            let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = [0u8; 65536];
            loop {
                let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
                if read == 0 { break; }
                hash.update(&buffer[..read]);
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn raw_records(root: &Path) -> Result<Vec<(String, String)>, String> {
    let db = storage::connect(root)?;
    let mut query = db.prepare("SELECT key,value FROM application_settings WHERE key LIKE 'world-copy:%'").map_err(|e| e.to_string())?;
    let rows = query.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    Ok(rows)
}

fn parse_record(key: String, json: String) -> Result<(String, Plan), String> {
    if json.len() > 32768 { return Err("Registro de publicación de mundo demasiado grande".into()); }
    let plan: Plan = serde_json::from_str(&json).map_err(|_| "Registro de publicación de mundo no válido")?;
    validate(&key, &plan)?;
    Ok((key, plan))
}

fn records(root: &Path) -> Result<Vec<(String, Plan)>, String> {
    raw_records(root)?.into_iter().map(|(key, json)| parse_record(key, json)).collect()
}

fn save(root: &Path, plan: &Plan) -> Result<(), String> {
    let db = storage::connect(root)?;
    db.execute_batch("PRAGMA synchronous=FULL;").map_err(|e| e.to_string())?;
    db.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        rusqlite::params![format!("{PREFIX}{}", plan.token), serde_json::to_string(plan).map_err(|e| e.to_string())?])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn recover(root: &Path, key: &str, mut plan: Plan, hook: &mut impl FnMut(usize)) -> Result<(), String> {
    validate(key, &plan)?;
    checked_parent(&plan.parent)?;
    if plan.phase == Phase::Prepared {
        let mut locations = Vec::new();
        for part in &plan.parts {
            let stage = plan.parent.join(&part.stage);
            let target = plan.parent.join(&part.target);
            let at_stage = metadata(&stage)?.is_some();
            let at_target = metadata(&target)?.is_some();
            if at_stage == at_target { return Err("La copia del mundo está incompleta o su destino está ocupado; se conservan los archivos".into()); }
            let location = if at_stage { stage } else { target };
            let meta = fs::symlink_metadata(&location).map_err(|e| e.to_string())?;
            if !meta.is_dir() || !plain(&meta) { return Err("La copia del mundo contiene una redirección".into()); }
            checked_marker(&location, &plan, true)?;
            if fingerprint(&location, &marker(&plan))? != part.digest {
                return Err("La copia del mundo cambió; se conservan todos sus archivos para revisarla".into());
            }
            locations.push(at_stage);
        }
        for name in &plan.reserved {
            if !plan.parts.iter().any(|part| part.target == *name) && metadata(&plan.parent.join(name))?.is_some() {
                return Err("Otra carpeta ocupa una dimensión reservada del mundo; se conservan los archivos".into());
            }
        }
        for (index, part) in plan.parts.iter().enumerate() {
            if locations[index] {
                let target = plan.parent.join(&part.target);
                // Recheck immediately before rename: Windows never replaces an existing directory.
                if metadata(&target)?.is_some() { return Err("El destino de la copia del mundo está ocupado".into()); }
                fs::rename(plan.parent.join(&part.stage), target).map_err(|e| e.to_string())?;
                hook(index + 1);
            }
        }
        plan.phase = Phase::Committed;
        save(root, &plan)?;
        hook(4);
    }
    for (index, part) in plan.parts.iter().enumerate() {
        let target = plan.parent.join(&part.target);
        let meta = fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
        if !meta.is_dir() || !plain(&meta) { return Err("No se encontró la copia confirmada del mundo".into()); }
        checked_marker(&target, &plan, false)?;
        if metadata(&target.join(marker(&plan)))?.is_some() {
            fs::remove_file(target.join(marker(&plan))).map_err(|e| e.to_string())?;
        }
        hook(5 + index);
    }
    storage::connect(root)?.execute("DELETE FROM application_settings WHERE key=?1", [key]).map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn publish(root: &Path, server_id: &str, parent: &Path, parts: &[Part], reserved: &[String]) -> Result<(), Failure> {
    publish_with(root, server_id, parent, parts, reserved, &mut |_| {})
}

fn publish_with(root: &Path, server_id: &str, parent: &Path, parts: &[Part], reserved: &[String], hook: &mut impl FnMut(usize)) -> Result<(), Failure> {
    let _guard = PUBLICATION.lock().unwrap_or_else(|error| error.into_inner());
    let plan = prepare(root, server_id, parent, parts, reserved).map_err(Failure::Unprepared)?;
    let key = format!("{PREFIX}{}", plan.token);
    hook(0);
    recover(root, &key, plan, hook).map_err(|error| Failure::Pending(format!("La copia del mundo queda pendiente de recuperación; se conservan sus archivos. Reinicia la aplicación para reintentar la publicación. Si vuelve a fallar, revisa el destino: {error}")))
}

fn prepare(root: &Path, server_id: &str, parent: &Path, parts: &[Part], reserved: &[String]) -> Result<Plan, String> {
    checked_parent(parent)?;
    let token = Uuid::new_v4().to_string();
    let mut plan = Plan { version: 1, token, server_id: server_id.into(), parent: parent.into(), parts: Vec::new(), reserved: reserved.into(), phase: Phase::Prepared };
    for part in parts {
        if part.stage.parent() != Some(parent) || part.target.parent() != Some(parent) { return Err("La copia del mundo sale de la carpeta elegida".into()); }
        plan.parts.push(CopyPart {
            stage: part.stage.file_name().and_then(|name| name.to_str()).ok_or("Carpeta temporal no válida")?.into(),
            target: part.target.file_name().and_then(|name| name.to_str()).ok_or("Destino de mundo no válido")?.into(),
            digest: "0".repeat(64),
        });
    }
    let key = format!("{PREFIX}{}", plan.token);
    validate(&key, &plan)?;
    for (_, existing) in records(root)? {
        if existing.parent == plan.parent && existing.reserved.iter().any(|name| plan.reserved.iter().any(|new| new.eq_ignore_ascii_case(name))) {
            return Err("Hay una copia de este mundo pendiente de recuperación".into());
        }
    }
    for name in reserved {
        if metadata(&parent.join(name))?.is_some() { return Err("El destino de la copia del mundo está ocupado".into()); }
    }
    let own_marker = marker(&plan);
    let prepared = (|| {
        for part in &mut plan.parts {
            let folder = parent.join(&part.stage);
            let meta = fs::symlink_metadata(&folder).map_err(|e| e.to_string())?;
            if !meta.is_dir() || !plain(&meta) { return Err("La copia del mundo no es una carpeta segura".into()); }
            let mut file = fs::OpenOptions::new().create_new(true).write(true).open(folder.join(&own_marker)).map_err(|e| e.to_string())?;
            file.write_all(plan.token.as_bytes()).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            part.digest = fingerprint(&folder, &own_marker)?;
        }
        save(root, &plan)
    })();
    if let Err(error) = prepared {
        for part in &plan.parts {
            let folder = parent.join(&part.stage);
            if checked_marker(&folder, &plan, true).is_ok() { let _ = fs::remove_file(folder.join(&own_marker)); }
        }
        return Err(error);
    }
    Ok(plan)
}

pub(crate) fn recover_all(root: &Path) -> Vec<String> {
    let _guard = PUBLICATION.lock().unwrap_or_else(|error| error.into_inner());
    let plans = match raw_records(root) { Ok(plans) => plans, Err(error) => return vec![error] };
    plans.into_iter().filter_map(|(key, json)| parse_record(key, json)
        .and_then(|(key, plan)| recover(root, &key, plan, &mut |_| {})).err()).collect()
}

pub(crate) fn recover_server(root: &Path, server: &Server) -> Result<(), String> {
    let parent = Path::new(&server.directory).canonicalize().map_err(|e| e.to_string())?;
    if !records(root)?.iter().any(|(_, plan)| plan.parent == parent) { return Ok(()); }
    let _guard = PUBLICATION.lock().unwrap_or_else(|error| error.into_inner());
    for (key, plan) in records(root)? {
        if plan.parent == parent { recover(root, &key, plan, &mut |_| {})?; }
    }
    Ok(())
}

pub(crate) fn pending_server(root: &Path, server: &Server) -> Result<bool, String> {
    let parent = Path::new(&server.directory).canonicalize().map_err(|e| e.to_string())?;
    Ok(records(root)?.iter().any(|(_, plan)| plan.parent == parent))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture { root: PathBuf, parent: PathBuf, server: Server, parts: Vec<Part>, reserved: Vec<String> }
    impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); } }
    fn fixture() -> Fixture {
        let root = std::env::temp_dir().join(format!("tch-world-publish-{}", Uuid::new_v4()));
        let server = storage::create(&root, crate::model::CreateServer { name: "Recovery".into(), version: "1.21.4".into(), platform: "paper".into(), memory_min_mb: 512, memory_max_mb: 1024, max_players: 2, port: 25565, storage_target_gb: 2, image_path: None }).unwrap();
        let parent = Path::new(&server.directory).canonicalize().unwrap();
        let reserved = vec!["copy".into(), "copy_nether".into(), "copy_the_end".into()];
        let parts = reserved.iter().enumerate().map(|(index, name)| {
            let stage = parent.join(format!(".tch-world-{}.part", Uuid::new_v4()));
            fs::create_dir(&stage).unwrap();
            fs::write(stage.join("level.dat"), format!("world-{index}")).unwrap();
            Part { stage, target: parent.join(name) }
        }).collect();
        Fixture { root, parent, server, parts, reserved }
    }
    fn interrupt(f: &Fixture, phase: usize) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| publish_with(&f.root, &f.server.id, &f.parent, &f.parts, &f.reserved, &mut |step| { if step == phase { panic!("simulated process interruption"); } })));
        assert!(result.is_err());
    }
    const CHILD_ROOT: &str = "TCH_TEST_WORLD_PUBLICATION_CHILD";
    #[derive(Serialize, Deserialize)]
    struct KillRequest { server_id: String, phase: usize, stages: Vec<PathBuf> }
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
    }
    fn publication_child(root: &Path) {
        let request: KillRequest = serde_json::from_slice(&fs::read(root.join("kill-request.json")).unwrap()).unwrap();
        assert!(request.phase <= 7);
        assert_eq!(request.stages.len(), 3);
        let server = storage::get(root, &request.server_id).unwrap();
        let parent = Path::new(&server.directory).canonicalize().unwrap();
        let reserved: Vec<String> = ["copy", "copy_nether", "copy_the_end"].into_iter().map(String::from).collect();
        let parts: Vec<_> = request.stages.into_iter().zip(&reserved)
            .map(|(stage, name)| Part { stage, target: parent.join(name) }).collect();
        let result = publish_with(root, &server.id, &parent, &parts, &reserved, &mut |step| {
            if step == request.phase {
                let mut signal = fs::File::create(root.join("publication-paused.part")).unwrap();
                signal.write_all(step.to_string().as_bytes()).unwrap();
                signal.sync_all().unwrap();
                drop(signal);
                fs::rename(root.join("publication-paused.part"), root.join("publication-paused")).unwrap();
                loop { std::thread::park(); }
            }
        });
        panic!("publication did not reach kill phase {}: {result:?}", request.phase);
    }
    fn kill_publication(f: &Fixture, phase: usize) {
        let request = KillRequest { server_id: f.server.id.clone(), phase, stages: f.parts.iter().map(|part| part.stage.clone()).collect() };
        fs::write(f.root.join("kill-request.json"), serde_json::to_vec(&request).unwrap()).unwrap();
        let mut child = Child(std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "world_publication::tests::interrupted_family_publication_and_cleanup_recover_idempotently"])
            .env(CHILD_ROOT, &f.root).stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().unwrap());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !f.root.join("publication-paused").is_file() {
            assert!(child.0.try_wait().unwrap().is_none(), "publication child exited before phase {phase}");
            assert!(std::time::Instant::now() < deadline, "publication child did not reach phase {phase}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        child.0.kill().unwrap();
        assert!(!child.0.wait().unwrap().success());
        assert_eq!(fs::read_to_string(f.root.join("publication-paused")).unwrap(), phase.to_string());
        assert_eq!(records(&f.root).unwrap().len(), 1);
        let published = phase.min(3);
        assert_eq!(f.parts.iter().filter(|part| part.target.is_dir()).count(), published);
        assert_eq!(f.parts.iter().filter(|part| part.stage.is_dir()).count(), 3 - published);
    }
    #[test]
    fn interrupted_family_publication_and_cleanup_recover_idempotently() {
        if let Some(root) = std::env::var_os(CHILD_ROOT) { publication_child(Path::new(&root)); }
        for kill in [false, true] {
            for phase in 0..=7 {
                let f = fixture();
                if kill { kill_publication(&f, phase); } else { interrupt(&f, phase); }
                recover_server(&f.root, &f.server).unwrap();
                assert!(recover_all(&f.root).is_empty());
                assert!(!pending_server(&f.root, &f.server).unwrap());
                for (index, part) in f.parts.iter().enumerate() {
                    assert_eq!(fs::read(part.target.join("level.dat")).unwrap(), format!("world-{index}").as_bytes());
                    assert!(!part.stage.exists());
                    assert_eq!(fs::read_dir(&part.target).unwrap().count(), 1);
                }
                let root = f.root.clone();
                drop(f);
                assert!(!root.exists(), "temporary publication fixture was not removed");
                if kill { eprintln!("Recovered family after process kill at phase {phase}; bytes, journal, stages, markers and temporary cleanup verified"); }
            }
        }
        let f = fixture();
        let names = ["world", "world_nether", "world_the_end"];
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&[10, 0, 0, 10, 0, 4, b'D', b'a', b't', b'a', 0, 0]).unwrap();
        let level = encoder.finish().unwrap();
        let sources: Vec<_> = names.iter().enumerate().map(|(index, name)| {
            let folder = f.parent.join(name);
            fs::create_dir_all(folder.join("region")).unwrap();
            fs::write(folder.join("level.dat"), &level).unwrap();
            fs::write(folder.join("region/r.0.0.mca"), format!("dimension-{index}")).unwrap();
            let digest = fingerprint(&folder, "").unwrap();
            (folder, digest)
        }).collect();
        let jobs = crate::file_tasks::Registry::default();
        let copied = crate::file_tasks::duplicate_world(&f.root, &f.server, "world", "clone", &jobs, |_| {}).unwrap();
        assert_eq!(copied.dimensions, ["clone_nether", "clone_the_end"]);
        let export = f.root.join("export"); fs::create_dir(&export).unwrap();
        let exported = crate::file_tasks::export_world(&f.root, &f.server, "world", &export, &jobs, |_| {}).unwrap();
        assert_eq!(exported, export.canonicalize().unwrap().join("world"));
        for (index, (source, digest)) in sources.iter().enumerate() {
            assert_eq!(fingerprint(source, "").unwrap(), *digest);
            for target in [f.parent.join(names[index].replacen("world", "clone", 1)), export.join(names[index])] {
                assert_eq!(fs::read(target.join("level.dat")).unwrap(), level);
                assert_eq!(fs::read(target.join("region/r.0.0.mca")).unwrap(), format!("dimension-{index}").as_bytes());
                assert_eq!(fingerprint(&target, "").unwrap(), *digest);
            }
        }
        assert!(jobs.lock().unwrap().is_empty());
        assert!(records(&f.root).unwrap().is_empty());
        let selected = crate::world_import::inspect(&exported).unwrap();
        assert_eq!(selected.dimensions, ["world_nether", "world_the_end"]);
        assert_eq!(selected.size_bytes, sources.iter().map(|(path, _)| crate::backup::size(path)).sum::<u64>());
        assert!(crate::world_import::inspect(&export.join("world_nether")).is_err());
        let imported = crate::file_tasks::import_world(&f.root, &f.server, &exported, "imported", &selected.dimensions, &jobs, |_| {}).unwrap();
        assert_eq!(imported.dimensions, ["imported_nether", "imported_the_end"]);
        for (index, (_, digest)) in sources.iter().enumerate() {
            assert_eq!(fingerprint(&f.parent.join(names[index].replacen("world", "imported", 1)), "").unwrap(), *digest);
        }
        assert!(crate::file_tasks::import_world(&f.root, &f.server, &exported, "unreviewed", &[], &jobs, |_| {}).is_err());
        assert!(!f.parent.join("unreviewed").exists());
        let mut vanilla = f.server.clone(); vanilla.platform = "vanilla".into();
        assert!(crate::file_tasks::import_world(&f.root, &vanilla, &exported, "wrong-engine", &selected.dimensions, &jobs, |_| {}).is_err());
        assert!(!f.parent.join("wrong-engine").exists());
        fs::rename(export.join("world_the_end"), export.join("removed-end")).unwrap();
        assert!(crate::file_tasks::import_world(&f.root, &f.server, &exported, "changed", &selected.dimensions, &jobs, |_| {}).is_err());
        assert!(!f.parent.join("changed").exists());
        fs::rename(export.join("removed-end"), export.join("world_the_end")).unwrap();
        fs::write(f.parent.join("occupied_nether"), b"foreign data").unwrap();
        assert!(crate::file_tasks::import_world(&f.root, &f.server, &exported, "occupied", &selected.dimensions, &jobs, |_| {}).is_err());
        assert_eq!(fs::read(f.parent.join("occupied_nether")).unwrap(), b"foreign data");
        assert!(!f.parent.join("occupied").exists());
        for (index, (_, digest)) in sources.iter().enumerate() {
            assert_eq!(fingerprint(&export.join(names[index]), "").unwrap(), *digest);
        }
        let single = export.join("standalone"); fs::create_dir(&single).unwrap();
        fs::write(single.join("level.dat"), &level).unwrap();
        assert!(crate::world_import::inspect(&single).unwrap().dimensions.is_empty());
        let plain = crate::file_tasks::import_world(&f.root, &vanilla, &single, "plain", &[], &jobs, |_| {}).unwrap();
        assert!(plain.dimensions.is_empty());
        assert_eq!(fs::read(f.parent.join("plain/level.dat")).unwrap(), level);
        let stages_before = fs::read_dir(&f.parent).unwrap().filter(|entry| entry.as_ref().unwrap().file_name().to_string_lossy().starts_with(".tch-world-")).count();
        storage::connect(&f.root).unwrap().execute_batch("CREATE TRIGGER reject_world_copy BEFORE INSERT ON application_settings WHEN NEW.key LIKE 'world-copy:%' BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
        assert!(crate::file_tasks::duplicate_world(&f.root, &f.server, "world", "failed", &jobs, |_| {}).is_err());
        assert!(crate::file_tasks::import_world(&f.root, &f.server, &exported, "failed-import", &selected.dimensions, &jobs, |_| {}).is_err());
        assert!(!f.parent.join("failed").exists());
        assert!(!f.parent.join("failed-import").exists());
        assert_eq!(fs::read_dir(&f.parent).unwrap().filter(|entry| entry.as_ref().unwrap().file_name().to_string_lossy().starts_with(".tch-world-")).count(), stages_before);
        assert!(jobs.lock().unwrap().is_empty());
        assert!(records(&f.root).unwrap().is_empty());
    }
    #[test]
    fn foreign_destinations_and_changed_owned_worlds_are_preserved() {
        {
            let f = fixture();
            fs::create_dir(&f.parts[0].target).unwrap();
            assert!(matches!(publish(&f.root, &f.server.id, &f.parent, &f.parts, &f.reserved), Err(Failure::Unprepared(_))));
            assert!(records(&f.root).unwrap().is_empty());
            assert!(f.parts.iter().all(|part| part.stage.exists()));
        }
        {
            let f = fixture();
            let result = publish_with(&f.root, &f.server.id, &f.parent, &f.parts, &f.reserved, &mut |step| {
                if step == 0 { fs::create_dir(&f.parts[0].target).unwrap(); }
            });
            assert!(matches!(result, Err(Failure::Pending(_))));
            assert_eq!(records(&f.root).unwrap().len(), 1);
            assert!(f.parts.iter().all(|part| part.stage.exists()));
        }
        for scenario in 0..4 {
            let mut f = fixture();
            if scenario == 3 { fs::remove_dir_all(&f.parts.pop().unwrap().stage).unwrap(); }
            interrupt(&f, 1);
            let evidence = match scenario {
                1 => f.parts[0].target.join("level.dat"),
                2 => {
                    let (_, plan) = records(&f.root).unwrap().remove(0);
                    f.parts[0].target.join(marker(&plan))
                }
                _ => {
                    let foreign = f.parent.join("copy_the_end");
                    fs::create_dir(&foreign).unwrap(); foreign.join("foreign.txt")
                }
            };
            fs::write(&evidence, b"user data").unwrap();
            assert!(recover_server(&f.root, &f.server).is_err());
            assert_eq!(fs::read(evidence).unwrap(), b"user data");
            assert!(f.parts[1].stage.exists() && !f.parts[1].target.exists());
            assert!(pending_server(&f.root, &f.server).unwrap());
        }
    }
    #[test]
    fn unavailable_external_export_remains_pending_without_blocking_server() {
        let mut f = fixture();
        let external = f.root.join("external"); fs::create_dir(&external).unwrap();
        f.parent = external.canonicalize().unwrap();
        for part in &mut f.parts {
            let stage = f.parent.join(part.stage.file_name().unwrap()); fs::rename(&part.stage, &stage).unwrap(); part.stage = stage;
            part.target = f.parent.join(part.target.file_name().unwrap());
        }
        interrupt(&f, 1);
        let offline = f.root.join("disconnected"); fs::rename(&f.parent, &offline).unwrap();
        assert!(!recover_all(&f.root).is_empty());
        recover_server(&f.root, &f.server).unwrap();
        assert!(!pending_server(&f.root, &f.server).unwrap());
        fs::rename(offline, &f.parent).unwrap();
        assert!(recover_all(&f.root).is_empty());
        for part in &f.parts { assert!(part.target.join("level.dat").exists()); }
    }
    #[test]
    fn imported_journals_are_ignored_and_invalid_trusted_records_cannot_touch_files() {
        let f = fixture();
        fs::write(f.parent.join(".tch-world-transaction.json"), b"untrusted imported journal").unwrap();
        assert!(recover_all(&f.root).is_empty());
        interrupt(&f, 0);
        let (key, mut plan) = records(&f.root).unwrap().remove(0);
        plan.parts[0].target = "../escape".into(); save(&f.root, &plan).unwrap();
        assert!(!recover_all(&f.root).is_empty());
        assert!(f.parts.iter().all(|part| part.stage.join("level.dat").exists() && !part.target.exists()));
        assert!(storage::connect(&f.root).unwrap().query_row("SELECT value FROM application_settings WHERE key=?1", [key], |_| Ok(())).is_ok());
    }
}
