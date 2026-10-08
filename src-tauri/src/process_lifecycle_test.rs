//! Opt-in verification of the production engine with isolated real Minecraft servers.
use super::*;
use crate::{backup,download::DownloadTask,eula,model::CreateServer,properties,storage};
use std::{collections::VecDeque,path::{Path,PathBuf}};

#[derive(Default)]
struct Events { logs:Mutex<VecDeque<String>>, states:Mutex<Vec<String>> }
impl ProcessEvents for Events {
    fn closing(&self)->bool {false}
    fn publish_state(&self,_id:&str,state:&str) {self.states.lock().unwrap().push(state.into());eprintln!("Server state: {state}");}
    fn startup(&self,event:crate::startup::Event) {eprintln!("Startup phase: {:?}",event.phase);}
    fn log(&self,event:crate::model::LogEvent) {
        let mut lines=self.logs.lock().unwrap();if lines.len()==512 {lines.pop_front();}lines.push_back(event.line);
    }
}
impl Events {
    fn contains(&self,text:&str)->bool {self.logs.lock().unwrap().iter().any(|line|line.contains(text))}
}
struct Library {root:PathBuf,manager:ProcessManager}
impl Drop for Library {
    fn drop(&mut self) {
        let controls:Vec<_>=self.manager.processes.lock().unwrap().values().cloned().collect();
        for control in controls {let _=terminate_and_reap(&mut control.child.lock().unwrap());}
        let deadline=Instant::now()+Duration::from_secs(4);
        while self.manager.any_active()&&Instant::now()<deadline {std::thread::sleep(Duration::from_millis(20));}
        if self.root.exists() {
            let temporary=std::env::temp_dir().canonicalize().unwrap();
            let actual=self.root.canonicalize().unwrap();
            assert!(actual.starts_with(&temporary)&&actual!=temporary,"Cleanup escaped the temporary library");
            if let Err(error)=fs::remove_dir_all(&actual) {eprintln!("Temporary lifecycle cleanup failed: {error}; {}",actual.display());}
        }
    }
}
fn task(id:&str,kind:&str)->DownloadTask {
    let last=Mutex::new(None);
    DownloadTask::new(id,kind,move |event| {
        let current=(event.label,event.stage);let mut previous=last.lock().unwrap();
        if previous.as_ref()!=Some(&current) {eprintln!("{}: {}",current.1,current.0);*previous=Some(current);}
    }).unwrap()
}
fn wait_for(label:&str,seconds:u64,events:&Events,mut condition:impl FnMut()->bool) {
    let deadline=Instant::now()+Duration::from_secs(seconds);
    while !condition() {
        if Instant::now()>=deadline {
            let recent=events.logs.lock().unwrap().clone();
            panic!("{label} timed out; recent real output: {recent:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
fn cycle(library:&Library,server:&Server,port:u16)->(String,u64) {
    let events=Arc::new(Events::default());let manager=&library.manager;
    let began=Instant::now();
    manager.launch(server,events.clone(),&crate::jvm::get(&library.root,&server.id).unwrap()).unwrap();
    let session=manager.session_token(&server.id).unwrap();
    wait_for("Minecraft readiness",180,&events,|| {
        assert!(manager.is_active(&server.id),"Minecraft exited before readiness: {:?}",events.logs.lock().unwrap().clone());
        manager.state(&server.id)=="running"
    });
    let startup_ms=began.elapsed().as_millis() as u64;
    if server.platform=="fabric" {assert!(events.contains("- lithium "),"Fabric did not report the installed Lithium mod: {:?}",events.logs.lock().unwrap().clone());}
    assert!(manager.process_snapshot(&server.id).unwrap().2.is_some());
    let response=crate::server_ping::query(([127,0,0,1],port).into()).unwrap();
    assert_eq!(response.online,Some(0));assert_eq!(response.maximum,Some(2));
    assert!(response.version.as_deref().is_some_and(|version|version.contains(&server.version)));
    if server.platform=="paper" {
        assert!(manager.tick_snapshot(&server.id).reading.is_none(),"TPS leaked from an earlier process");
        manager.request_ticks(&server.id).unwrap();
        wait_for("Paper TPS response",10,&events,||manager.tick_snapshot(&server.id).reading.is_some());
        let reading=manager.tick_snapshot(&server.id).reading.unwrap();
        assert!(!manager.tick_snapshot(&server.id).pending);
        assert!([reading.one_minute,reading.five_minutes,reading.fifteen_minutes].iter().all(|value|value.is_finite()&&*value>=0.0&&*value<=100.0));
        eprintln!("PAPER_TPS_EVIDENCE {}",serde_json::to_string(&reading).unwrap());
    }
    let marker=format!("TCH_LIFECYCLE_{}",uuid::Uuid::new_v4());
    manager.send(&server.id,&format!("say {marker}")).unwrap();
    wait_for("Minecraft command response",10,&events,||events.contains(&marker));
    manager.request_stop(&server.id,events.as_ref()).unwrap();
    wait_for("Graceful Java exit",60,&events,||!manager.is_active(&server.id)&&events.contains("Proceso finalizado"));
    assert_eq!(manager.state(&server.id),"stopped");
    assert!(manager.tick_snapshot(&server.id).reading.is_none());
    assert_eq!(*events.states.lock().unwrap(),vec!["starting","running","stopping","stopped"]);
    let exit=manager.exits()[&server.id].clone();assert!(exit.successful&&exit.was_running&&!exit.unexpected);
    let report=crate::crash_diagnostics::load(Path::new(&server.directory),&server.id,&session).unwrap().unwrap();
    assert_eq!(report.exit_code,Some(0));assert!(report.stop_requested&&report.output_complete&&!report.forced);
    (session,startup_ms)
}

#[test]
#[ignore="Downloads Java/Paper and runs Minecraft; requires a verified existing consent record in TCH_TEST_CONSENT_RECORD"]
fn paper_download_ready_command_stop_restart_backup_restore() {verify_lifecycle("paper");}

#[test]
#[ignore="Downloads Java/Fabric/Lithium and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn fabric_mod_ready_command_stop_restart_backup_restore() {verify_lifecycle("fabric");}

#[test]
#[ignore="Downloads Java/NeoForge and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn neoforge_ready_command_stop_restart_backup_restore() {verify_lifecycle("neoforge");}

#[test]
#[ignore="Downloads Java/Vanilla and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn vanilla_ready_command_stop_restart_backup_restore() {verify_lifecycle("vanilla");}

fn verify_lifecycle(platform:&str) {
    let consent_path=std::env::var("TCH_TEST_CONSENT_RECORD").expect("Provide a copy of an already recorded explicit Minecraft EULA acceptance");
    let bytes=fs::read(consent_path).unwrap();assert!(bytes.len()<=4096);
    let consent:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
    assert_eq!(consent["accepted"],true);assert_eq!(consent["document"],"https://www.minecraft.net/en-us/eula");
    chrono::DateTime::parse_from_rfc3339(consent["acceptedAt"].as_str().unwrap()).unwrap();
    let root=std::env::temp_dir().join(format!("tch-{platform}-lifecycle-{}",uuid::Uuid::new_v4()));fs::create_dir_all(&root).unwrap();
    let library=Library{root,manager:ProcessManager::default()};
    eprintln!("Temporary library: {}",library.root.display());
    // Reuse the user's existing consent; never call the acceptance command here.
    storage::connect(&library.root).unwrap().execute("INSERT INTO application_settings(key,value) VALUES(?1,?2)",rusqlite::params!["minecraft_eula_acceptance",consent.to_string()]).unwrap();
    let port=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap().local_addr().unwrap().port();
    let version=std::env::var("TCH_TEST_MINECRAFT").unwrap_or_else(|_|"1.21.4".into());
    let server=storage::create_with_seed(&library.root,CreateServer{name:format!("{platform} lifecycle verification"),version:version.clone(),platform:platform.into(),memory_min_mb:512,memory_max_mb:2048,max_players:2,port,storage_target_gb:4,image_path:None},Some("42".into())).unwrap();
    let directory=Path::new(&server.directory);
    properties::write(&directory.join("server.properties"),[
        ("server-ip".into(),"127.0.0.1".into()),("view-distance".into(),"3".into()),("simulation-distance".into(),"3".into()),
        ("generate-structures".into(),"false".into()),
    ].into()).unwrap();
    eula::require_download_consent(&library.root,&server).unwrap();
    assert!(!directory.join("world").exists());assert!(eula::file_accepted(&server));
    crate::install::install(&library.root,&server,&task(&server.id,"install")).unwrap();
    let installed=storage::get(&library.root,&server.id).unwrap();
    assert!(Path::new(installed.java_path.as_deref().unwrap()).starts_with(library.root.join("runtimes")));
    crate::runtime::validate_configured(&library.root,&installed).unwrap();
    let mod_file=if platform=="fabric" {
        let plan=crate::content::plan(&task(&server.id,"content_plan"),&installed,"lithium",None).unwrap();
        let prepared=crate::content_storage::review(&installed,plan).unwrap();
        crate::content_storage::install(&library.root,&installed,&prepared,&task(&server.id,"content_install")).unwrap();
        let files=crate::content_storage::list(&installed).unwrap();
        let lithium=files.iter().find(|file|file.project_id.is_some()).expect("No managed installed mod");
        Some((lithium.filename.clone(),fs::read(directory.join("mods").join(&lithium.filename)).unwrap()))
    } else {None};
    let first=cycle(&library,&installed,port);
    assert!(directory.join("world/level.dat").is_file());
    let second=cycle(&library,&installed,port);assert_ne!(first.0,second.0);
    let original=fs::read(directory.join("world/level.dat")).unwrap();assert!(!original.is_empty());
    let saved=backup::create_tracked(&library.root,&server.id,&task(&server.id,"backup")).unwrap();
    assert_eq!(fs::read(Path::new(&saved.path).join("world/level.dat")).unwrap(),original);
    fs::write(directory.join("world/level.dat"),b"isolated test mutation").unwrap();
    properties::write(&directory.join("server.properties"),[("motd".into(),"Changed after backup".into())].into()).unwrap();
    backup::restore_tracked(&library.root,&server.id,&saved.id,&task(&server.id,"restore")).unwrap();
    assert_eq!(fs::read(directory.join("world/level.dat")).unwrap(),original);
    assert_eq!(properties::read(&directory.join("server.properties")).unwrap()["motd"],server.name);
    let restored=storage::get(&library.root,&server.id).unwrap();
    assert_eq!(restored.version,version);assert_eq!(restored.platform,platform);assert!(eula::file_accepted(&restored));
    if let Some((filename,bytes))=&mod_file {assert_eq!(&fs::read(directory.join("mods").join(filename)).unwrap(),bytes);}
    let third=cycle(&library,&restored,port);assert_ne!(third.0,second.0);
    let backups=backup::list_details(&library.root,&server.id).unwrap();assert_eq!(backups.len(),2);
    assert!(backups.iter().any(|copy|copy.reason==Some(backup::Reason::BeforeRestore)));
    println!("LIFECYCLE_EVIDENCE {}",serde_json::json!({"platform":platform,"minecraft":version,"modLoadedInAllSessions":mod_file.is_some(),"modRestoredByteForByte":mod_file.is_some(),"runtimeDownloaded":true,"verifiedSessions":3,"startupMilliseconds":[first.1,second.1,third.1],"commandResponses":3,"gracefulStops":3,"backupBytes":saved.size_bytes,"backups":backups.len(),"worldRestoredByteForByte":true,"restoredWorldBooted":true}));
}
