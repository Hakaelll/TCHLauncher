//! Share a selected world and its server content, without Java, accounts or logs.
use crate::{download::DownloadTask, model::Server};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}, time::Instant};
use zip::{write::SimpleFileOptions, ZipWriter};

struct Entry { source: PathBuf, name: String, bytes: u64, modified: std::time::SystemTime }
struct StageFile(PathBuf);
impl Drop for StageFile {
    fn drop(&mut self) { let _ = fs::remove_file(&self.0); }
}
#[derive(Serialize)]
struct FileRecord { path: String, bytes: u64, sha256: String }

#[cfg(windows)]
fn publish_new(source:&Path,destination:&Path)->std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name="kernel32")]
    extern "system" { fn MoveFileExW(existing:*const u16,new:*const u16,flags:u32)->i32; }
    let source:Vec<u16>=source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination:Vec<u16>=destination.as_os_str().encode_wide().chain(Some(0)).collect();
    // No REPLACE_EXISTING flag: an unexpected file is never overwritten.
    if unsafe {MoveFileExW(source.as_ptr(),destination.as_ptr(),0)}==0 {Err(std::io::Error::last_os_error())} else {Ok(())}
}
#[cfg(not(windows))]
fn publish_new(source:&Path,destination:&Path)->std::io::Result<()> {
    fs::hard_link(source,destination)?;fs::remove_file(source)
}

fn plain(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() { return false; }
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 { return false; }
    }
    true
}

fn collect(root: &Path, folder: &Path, prefix: &str, jars_only: bool, entries: &mut Vec<Entry>, task: &DownloadTask) -> Result<(), String> {
    let meta=fs::symlink_metadata(folder).map_err(|e|e.to_string())?;
    if !plain(&meta) || !meta.is_dir() { return Err("La exportación no admite enlaces ni carpetas redirigidas".into()); }
    for item in walkdir::WalkDir::new(folder).follow_links(false).max_depth(if jars_only {1} else {usize::MAX}) {
        task.cancellation.check()?;
        let item=item.map_err(|e|e.to_string())?;
        let meta=fs::symlink_metadata(item.path()).map_err(|e|e.to_string())?;
        if !plain(&meta) { return Err("La exportación no admite enlaces ni carpetas redirigidas".into()); }
        if meta.is_dir() { continue; }
        if !meta.is_file() { return Err("La exportación solo admite archivos normales".into()); }
        let actual=item.path().canonicalize().map_err(|e|e.to_string())?;
        if !actual.starts_with(root) { return Err("Un archivo sale de la carpeta del servidor".into()); }
        let relative=item.path().strip_prefix(folder).map_err(|e|e.to_string())?
            .to_str().ok_or("Un archivo tiene un nombre no compatible con ZIP")?.replace('\\',"/");
        if jars_only {
            let filename=relative.to_ascii_lowercase();
            if !filename.ends_with(".jar")&&!filename.ends_with(".jar.disabled") {continue;}
        }
        let name=format!("{prefix}/{relative}");
        crate::modpack::relative_path(&name)?;
        if entries.len()>=100_000 { return Err("La exportación supera 100.000 archivos".into()); }
        entries.push(Entry {source:item.path().into(),name,bytes:meta.len(),modified:meta.modified().map_err(|e|e.to_string())?});
    }
    Ok(())
}

pub fn export(server: &Server, name: &str, destination: &Path, include_config: bool, task: &DownloadTask) -> Result<PathBuf,String> {
    task.cancellation.check()?;
    task.report("Revisando mundo y contenido","preparing");
    let root=Path::new(&server.directory).canonicalize().map_err(|e|e.to_string())?;
    let family=crate::world::world_family(server,name)?;
    let parent=destination.parent().ok_or("Elige un archivo ZIP de destino")?.canonicalize().map_err(|e|e.to_string())?;
    if parent.starts_with(&root) { return Err("Guarda el ZIP fuera de la carpeta del servidor".into()); }
    let destination=parent.join(destination.file_name().ok_or("Elige un archivo ZIP de destino")?);
    if destination.extension().and_then(|s|s.to_str()).is_none_or(|s|!s.eq_ignore_ascii_case("zip")) {
        return Err("El archivo de exportación debe terminar en .zip".into());
    }
    let previous_metadata=match fs::symlink_metadata(&destination) {
        Ok(meta) if plain(&meta)&&meta.is_file()=>Some(meta),
        Ok(_)=>return Err("El destino debe ser un archivo normal".into()),
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>None,
        Err(e)=>return Err(e.to_string()),
    };
    let mut entries=Vec::new();
    let worlds: Vec<_> = family.iter().map(|(name, _)| name.clone()).collect();
    for (part, folder) in &family {
        collect(&root,folder,&format!("worlds/{part}"),false,&mut entries,task)?;
    }
    let content=if server.platform=="paper" {"plugins"} else {"mods"};
    let mut folders=vec![content];
    if include_config { folders.extend(["config","defaultconfigs"]); }
    for folder in &folders {
        match fs::symlink_metadata(root.join(folder)) {
            Ok(_)=>collect(&root,&root.join(folder),folder,server.platform=="paper"&&!include_config&&*folder=="plugins",&mut entries,task)?,
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>(),
            Err(e)=>return Err(e.to_string()),
        }
    }
    entries.sort_by(|a,b|a.name.cmp(&b.name));
    let total=entries.iter().try_fold(0u64,|sum,item|sum.checked_add(item.bytes)).ok_or("La exportación supera el tamaño admitido")?;
    let stage=parent.join(format!(".tch-export-{}.part",uuid::Uuid::new_v4()));
    let result=(|| {
        let file=fs::OpenOptions::new().write(true).create_new(true).open(&stage).map_err(|e|e.to_string())?;
        let _cleanup=StageFile(stage.clone());
        let mut zip=ZipWriter::new(file);
        let options=SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).large_file(true);
        let mut received=0u64;let mut records=Vec::new();let mut buffer=vec![0u8;256*1024];
        let began=Instant::now();let mut last=began;
        task.progress("Comprimiendo mundo y contenido","copying",0,Some(total),None,1,None);
        for item in &entries {
            task.cancellation.check()?;
            let meta=fs::symlink_metadata(&item.source).map_err(|e|e.to_string())?;
            if !plain(&meta)||!meta.is_file()||!item.source.canonicalize().map_err(|e|e.to_string())?.starts_with(&root) {
                return Err("Un archivo cambió de ubicación durante la exportación".into());
            }
            if meta.len()!=item.bytes||meta.modified().map_err(|e|e.to_string())?!=item.modified {
                return Err("Los archivos cambiaron durante la exportación. Vuelve a intentarlo.".into());
            }
            let mut source=File::open(&item.source).map_err(|e|e.to_string())?;
            zip.start_file(&item.name,options).map_err(|e|e.to_string())?;
            let mut hash=Sha256::new();let mut bytes=0u64;
            loop {
                task.cancellation.check()?;
                let count=source.read(&mut buffer).map_err(|e|e.to_string())?;
                if count==0 { break; }
                bytes+=count as u64;
                if bytes>item.bytes { return Err("Los archivos cambiaron durante la exportación. Vuelve a intentarlo.".into()); }
                zip.write_all(&buffer[..count]).map_err(|e|e.to_string())?;hash.update(&buffer[..count]);received+=count as u64;
                if last.elapsed().as_millis()>=100 {
                    task.progress("Comprimiendo mundo y contenido","copying",received,Some(total),Some(received as f64/began.elapsed().as_secs_f64()),1,None);last=Instant::now();
                }
            }
            let after=source.metadata().map_err(|e|e.to_string())?;
            if bytes!=item.bytes||after.len()!=item.bytes||after.modified().map_err(|e|e.to_string())?!=item.modified { return Err("Los archivos cambiaron durante la exportación. Vuelve a intentarlo.".into()); }
            records.push(FileRecord {path:item.name.clone(),bytes,sha256:format!("{:x}",hash.finalize())});
        }
        task.progress("Mundo y contenido comprimidos","copying",received,Some(total),None,1,None);
        task.cancellation.check()?;task.report("Guardando inventario del paquete","verifying");
        let manifest=serde_json::json!({"format":1,"minecraftVersion":server.version,"platform":server.platform,"world":name,"worldFolders":worlds,"contentFolder":content,"includesConfig":include_config,"files":records});
        zip.start_file("tch-export.json",options).map_err(|e|e.to_string())?;
        zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        zip.start_file("README.txt",options).map_err(|e|e.to_string())?;
        zip.write_all(format!("TCHLauncher world + content export\nMinecraft: {}\nServer type: {}\nWorld: {}\n\nExtract this ZIP. Create a stopped server with the same Minecraft version and server type. Import worlds/{name}/ using Worlds > Import world. Keep the extracted sibling folders together: for Paper, review the detected Nether/End folders before importing. The application copies the reviewed family together and applies any chosen destination name to every dimension. Select the imported world as the primary world. Copy {content}/ and, if included, config/ and defaultconfigs/ to that server's folder before starting it. Disabled files remain disabled. When configuration is excluded, Paper includes only top-level .jar and .jar.disabled plugin files, without plugin data folders. Client-required mods must also be installed by each player.\n\nThis is not a complete server backup. It does not include Java, the server launcher, server.properties, EULA acceptance, logs or launcher accounts. World player data and any included mod/plugin configuration remain in this archive. tch-export.json lists the files and their SHA-256 hashes.\n",server.version,server.platform,name).as_bytes()).map_err(|e|e.to_string())?;
        let file=zip.finish().map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;drop(file);
        task.begin_publication("Guardando ZIP de mundo y contenido")?;
        if previous_metadata.is_some() {
            let meta=fs::symlink_metadata(&destination).map_err(|e|e.to_string())?;
            if !plain(&meta)||!meta.is_file() { return Err("El destino cambió durante la exportación".into()); }
            let previous=parent.join(format!(".tch-export-{}.previous",uuid::Uuid::new_v4()));
            fs::rename(&destination,&previous).map_err(|e|e.to_string())?;
            if let Err(error)=fs::rename(&stage,&destination) {
                if let Err(restore)=fs::rename(&previous,&destination) { return Err(format!("{error}; archivo anterior conservado en {} ({restore})",previous.display())); }
                return Err(error.to_string());
            }
            let _=fs::remove_file(previous);
        } else {
            // An unexpected existing file must not be replaced after inspection.
            publish_new(&stage,&destination).map_err(|e|e.to_string())?;
        }
        Ok(destination.clone())
    })();
    result
}
