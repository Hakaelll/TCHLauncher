//! Publish a prepared Minecraft directory and its database identity together.
//! The SQLite token decides recovery after interruption at either rename.
use crate::{model::Server,storage};
use std::{fs,io::Write,path::{Path,PathBuf}};
use uuid::Uuid;
use rusqlite::OptionalExtension;

pub(crate) fn paths(root:&Path,server:&Server)->Result<(PathBuf,PathBuf,PathBuf,PathBuf),String> {
    Uuid::parse_str(&server.id).map_err(|_|"Servidor no válido")?;
    let target=PathBuf::from(&server.directory);
    if !target.is_absolute(){return Err("La carpeta del servidor debe ser absoluta".into());}
    let parent=target.parent().ok_or("Carpeta del servidor no válida")?;
    let stage=parent.join(format!(".tch-minecraft-stage-{}",server.id));
    let old=parent.join(format!(".tch-minecraft-old-{}",server.id));
    let journal=root.join("database").join(format!(".tch-minecraft-{}.journal",server.id));
    for path in [&target,&stage,&old,&journal,&journal.with_extension("pending")] {
        if fs::symlink_metadata(path).is_ok_and(|meta|meta.file_type().is_symlink()) {
            return Err("El cambio de Minecraft no admite enlaces simbólicos".into());
        }
    }
    Ok((target,stage,old,journal))
}

pub(crate) fn recover(root:&Path,id:&str)->Result<(),String> {
    let server=storage::get(root,id)?;
    let (target,stage,old,journal)=paths(root,&server)?;
    if journal.exists() {
        let meta=fs::metadata(&journal).map_err(|e|e.to_string())?;
        if !meta.is_file()||meta.len()>64{return Err("Registro del cambio de Minecraft no válido".into());}
        let token=fs::read_to_string(&journal).map_err(|e|e.to_string())?;
        Uuid::parse_str(&token).map_err(|_|"Registro del cambio de Minecraft no válido")?;
        let marker:Option<String>=storage::connect(root)?.query_row("SELECT value FROM application_settings WHERE key=?1",[format!("minecraft-update:{id}")],|row|row.get(0)).optional().map_err(|e|e.to_string())?;
        if marker.as_deref()==Some(&token) {
            if !target.is_dir(){return Err("Falta la instalación de Minecraft confirmada".into());}
            let jar=Path::new(server.jar_path.as_deref().ok_or("Falta el arranque confirmado")?);
            crate::loader_identity::checked_file(&target,jar,512_000_000)?;
            if old.exists(){fs::remove_dir_all(&old).map_err(|e|e.to_string())?;}
        }else if old.exists() {
            if target.exists(){fs::remove_dir_all(&target).map_err(|e|e.to_string())?;}
            fs::rename(&old,&target).map_err(|e|e.to_string())?;
        }else if !target.is_dir(){return Err("Falta la carpeta original del servidor".into());}
        if stage.exists(){fs::remove_dir_all(&stage).map_err(|e|e.to_string())?;}
        fs::remove_file(&journal).map_err(|e|e.to_string())?;
    }else {
        if old.exists(){return Err("Hay una instalación anterior sin registro de recuperación".into());}
        if stage.exists(){fs::remove_dir_all(&stage).map_err(|e|e.to_string())?;}
    }
    let pending=journal.with_extension("pending");
    if pending.exists(){fs::remove_file(pending).map_err(|e|e.to_string())?;}Ok(())
}

pub(crate) fn recover_all(root:&Path)->Result<(),String> {
    for server in storage::list(root)?{recover(root,&server.id)?;}Ok(())
}

fn commit(root:&Path,before:&Server,version:&str,jar:&Path,java:&Path,token:&str)->Result<(),String> {
    let mut db=storage::connect(root)?;let transaction=db.transaction().map_err(|e|e.to_string())?;
    let changed=transaction.execute(
        "UPDATE servers SET version=?2,jar_path=?3,java_path=?4 WHERE id=?1 AND version=?5 AND jar_path IS ?6 AND directory=?7 AND platform=?8 AND java_path IS ?9",
        rusqlite::params![before.id,version,jar.to_string_lossy(),java.to_string_lossy(),before.version,before.jar_path,before.directory,before.platform,before.java_path]
    ).map_err(|e|e.to_string())?;
    if changed!=1{return Err("La instalación cambió durante la publicación de Minecraft".into());}
    transaction.execute("INSERT INTO application_settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",rusqlite::params![format!("minecraft-update:{}",before.id),token]).map_err(|e|e.to_string())?;
    transaction.commit().map_err(|e|e.to_string())
}

pub(crate) fn publish(root:&Path,before:&Server,version:&str,launcher:&Path,java:&Path)->Result<(),String> {
    crate::fabric::validate_version(version)?;
    if !java.is_absolute()||!java.is_file(){return Err("Falta el runtime preparado".into());}
    let (target,stage,old,journal)=paths(root,before)?;
    if old.exists()||journal.exists(){return Err("Recupera primero la actualización anterior".into());}
    if !target.is_dir()||!stage.is_dir(){return Err("Falta la carpeta original o la instalación preparada".into());}
    crate::loader_identity::checked_file(&stage,launcher,512_000_000)?;
    crate::loader_identity::checked_file(&stage,&stage.join("server.properties"),2_000_000)?;
    let base=stage.canonicalize().map_err(|e|e.to_string())?;
    let canonical=launcher.canonicalize().map_err(|e|e.to_string())?;
    let relative=canonical.strip_prefix(&base).map_err(|_|"El arranque preparado sale de su carpeta")?;
    let final_jar=target.join(relative);
    let token=Uuid::new_v4().to_string();
    // Keep journal and pending file on the library's volume. Imported servers
    // can reside on another drive; their stage/old directories stay adjacent.
    let pending=journal.with_extension("pending");
    let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&pending).map_err(|e|e.to_string())?;
    file.write_all(token.as_bytes()).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())?;drop(file);
    fs::rename(&pending,&journal).map_err(|e|e.to_string())?;
    let result=(||{
        fs::rename(&target,&old).map_err(|e|e.to_string())?;
        fs::rename(&stage,&target).map_err(|e|e.to_string())?;
        commit(root,before,version,&final_jar,java,&token)
    })();
    recover(root,&before.id)?;result
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Library(PathBuf);
    impl Drop for Library{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture()->(Library,Server,PathBuf,PathBuf) {
        let root=Library(std::env::temp_dir().join(format!("tch-instance-publish-{}",Uuid::new_v4())));
        let server=storage::create(&root.0,crate::model::CreateServer{name:"Publication".into(),version:"1.21.4".into(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        let old_jar=Path::new(&server.directory).join("server.jar");fs::write(&old_jar,b"old launcher").unwrap();
        let java=root.0.join("java.exe");fs::write(&java,b"runtime fixture").unwrap();
        storage::update_install(&root.0,&server.id,&old_jar,&java).unwrap();
        let server=storage::get(&root.0,&server.id).unwrap();
        let (_,stage,_,_)=paths(&root.0,&server).unwrap();
        crate::backup::copy_tree(Path::new(&server.directory),&stage).unwrap();
        let jar=stage.join("server.jar");fs::write(&jar,b"new launcher").unwrap();
        (root,server,jar,java)
    }

    #[test]
    fn database_failure_rolls_back_original_folder_and_identity() {
        let (root,server,jar,java)=fixture();
        storage::connect(&root.0).unwrap().execute_batch("CREATE TRIGGER reject_update BEFORE UPDATE ON servers BEGIN SELECT RAISE(ABORT,'fixture failure'); END;").unwrap();
        assert!(publish(&root.0,&server,"1.21.5",&jar,&java).is_err());
        let current=storage::get(&root.0,&server.id).unwrap();assert_eq!(current.version,server.version);assert_eq!(current.jar_path,server.jar_path);
        assert_eq!(fs::read(current.jar_path.unwrap()).unwrap(),b"old launcher");
        let (_,stage,old,journal)=paths(&root.0,&server).unwrap();assert!(!stage.exists()&&!old.exists()&&!journal.exists());
    }
    #[test]
    fn recovers_interruption_before_and_after_the_database_commit() {
        for committed in [false,true] {
            let (root,server,jar,java)=fixture();let (target,stage,old,journal)=paths(&root.0,&server).unwrap();let token=Uuid::new_v4().to_string();
            fs::write(&journal,&token).unwrap();fs::rename(&target,&old).unwrap();fs::rename(&stage,&target).unwrap();
            if committed{commit(&root.0,&server,"1.21.5",&target.join(jar.file_name().unwrap()),&java,&token).unwrap();}
            recover(&root.0,&server.id).unwrap();
            let current=storage::get(&root.0,&server.id).unwrap();
            assert_eq!(current.version,if committed{"1.21.5"}else{"1.21.4"});
            assert_eq!(fs::read(current.jar_path.unwrap()).unwrap(),if committed{b"new launcher"}else{b"old launcher"});
            assert!(!stage.exists()&&!old.exists()&&!journal.exists());
            recover(&root.0,&server.id).unwrap();
        }
    }

}
