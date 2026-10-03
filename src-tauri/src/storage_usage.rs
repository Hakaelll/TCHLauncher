use crate::{model::Server,storage};
use serde::Serialize;
use std::{collections::HashSet,fs,path::{Path,PathBuf}};
use rusqlite::OptionalExtension;
use walkdir::WalkDir;
use std::{collections::HashMap,sync::{Arc,Mutex},time::{Duration,Instant}};

#[derive(Clone,Debug)]
pub struct MeasuredSize{pub bytes:u64,pub skipped_links:u64}
#[derive(Clone)]
pub struct SizeSample{pub sampled_at:i64,pub result:Result<MeasuredSize,String>}
struct CachedSize{path:PathBuf,when:Instant,sample:SizeSample}
#[derive(Default)]
pub struct SizeCache{entries:Mutex<HashMap<String,Arc<Mutex<Option<CachedSize>>>>>}
impl SizeCache{
    pub fn invalidate(&self,id:&str){if let Ok(mut entries)=self.entries.lock(){entries.remove(id);}}
    pub fn sample(&self,id:&str,path:&Path)->Result<SizeSample,String>{
        let entry=self.entries.lock().map_err(|_|"No se pudo consultar la caché de almacenamiento")?.entry(id.into()).or_default().clone();
        // A scan holds only this server's cell; other servers remain independent.
        let mut cached=entry.lock().map_err(|_|"No se pudo medir el almacenamiento")?;
        if let Some(previous)=cached.as_ref(){
            let ttl=if previous.sample.result.is_ok(){60}else{15};
            if previous.path==path&&previous.when.elapsed()<Duration::from_secs(ttl){return Ok(previous.sample.clone());}
        }
        let result=measure_size(path);
        let sample=SizeSample{sampled_at:chrono::Utc::now().timestamp_millis(),result};
        *cached=Some(CachedSize{path:path.into(),when:Instant::now(),sample:sample.clone()});Ok(sample)
    }
}

pub fn measure_size(path:&Path)->Result<MeasuredSize,String>{
    let base=path.canonicalize().map_err(|e|format!("No se encontró la carpeta del servidor: {e}"))?;
    if !base.is_dir(){return Err("La ruta del servidor no es una carpeta".into());}
    let mut measured=MeasuredSize{bytes:0,skipped_links:0};
    for entry in WalkDir::new(base).follow_links(false){
        let entry=entry.map_err(|e|format!("No se pudieron medir todos los archivos: {e}"))?;
        if entry.file_type().is_symlink(){measured.skipped_links+=1;}
        else if entry.file_type().is_file(){add(&mut measured.bytes,entry.metadata().map_err(|e|format!("No se pudo medir un archivo: {e}"))?.len())?;}
    }
    Ok(measured)
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Volume {pub mount:String,pub available_bytes:u64,pub total_bytes:u64}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Usage {pub sampled_at:i64,pub server_bytes:u64,pub worlds_bytes:u64,pub content_bytes:u64,pub logs_bytes:u64,pub other_bytes:u64,pub backup_bytes:u64,pub skipped_links:u64,pub warning_percent:u8,pub server_volume:Option<Volume>,pub backup_volume:Option<Volume>}

fn settings(root:&Path)->Result<rusqlite::Connection,String>{
    let db=storage::connect(root)?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS storage_warnings (server_id TEXT PRIMARY KEY REFERENCES servers(id) ON DELETE CASCADE, warning_percent INTEGER NOT NULL CHECK(warning_percent BETWEEN 1 AND 100));").map_err(|e|e.to_string())?;
    Ok(db)
}
pub fn warning_percent(root:&Path,id:&str)->Result<u8,String>{
    settings(root)?.query_row("SELECT warning_percent FROM storage_warnings WHERE server_id=?1",[id],|row|row.get::<_,u8>(0)).optional().map(|value|value.unwrap_or(90)).map_err(|e|e.to_string())
}
pub fn save_warning(root:&Path,id:&str,percent:u8)->Result<(),String>{
    if !(1..=100).contains(&percent){return Err("El aviso debe estar entre el 1 y el 100 % del objetivo".into());}
    storage::get(root,id)?;
    settings(root)?.execute("INSERT INTO storage_warnings(server_id,warning_percent) VALUES(?1,?2) ON CONFLICT(server_id) DO UPDATE SET warning_percent=excluded.warning_percent",rusqlite::params![id,percent]).map_err(|e|e.to_string())?;
    Ok(())
}
fn add(total:&mut u64,size:u64)->Result<(),String>{*total=total.checked_add(size).ok_or("El tamaño de los archivos supera el límite admitido")?;Ok(())}
fn backup_size(path:&Path)->Result<(u64,u64),String>{
    match fs::symlink_metadata(path){Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok((0,0)),Err(error)=>return Err(error.to_string()),Ok(meta) if !meta.is_dir()||meta.file_type().is_symlink()=>return Err("La carpeta de copias no es segura".into()),_=>()}
    let mut bytes=0;let mut skipped=0;
    for entry in WalkDir::new(path).follow_links(false){let entry=entry.map_err(|e|format!("No se pudieron medir todas las copias: {e}"))?;if entry.file_type().is_symlink(){skipped+=1;}else if entry.file_type().is_file(){add(&mut bytes,entry.metadata().map_err(|e|e.to_string())?.len())?;}}
    Ok((bytes,skipped))
}
fn volume(path:&Path,disks:&sysinfo::Disks)->Option<Volume>{
    let mut existing=path.to_path_buf();while !existing.exists(){if !existing.pop(){return None;}}
    let actual=existing.canonicalize().ok()?;
    let disk=disks.list().iter().filter(|disk|disk.mount_point().canonicalize().is_ok_and(|mount|actual.starts_with(mount))).max_by_key(|disk|disk.mount_point().as_os_str().len())?;
    if disk.total_space()==0{return None;}
    Some(Volume{mount:disk.mount_point().to_string_lossy().into(),available_bytes:disk.available_space(),total_bytes:disk.total_space()})
}
pub fn inspect(root:&Path,server:&Server)->Result<Usage,String>{
    uuid::Uuid::parse_str(&server.id).map_err(|_|"Identificador de servidor no válido")?;
    let base=PathBuf::from(&server.directory).canonicalize().map_err(|e|format!("No se encontró la carpeta del servidor: {e}"))?;
    let mut worlds=HashSet::new();
    for entry in fs::read_dir(&base).map_err(|e|e.to_string())?{let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name().to_string_lossy().into_owned();if crate::world::is_world(server,&name){worlds.insert(name);}}
    let mut usage=Usage{sampled_at:0,server_bytes:0,worlds_bytes:0,content_bytes:0,logs_bytes:0,other_bytes:0,backup_bytes:0,skipped_links:0,warning_percent:warning_percent(root,&server.id)?,server_volume:None,backup_volume:None};
    for entry in WalkDir::new(&base).follow_links(false){
        let entry=entry.map_err(|e|format!("No se pudieron medir todos los archivos del servidor: {e}"))?;
        if entry.file_type().is_symlink(){usage.skipped_links+=1;continue;}
        if !entry.file_type().is_file(){continue;}
        let size=entry.metadata().map_err(|e|e.to_string())?.len();add(&mut usage.server_bytes,size)?;
        let relative=entry.path().strip_prefix(&base).map_err(|e|e.to_string())?;
        let first=relative.components().next().map(|part|part.as_os_str().to_string_lossy().into_owned()).unwrap_or_default();
        let bucket=if worlds.contains(&first){&mut usage.worlds_bytes}else if first=="mods"||first=="plugins"{&mut usage.content_bytes}else if first=="logs"{&mut usage.logs_bytes}else{&mut usage.other_bytes};add(bucket,size)?;
    }
    let backup_base=root.join("backups");
    if fs::symlink_metadata(&backup_base).is_ok_and(|meta|meta.file_type().is_symlink()){return Err("La carpeta de copias no es segura".into());}
    let backup_path=backup_base.join(&server.id);let (bytes,skipped)=backup_size(&backup_path)?;usage.backup_bytes=bytes;usage.skipped_links+=skipped;
    let disks=sysinfo::Disks::new_with_refreshed_list();usage.server_volume=volume(&base,&disks);usage.backup_volume=volume(&backup_path,&disks);usage.sampled_at=chrono::Utc::now().timestamp_millis();
    Ok(usage)
}

#[cfg(test)] mod tests{
    use super::*;
    use std::io::Write;
    #[test]fn size_cache_distinguishes_empty_missing_and_changed_server_locations(){
        let root=std::env::temp_dir().join(format!("tch-size-cache-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
        let cache=SizeCache::default();let first=root.join("first");
        assert!(cache.sample("one",&first).unwrap().result.is_err());
        fs::create_dir(&first).unwrap();
        assert!(cache.sample("one",&first).unwrap().result.is_err());
        cache.invalidate("one");assert_eq!(cache.sample("one",&first).unwrap().result.unwrap().bytes,0);
        fs::write(first.join("world.bin"),vec![1u8;250]).unwrap();
        assert_eq!(cache.sample("one",&first).unwrap().result.unwrap().bytes,0);
        cache.invalidate("one");assert_eq!(cache.sample("one",&first).unwrap().result.unwrap().bytes,250);
        let second=root.join("second");fs::create_dir(&second).unwrap();fs::write(second.join("server.jar"),vec![2u8;125]).unwrap();
        assert_eq!(cache.sample("one",&second).unwrap().result.unwrap().bytes,125);
        assert_eq!(cache.sample("two",&first).unwrap().result.unwrap().bytes,250);
        assert!(measure_size(&first.join("world.bin")).is_err());
        let mut size=u64::MAX;assert!(add(&mut size,1).is_err());assert_eq!(size,u64::MAX);
        fs::remove_dir_all(root).unwrap();
    }
    #[test] fn measures_disjoint_categories_and_persists_validated_warning_preferences(){
        let root=std::env::temp_dir().join(format!("tch-storage-usage-{}",uuid::Uuid::new_v4()));
        let server=storage::create(&root,crate::model::CreateServer{name:"Storage".into(),version:"1.21.4".into(),platform:"paper".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();let base=Path::new(&server.directory);
        for folder in ["world/region","plugins","logs"]{fs::create_dir_all(base.join(folder)).unwrap();}
        let mut nbt=flate2::write::GzEncoder::new(fs::File::create(base.join("world/level.dat")).unwrap(),flate2::Compression::default());nbt.write_all(&[10,0,0,10,0,4,b'D',b'a',b't',b'a',0]).unwrap();nbt.finish().unwrap();
        fs::write(base.join("world/region/r.0.0.mca"),vec![1u8;300]).unwrap();fs::write(base.join("plugins/example.jar"),vec![2u8;50]).unwrap();fs::write(base.join("logs/latest.log"),vec![3u8;40]).unwrap();fs::write(base.join("server.jar"),vec![4u8;70]).unwrap();
        let backup=root.join("backups").join(&server.id).join("copy");fs::create_dir_all(&backup).unwrap();fs::write(backup.join("server.jar"),vec![5u8;90]).unwrap();
        let measured=inspect(&root,&server).unwrap();assert_eq!(measured.server_bytes,crate::backup::size(base));assert_eq!(measured.worlds_bytes,crate::backup::size(&base.join("world")));assert_eq!(measured.content_bytes,50);assert_eq!(measured.logs_bytes,40);assert_eq!(measured.backup_bytes,90);assert_eq!(measured.worlds_bytes+measured.content_bytes+measured.logs_bytes+measured.other_bytes,measured.server_bytes);assert_eq!(measured.warning_percent,90);
        save_warning(&root,&server.id,75).unwrap();assert_eq!(warning_percent(&root,&server.id).unwrap(),75);assert!(save_warning(&root,&server.id,0).is_err());assert!(save_warning(&root,&server.id,101).is_err());assert_eq!(warning_percent(&root,&server.id).unwrap(),75);
        storage::delete(&root,&server.id).unwrap();assert_eq!(settings(&root).unwrap().query_row("SELECT count(*) FROM storage_warnings",[],|row|row.get::<_,u32>(0)).unwrap(),0);fs::remove_dir_all(root).unwrap();
    }
    #[test] fn invalid_backup_location_is_an_error_instead_of_zero_usage(){
        let root=std::env::temp_dir().join(format!("tch-storage-invalid-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();let path=root.join("copies");fs::write(&path,b"keep").unwrap();assert!(backup_size(&path).is_err());assert_eq!(fs::read(&path).unwrap(),b"keep");fs::remove_dir_all(root).unwrap();
    }
}
