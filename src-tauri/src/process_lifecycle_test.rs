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
fn cycle(library:&Library,server:&Server,port:u16,loaded_content:Option<&str>)->(String,u64) {
    let events=Arc::new(Events::default());let manager=&library.manager;
    let began=Instant::now();
    manager.launch(server,events.clone(),&crate::jvm::get(&library.root,&server.id).unwrap()).unwrap();
    let session=manager.session_token(&server.id).unwrap();
    wait_for("Minecraft readiness",180,&events,|| {
        assert!(manager.is_active(&server.id),"Minecraft exited before readiness: {:?}",events.logs.lock().unwrap().clone());
        manager.state(&server.id)=="running"
    });
    let startup_ms=began.elapsed().as_millis() as u64;
    if let Some(marker)=loaded_content {assert!(events.contains(marker),"Loader did not report the installed content: {:?}",events.logs.lock().unwrap().clone());}
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
fn paper_download_ready_command_stop_restart_backup_restore() {verify_lifecycle("paper",None);}

#[test]
#[ignore="Downloads Java/Fabric/Lithium and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn fabric_mod_ready_command_stop_restart_backup_restore() {verify_lifecycle("fabric",Some(("lithium","- lithium ")));}

#[test]
#[ignore="Downloads Java/NeoForge and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn neoforge_ready_command_stop_restart_backup_restore() {verify_lifecycle("neoforge",None);}

#[test]
#[ignore="Downloads Java/Vanilla and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn vanilla_ready_command_stop_restart_backup_restore() {verify_lifecycle("vanilla",None);}

#[test]
#[ignore="Downloads Java/Paper/Chunky and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn paper_plugin_ready_command_stop_restart_backup_restore() {verify_lifecycle("paper",Some(("chunky","[Chunky] Enabling Chunky v")));}

fn installed_library(platform:&str)->(Library,Server,u16) {
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
    (library,installed,port)
}

fn verify_lifecycle(platform:&str,content:Option<(&str,&str)>) {
    let (library,installed,port)=installed_library(platform);
    let directory=Path::new(&installed.directory);
    let server=&installed;let version=installed.version.as_str();
    let content_folder=if platform=="paper" {"plugins"} else {"mods"};
    let mod_file=if let Some((project,_))=content {
        let plan=crate::content::plan(&task(&server.id,"content_plan"),&installed,project,None).unwrap();
        let primary_project=plan.entries.iter().find(|entry|!entry.required).expect("No primary content in plan").project.id.clone();
        let prepared=crate::content_storage::review(&installed,plan).unwrap();
        crate::content_storage::install(&library.root,&installed,&prepared,&task(&server.id,"content_install")).unwrap();
        let files=crate::content_storage::list(&installed).unwrap();
        let installed_file=files.iter().find(|file|file.project_id.as_ref()==Some(&primary_project)).expect("No managed installed content");
        Some((installed_file.filename.clone(),fs::read(directory.join(content_folder).join(&installed_file.filename)).unwrap()))
    } else {None};
    let loaded_content=content.map(|(_,marker)|marker);
    let first=cycle(&library,&installed,port,loaded_content);
    assert!(directory.join("world/level.dat").is_file());
    let second=cycle(&library,&installed,port,loaded_content);assert_ne!(first.0,second.0);
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
    if let Some((filename,bytes))=&mod_file {assert_eq!(&fs::read(directory.join(content_folder).join(filename)).unwrap(),bytes);}
    let third=cycle(&library,&restored,port,loaded_content);assert_ne!(third.0,second.0);
    let backups=backup::list_details(&library.root,&server.id).unwrap();assert_eq!(backups.len(),2);
    assert!(backups.iter().any(|copy|copy.reason==Some(backup::Reason::BeforeRestore)));
    println!("LIFECYCLE_EVIDENCE {}",serde_json::json!({"platform":platform,"minecraft":version,"contentProject":content.map(|(project,_)|project),"contentFolder":content.map(|_|content_folder),"contentLoadedInAllSessions":mod_file.is_some(),"contentRestoredByteForByte":mod_file.is_some(),"runtimeDownloaded":true,"verifiedSessions":3,"startupMilliseconds":[first.1,second.1,third.1],"commandResponses":3,"gracefulStops":3,"backupBytes":saved.size_bytes,"backups":backups.len(),"worldRestoredByteForByte":true,"restoredWorldBooted":true}));
}

#[test]
#[ignore="Downloads Java/Paper and runs Minecraft; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn paper_stop_during_startup_then_copy_after_clean_running_stop() {
    let (library,server,port)=installed_library("paper");
    let manager=&library.manager;let id=&server.id;
    let jobs=Arc::new(Mutex::new(HashMap::new()));
    crate::stop_backup::configure(&library.root,id,true,None).unwrap();
    let events=Arc::new(Events::default());
    manager.launch(&server,events.clone(),&crate::jvm::get(&library.root,id).unwrap()).unwrap();
    assert_eq!(manager.state(id),"starting");
    manager.request_stop(id,events.as_ref()).unwrap();
    manager.request_stop(id,events.as_ref()).unwrap(); // Repeated stop is idempotent.
    wait_for("Paper stop requested during startup",180,&events,||!manager.is_active(id)&&events.contains("Proceso finalizado"));
    assert_eq!(manager.state(id),"stopped");
    assert_eq!(*events.states.lock().unwrap(),vec!["starting","stopping","stopped"]);
    let early_exit=manager.exits()[id].clone();
    assert!(early_exit.successful&&!early_exit.was_running&&!early_exit.unexpected);
    assert!(!crate::stop_backup::pending(&library.root,id,&early_exit).unwrap());
    assert!(!crate::stop_backup::run(&library.root,id,&early_exit,&jobs,|_|{}).unwrap());
    assert!(backup::list_details(&library.root,id).unwrap().is_empty());
    let report=crate::crash_diagnostics::load(Path::new(&server.directory),id,&early_exit.token).unwrap().unwrap();
    assert!(report.stop_requested&&report.output_complete&&!report.forced);assert_eq!(report.exit_code,Some(0));
    let (session,_)=cycle(&library,&server,port,None);
    let exit=manager.exits()[id].clone();assert_eq!(exit.token,session);
    assert!(crate::stop_backup::pending(&library.root,id,&exit).unwrap());
    assert!(crate::stop_backup::run(&library.root,id,&exit,&jobs,|_|{}).unwrap());
    let copies=backup::list_details(&library.root,id).unwrap();assert_eq!(copies.len(),1);
    assert_eq!(copies[0].reason,Some(backup::Reason::ServerStop));
    let world=Path::new(&server.directory).join("world/level.dat");
    assert_eq!(fs::read(world).unwrap(),fs::read(Path::new(&copies[0].backup.path).join("world/level.dat")).unwrap());
    assert_eq!(crate::stop_backup::status(&library.root,id).unwrap().phase,"completed");
    crate::stop_backup::recover(&library.root).unwrap();
    assert!(!crate::stop_backup::pending(&library.root,id,&exit).unwrap());
    assert!(!crate::stop_backup::run(&library.root,id,&exit,&jobs,|_|{}).unwrap());
    assert_eq!(backup::list_details(&library.root,id).unwrap().len(),1);
    assert!(jobs.lock().unwrap().is_empty());
    println!("PAPER_STOP_COPY_EVIDENCE startupStop=graceful readyAfterStop=false cleanRunningStopCopy=1 replayed=false levelDatMatches=true");
}

#[test]
#[ignore="Downloads Java/Vanilla and runs two Minecraft instances; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn two_real_servers_keep_processes_commands_and_stops_independent() {
    let (library,first,first_port)=installed_library("vanilla");
    let listener=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap();
    let second_port=listener.local_addr().unwrap().port();assert_ne!(first_port,second_port);
    let second=storage::create_with_seed(&library.root,CreateServer{name:"Second isolated server".into(),version:first.version.clone(),platform:"vanilla".into(),memory_min_mb:512,memory_max_mb:2048,max_players:3,port:second_port,storage_target_gb:4,image_path:None},Some("84".into())).unwrap();
    properties::write(&Path::new(&second.directory).join("server.properties"),[
        ("server-ip".into(),"127.0.0.1".into()),("view-distance".into(),"3".into()),
        ("simulation-distance".into(),"3".into()),("generate-structures".into(),"false".into()),
    ].into()).unwrap();
    eula::require_download_consent(&library.root,&second).unwrap();
    crate::install::install(&library.root,&second,&task(&second.id,"install")).unwrap();
    let second=storage::get(&library.root,&second.id).unwrap();
    assert_ne!(first.id,second.id);assert_ne!(first.directory,second.directory);
    assert_eq!(first.java_path,second.java_path); // The managed runtime is safely shared.
    assert_ne!(first.jar_path,second.jar_path);
    drop(listener);
    let manager=&library.manager;let a=Arc::new(Events::default());let b=Arc::new(Events::default());
    manager.launch(&first,a.clone(),&crate::jvm::get(&library.root,&first.id).unwrap()).unwrap();
    manager.launch(&second,b.clone(),&crate::jvm::get(&library.root,&second.id).unwrap()).unwrap();
    assert!(manager.launch(&first,a.clone(),&crate::jvm::get(&library.root,&first.id).unwrap()).is_err());
    for(server,events)in [(&first,&a),(&second,&b)] {
        wait_for("Simultaneous Minecraft readiness",180,events,||{
            assert!(manager.is_active(&server.id),"Owned Java exited before readiness");
            manager.state(&server.id)=="running"
        });
    }
    assert_eq!(manager.active_ids().len(),2);
    assert_ne!(manager.process_snapshot(&first.id).unwrap().0,manager.process_snapshot(&second.id).unwrap().0);
    let second_session=manager.session_token(&second.id).unwrap();
    for(port,maximum)in [(first_port,2),(second_port,3)] {
        let status=crate::server_ping::query(([127,0,0,1],port).into()).unwrap();
        assert_eq!(status.online,Some(0));assert_eq!(status.maximum,Some(maximum));
    }
    let marker_a=format!("TCH_FIRST_{}",uuid::Uuid::new_v4());let marker_b=format!("TCH_SECOND_{}",uuid::Uuid::new_v4());
    manager.send(&first.id,&format!("say {marker_a}")).unwrap();manager.send(&second.id,&format!("say {marker_b}")).unwrap();
    wait_for("First independent console",10,&a,||a.contains(&marker_a));
    wait_for("Second independent console",10,&b,||b.contains(&marker_b));
    manager.request_stop(&first.id,a.as_ref()).unwrap();
    wait_for("First independent stop",60,&a,||!manager.is_active(&first.id)&&a.contains("Proceso finalizado"));
    assert_eq!(manager.state(&first.id),"stopped");assert_eq!(manager.state(&second.id),"running");
    assert_eq!(manager.session_token(&second.id).as_deref(),Some(second_session.as_str()));
    let surviving_marker=format!("TCH_STILL_RUNNING_{}",uuid::Uuid::new_v4());
    manager.send(&second.id,&format!("say {surviving_marker}")).unwrap();
    wait_for("Surviving server command",10,&b,||b.contains(&surviving_marker));
    assert_eq!(crate::server_ping::query(([127,0,0,1],second_port).into()).unwrap().maximum,Some(3));
    manager.request_stop(&second.id,b.as_ref()).unwrap();
    wait_for("Second independent stop",60,&b,||!manager.is_active(&second.id)&&b.contains("Proceso finalizado"));
    assert!(!manager.any_active());assert!(!a.contains(&marker_b)&&!a.contains(&surviving_marker)&&!b.contains(&marker_a));
    for(server,events)in [(&first,&a),(&second,&b)] {
        assert_eq!(*events.states.lock().unwrap(),vec!["starting","running","stopping","stopped"]);
        let exit=manager.exits()[&server.id].clone();assert!(exit.successful&&exit.was_running&&!exit.unexpected);
        let report=crate::crash_diagnostics::load(Path::new(&server.directory),&server.id,&exit.token).unwrap().unwrap();
        assert_eq!(report.exit_code,Some(0));assert!(report.stop_requested&&report.output_complete&&!report.forced);
        let log=fs::read_to_string(Path::new(&server.directory).join("launcher-logs/latest.log")).unwrap();
        assert!(log.contains(if server.id==first.id {&marker_a}else{&marker_b}));
        assert!(!log.contains(if server.id==first.id {&marker_b}else{&marker_a}));
    }
    assert_eq!(properties::read(&Path::new(&first.directory).join("server.properties")).unwrap()["level-seed"],"42");
    assert_eq!(properties::read(&Path::new(&second.directory).join("server.properties")).unwrap()["level-seed"],"84");
    println!("MULTI_SERVER_EVIDENCE simultaneous=2 distinctPids=true distinctDirectories=true separateLogs=true survivingCommand=true gracefulStops=2");
}


#[test]
#[ignore="Downloads Java/Vanilla and upgrades a real world; requires verified existing consent in TCH_TEST_CONSENT_RECORD"]
fn vanilla_generated_world_upgrade_and_full_world_rollback() {
    use sha2::{Digest,Sha256};
    use std::{collections::BTreeMap,io::Read};
    let world_hashes=|base:&Path| {
        let mut files=BTreeMap::new();let mut buffer=[0u8;65536];
        for entry in walkdir::WalkDir::new(base) {
            let entry=entry.unwrap();assert!(!entry.file_type().is_symlink());
            if !entry.file_type().is_file(){continue;}
            let mut input=fs::File::open(entry.path()).unwrap();let mut hash=Sha256::new();
            loop {let count=input.read(&mut buffer).unwrap();if count==0{break;}hash.update(&buffer[..count]);}
            files.insert(entry.path().strip_prefix(base).unwrap().to_path_buf(),format!("{:x}",hash.finalize()));
        }
        files
    };
    let (library,before,port)=installed_library("vanilla");
    assert_eq!(before.version,"1.21.4","This conversion fixture starts with Minecraft 1.21.4");
    let directory=Path::new(&before.directory);let world=directory.join("world");
    properties::write(&directory.join("server.properties"),[("difficulty".into(),"hard".into())].into()).unwrap();
    let custom=crate::jvm::Settings{mode:"custom".into(),arguments:"-Dfile.encoding=UTF-8".into()};
    crate::jvm::save(&library.root,&before.id,&custom).unwrap();
    let original_session=cycle(&library,&before,port,None);
    let original=world_hashes(&world);
    assert!(original.contains_key(Path::new("level.dat")));
    assert!(original.keys().any(|path|path.extension().is_some_and(|extension|extension=="mca")),"No generated region files");
    let old_jar=fs::read(before.jar_path.as_ref().unwrap()).unwrap();
    let config=fs::read(directory.join("server.properties")).unwrap();
    let target="1.21.5";let operation=task(&before.id,"minecraft_update");
    let reviewed=crate::minecraft_update::review(&library.root,&before,target,&operation).unwrap();
    assert!(reviewed.blockers.is_empty());assert_eq!(reviewed.java_action,"retain_configured");
    assert!(!reviewed.warnings.is_empty());
    assert!(crate::minecraft_update::update(&library.root,&before.id,target,&reviewed.fingerprint,false,&operation).is_err());
    assert!(backup::list_details(&library.root,&before.id).unwrap().is_empty());
    assert_eq!(world_hashes(&world),original,"Unconfirmed conversion changed the world");
    eula::require_download_consent(&library.root,&before).unwrap();
    crate::minecraft_update::update(&library.root,&before.id,target,&reviewed.fingerprint,true,&operation).unwrap();
    let updated=storage::get(&library.root,&before.id).unwrap();
    assert_eq!(updated.version,target);assert_eq!(updated.java_path,before.java_path);
    assert_eq!(crate::jvm::get(&library.root,&before.id).unwrap(),custom);
    assert_eq!(fs::read(directory.join("server.properties")).unwrap(),config);
    assert_eq!(world_hashes(&world),original,"Publishing the new engine changed the unbooted world");
    let copies=backup::list_details(&library.root,&before.id).unwrap();assert_eq!(copies.len(),1);
    let saved=&copies[0];assert_eq!(saved.reason,Some(backup::Reason::BeforeMinecraftUpdate));
    assert_eq!(saved.minecraft_version.as_deref(),Some("1.21.4"));assert!(saved.metadata_error.is_none());
    assert_eq!(world_hashes(&Path::new(&saved.backup.path).join("world")),original);
    crate::runtime::validate_configured(&library.root,&updated).unwrap();
    let upgrade_session=cycle(&library,&updated,port,None);assert_ne!(original_session.0,upgrade_session.0);
    let converted=world_hashes(&world);assert_ne!(converted.get(Path::new("level.dat")),original.get(Path::new("level.dat")));
    backup::restore_tracked(&library.root,&before.id,&saved.backup.id,&task(&before.id,"restore")).unwrap();
    let restored=storage::get(&library.root,&before.id).unwrap();
    assert_eq!(restored.version,"1.21.4");assert!(restored.java_path.is_none());
    assert_eq!(world_hashes(&world),original,"Rollback did not restore every world file");
    assert_eq!(fs::read(restored.jar_path.as_ref().unwrap()).unwrap(),old_jar);
    assert_eq!(fs::read(directory.join("server.properties")).unwrap(),config);
    assert_eq!(crate::jvm::get(&library.root,&before.id).unwrap(),custom);
    let copies=backup::list_details(&library.root,&before.id).unwrap();assert_eq!(copies.len(),2);
    let safety=copies.iter().find(|copy|copy.reason==Some(backup::Reason::BeforeRestore)).unwrap();
    assert_eq!(safety.minecraft_version.as_deref(),Some(target));
    assert_eq!(world_hashes(&Path::new(&safety.backup.path).join("world")),converted);
    // Use the normal managed-runtime preparation after a cross-version restore.
    crate::install::prepare_runtime(&library.root,&restored,&task(&before.id,"runtime"),false).unwrap();
    let restored=storage::get(&library.root,&before.id).unwrap();
    crate::runtime::validate_configured(&library.root,&restored).unwrap();
    let rollback_session=cycle(&library,&restored,port,None);assert_ne!(rollback_session.0,upgrade_session.0);
    let (_,stage,old,journal)=crate::instance_publication::paths(&library.root,&restored).unwrap();
    assert!(!stage.exists()&&!old.exists()&&!journal.exists());assert!(eula::recorded(&library.root).unwrap());
    println!("WORLD_UPGRADE_EVIDENCE {}",serde_json::json!({"from":before.version,"to":target,"worldFiles":original.len(),"allWorldHashesRestored":true,"convertedSafetyCopyPreserved":true,"originalJarRestored":true,"verifiedSessions":3,"startupMilliseconds":[original_session.1,upgrade_session.1,rollback_session.1],"runtimeReverified":true,"backups":copies.len()}));
}
