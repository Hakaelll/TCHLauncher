use crate::model::Progress;
use reqwest::{Client, Response};
use serde_json::Value;
use sha1::{Digest as _, Sha1};
use sha2::{Sha256, Sha512};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use uuid::Uuid;

pub const CANCELLED: &str = "Instalación cancelada";
const MAX_BYTES: u64 = 2_000_000_000;
const ATTEMPTS: u32 = 3;

#[derive(Default)]
pub struct Cancellation {
    requested: AtomicBool,
    notification: Notify,
}

impl Cancellation {
    pub fn cancel(&self) { self.requested.store(true, Ordering::SeqCst); self.notification.notify_one(); }
    pub fn is_cancelled(&self) -> bool { self.requested.load(Ordering::SeqCst) }
    pub fn check(&self) -> Result<(), String> { if self.is_cancelled() { Err(CANCELLED.into()) } else { Ok(()) } }
    async fn wait(&self) {
        if !self.is_cancelled() { self.notification.notified().await; }
    }
}

pub struct DownloadTask {
    pub id: String,
    pub cancellation: Arc<Cancellation>,
    server_id: String,
    kind: String,
    client: Client,
    max_bytes:u64,
    emit: Arc<dyn Fn(Progress) + Send + Sync>,
    latest: Arc<Mutex<Progress>>,
}

impl DownloadTask {
    pub fn new(server_id: &str, kind: &str, emit: impl Fn(Progress) + Send + Sync + 'static) -> Result<Self, String> {
        let client = Client::builder()
            .user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION")," (https://github.com/Hakaelll/TCHLauncher)"))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.url().scheme() != "https" { attempt.error("HTTPS required") }
                else if attempt.previous().len() > 10 { attempt.error("too many redirects") }
                else { attempt.follow() }
            }))
            .connect_timeout(Duration::from_secs(if kind=="content_search" {8}else{20}))
            .read_timeout(Duration::from_secs(if kind=="content_search" {12}else{30}))
            .timeout(Duration::from_secs(if kind=="content_search" {20}else{900})).build().map_err(|e| e.to_string())?;
        let id=Uuid::new_v4().to_string();
        let latest=Mutex::new(Progress{server_id:server_id.into(),task_id:id.clone(),kind:kind.into(),label:"Preparando tarea".into(),stage:"preparing".into(),received:0,total:None,bytes_per_second:None,attempt:1,error:None});
        Ok(Self { id,server_id:server_id.into(),kind:kind.into(),client,max_bytes:MAX_BYTES,
            cancellation:Arc::new(Cancellation::default()),emit:Arc::new(emit),latest:Arc::new(latest) })
    }

    pub fn report(&self, label: &str, stage: &str) { self.progress(label,stage,0,None,None,1,None); }

    pub(crate) fn progress(&self,label: &str,stage: &str,received: u64,total: Option<u64>,speed: Option<f64>,attempt: u32,error: Option<String>) {
        let progress=Progress {server_id:self.server_id.clone(),task_id:self.id.clone(),kind:self.kind.clone(),label:label.into(),stage:stage.into(),
            received,total,bytes_per_second:speed,attempt,error};
        if let Ok(mut latest)=self.latest.lock(){*latest=progress.clone();}
        (self.emit)(progress);
    }

    pub fn snapshot(&self)->Result<Progress,String>{self.latest.lock().map(|value|value.clone()).map_err(|_|"No se pudo leer el estado de la tarea".into())}
    pub(crate) fn kind(&self)->&str{&self.kind}
    pub fn is_file_operation(&self)->bool{matches!(self.kind.as_str(),"backup"|"restore"|"world_import"|"world_duplicate"|"world_export"|"world_activate"|"world_delete"|"server_import"|"content_remove"|"content_toggle")}
    pub fn is_restore(&self)->bool{self.kind=="restore"}
    pub fn request_cancel(&self)->bool{
        let Ok(latest)=self.latest.lock()else{return false;};
        if matches!(latest.stage.as_str(),"completed"|"cancelled"|"failed"|"publishing"){return false;}
        self.cancellation.cancel();true
    }
    pub fn begin_publication(&self,label:&str)->Result<(),String>{
        let mut latest=self.latest.lock().map_err(|_|"No se pudo publicar la tarea")?;
        self.cancellation.check()?;
        latest.label=label.into();latest.stage="publishing".into();latest.received=0;latest.total=None;latest.bytes_per_second=None;
        let progress=latest.clone();drop(latest);(self.emit)(progress);Ok(())
    }

    pub fn finish(&self, result: &Result<(), String>) {
        if matches!(self.kind.as_str(),"world_activate"|"world_delete"){
            let (success,cancelled,failed)=if self.kind=="world_delete"{("Mundo eliminado","Eliminación cancelada","No se pudo eliminar el mundo")}else{("Mundo principal actualizado","Cambio de mundo cancelado","No se pudo cambiar el mundo principal")};
            match result{
                Ok(())=>self.report(success,"completed"),
                Err(error) if error==CANCELLED=>self.report(cancelled,"cancelled"),
                Err(error)=>self.progress(failed,"failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if matches!(self.kind.as_str(),"world_duplicate"|"world_export"){
            let (success,cancelled,failed)=if self.kind=="world_export"{("Exportación completada","Exportación cancelada","No se pudo exportar el mundo")}else{("Mundo duplicado","Duplicado cancelado","No se pudo duplicar el mundo")};
            match result{
                Ok(())=>self.report(success,"completed"),
                Err(error) if error==CANCELLED=>self.report(cancelled,"cancelled"),
                Err(error)=>self.progress(failed,"failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="content_remove"{
            match result{
                Ok(())=>self.report("Contenido retirado","completed"),
                Err(error) if error==CANCELLED=>self.report("Retirada cancelada","cancelled"),
                Err(error)=>self.progress("No se pudo retirar el contenido","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="content_toggle"{
            match result{
                Ok(())=>self.report("Estado del contenido actualizado","completed"),
                Err(error) if error==CANCELLED=>self.report("Cambio de contenido cancelado","cancelled"),
                Err(error)=>self.progress("No se pudo cambiar el estado del contenido","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="server_import"{
            match result{
                Ok(())=>self.report("Servidor importado","completed"),
                Err(error) if error==CANCELLED=>self.report("Importación cancelada","cancelled"),
                Err(error)=>self.progress("No se pudo importar el servidor","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="world_import"{
            match result{
                Ok(())=>self.report("Mundo importado","completed"),
                Err(error) if error==CANCELLED=>self.report("Importación cancelada","cancelled"),
                Err(error)=>self.progress("No se pudo importar el mundo","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="restore"{
            match result{
                Ok(())=>self.report("Copia restaurada","completed"),
                Err(error) if error==CANCELLED=>self.report("Restauración cancelada","cancelled"),
                Err(error)=>self.progress("No se pudo restaurar la copia","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        if self.kind=="backup" {
            match result {
                Ok(())=>self.report("Copia guardada","completed"),
                Err(error) if error==CANCELLED=>self.report("Copia cancelada","cancelled"),
                Err(error)=>self.progress("No se pudo guardar la copia","failed",0,None,None,1,Some(error.clone())),
            }
            return;
        }
        match result {
            Ok(()) => self.report(if self.kind == "install" {"Instalación completada"} else if self.kind == "content_install" {"Contenido instalado"} else if self.kind=="pack_install" {"Modpack instalado"} else if self.kind=="loader_update" {"Loader actualizado"} else if self.kind=="minecraft_update" {"Minecraft actualizado"} else {"Java preparado"},"completed"),
            Err(error) if error == CANCELLED => self.report("Instalación cancelada","cancelled"),
            Err(error) => self.progress("No se pudo instalar","failed",0,None,None,1,Some(error.clone())),
        }
    }

    pub fn json(&self, url: &str, label: &str) -> Result<Value,String> {
        validate_url(url)?;
        tauri::async_runtime::block_on(async {
            for attempt in 1..=ATTEMPTS {
                self.cancellation.check()?;
                self.progress(label,"preparing",0,None,None,attempt,None);
                let result = async {
                    let mut response = self.request(url).await?;
                    let mut body = Vec::new();
                    while let Some(chunk) = self.chunk(&mut response).await? {
                        if body.len() + chunk.len() > 5_000_000 { return Err(Failure::local("La respuesta de versiones supera 5 MB")); }
                        body.extend_from_slice(&chunk);
                    }
                    serde_json::from_slice(&body).map_err(|_| Failure::local("La fuente devolvió datos de versiones no válidos"))
                }.await;
                match result {
                    Ok(value) => return Ok(value),
                    Err(error) if error.retryable && attempt < ATTEMPTS => self.retry(label,attempt,error.message).await?,
                    Err(error) => return Err(error.message),
                }
            }
            unreachable!()
        })
    }

    pub fn text(&self, url: &str, label: &str) -> Result<String,String> {
        validate_url(url)?;
        tauri::async_runtime::block_on(async {
            for attempt in 1..=ATTEMPTS {
                self.cancellation.check()?;
                self.progress(label,"preparing",0,None,None,attempt,None);
                let result = async {
                    let mut response = self.request(url).await?;
                    let mut body = Vec::new();
                    while let Some(chunk) = self.chunk(&mut response).await? {
                        if body.len() + chunk.len() > 4096 { return Err(Failure::local("La respuesta de checksum supera 4 KB")); }
                        body.extend_from_slice(&chunk);
                    }
                    String::from_utf8(body).map_err(|_| Failure::local("El checksum no contiene texto válido"))
                }.await;
                match result {
                    Ok(value) => return Ok(value),
                    Err(error) if error.retryable && attempt < ATTEMPTS => self.retry(label,attempt,error.message).await?,
                    Err(error) => return Err(error.message),
                }
            }
            unreachable!()
        })
    }

    pub fn download(&self,url: &str,target: &Path,hash: Option<(&str,&str)>,label: &str) -> Result<(),String> {
        validate_url(url)?;
        tauri::async_runtime::block_on(self.transfer(url,target,hash,label))
    }

    pub fn download_pack(&self,url:&str,target:&Path,sha512:&str,size:u64,label:&str)->Result<(),String> {
        validate_pack_url(url,false)?;
        if size==0 || size>512_000_000 {return Err("Tamaño de archivo del modpack no válido".into());}
        let client=Client::builder()
            .user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len()>=10 {attempt.error("Demasiadas redirecciones")}
                else if validate_pack_url(attempt.url().as_str(),true).is_err() {attempt.error("Origen de redirección no admitido")}
                else {attempt.follow()}
            }))
            .connect_timeout(Duration::from_secs(20)).read_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(900)).build().map_err(|e|e.to_string())?;
        let scoped=Self {id:self.id.clone(),server_id:self.server_id.clone(),kind:self.kind.clone(),client,max_bytes:size,
            cancellation:self.cancellation.clone(),emit:self.emit.clone(),latest:self.latest.clone()};
        scoped.download(url,target,Some(("sha512",sha512)),label)
    }

    async fn request(&self,url: &str) -> Result<Response,Failure> {
        let response = tokio::select! {
            _ = self.cancellation.wait() => return Err(Failure::local(CANCELLED)),
            result = self.client.get(url).send() => result.map_err(Failure::http)?,
        };
        response.error_for_status().map_err(Failure::http)
    }

    async fn chunk(&self,response: &mut Response) -> Result<Option<Vec<u8>>,Failure> {
        tokio::select! {
            _ = self.cancellation.wait() => Err(Failure::local(CANCELLED)),
            result = response.chunk() => result.map(|chunk| chunk.map(|bytes| bytes.to_vec())).map_err(Failure::http),
        }
    }

    async fn retry(&self,label: &str,attempt: u32,error: String) -> Result<(),String> {
        self.progress(label,"retrying",0,None,None,attempt,Some(error));
        tokio::select! {
            _ = self.cancellation.wait() => Err(CANCELLED.into()),
            _ = tokio::time::sleep(Duration::from_secs(u64::from(attempt))) => Ok(()),
        }
    }

    async fn transfer(&self,url: &str,target: &Path,hash: Option<(&str,&str)>,label: &str) -> Result<(),String> {
        for attempt in 1..=ATTEMPTS {
            self.cancellation.check()?;
            self.progress(label,"connecting",0,None,None,attempt,None);
            let result = self.transfer_once(url,target,hash,label,attempt).await;
            match result {
                Ok(()) => return Ok(()),
                Err(error) if error.retryable && attempt < ATTEMPTS => self.retry(label,attempt,error.message).await?,
                Err(error) => return Err(error.message),
            }
        }
        unreachable!()
    }

    async fn transfer_once(&self,url: &str,target: &Path,hash: Option<(&str,&str)>,label: &str,attempt: u32) -> Result<(),Failure> {
        let mut response = self.request(url).await?;
        let total = response.content_length();
        if total.is_some_and(|bytes| bytes > self.max_bytes) { return Err(Failure::local("La descarga supera el tamaño permitido")); }
        let part = PartialFile(target.with_extension(format!("part-{}",self.id)));
        let mut file = File::create(&part.0).map_err(Failure::io)?;
        let mut sha1 = Sha1::new();
        let mut sha256 = Sha256::new();
        let mut sha512 = Sha512::new();
        let mut received = 0u64;
        let mut last = Instant::now();
        let mut last_bytes = 0;
        self.progress(label,"downloading",0,total,None,attempt,None);
        while let Some(chunk) = self.chunk(&mut response).await? {
            received += chunk.len() as u64;
            if received > self.max_bytes { return Err(Failure::local("La descarga supera el tamaño permitido")); }
            file.write_all(&chunk).map_err(Failure::io)?;
            sha1.update(&chunk); sha256.update(&chunk); sha512.update(&chunk);
            let elapsed = last.elapsed().as_secs_f64();
            if elapsed >= 0.2 {
                self.progress(label,"downloading",received,total,Some((received-last_bytes) as f64 / elapsed),attempt,None);
                last = Instant::now(); last_bytes = received;
            }
        }
        self.cancellation.check().map_err(Failure::local)?;
        if total.is_some_and(|bytes| bytes != received) {
            return Err(Failure {message:"La descarga terminó antes de recibir todos los bytes".into(),retryable:true});
        }
        if received == 0 { return Err(Failure::local("La descarga está vacía")); }
        self.progress(label,"verifying",received,total,None,attempt,None);
        if let Some((kind,expected)) = hash {
            let actual = match kind {
                "sha1" => hex(&sha1.finalize()), "sha256" => hex(&sha256.finalize()),
                "sha512" => hex(&sha512.finalize()),
                _ => return Err(Failure::local("Algoritmo de hash no compatible")),
            };
            if !actual.eq_ignore_ascii_case(expected) { return Err(Failure::local(format!("La suma de comprobación de {label} no coincide"))); }
        }
        file.sync_all().map_err(Failure::io)?;
        drop(file);
        self.cancellation.check().map_err(Failure::local)?;
        replace_file(&part.0,target).map_err(Failure::local)
    }
}

struct Failure { message: String, retryable: bool }
impl Failure {
    fn local(message: impl Into<String>) -> Self { Self {message:message.into(),retryable:false} }
    fn io(error: std::io::Error) -> Self { Self::local(format!("No se pudo guardar la descarga: {error}")) }
    fn http(error: reqwest::Error) -> Self {
        let retryable = error.is_timeout() || error.is_connect() || error.is_body() || error.status().is_some_and(|s| s.is_server_error() || s.as_u16() == 408 || s.as_u16() == 429);
        Self {message:format!("No se pudo completar la descarga: {error}"),retryable}
    }
}

struct PartialFile(PathBuf);
impl Drop for PartialFile { fn drop(&mut self) { let _ = fs::remove_file(&self.0); } }

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

fn validate_url(url: &str) -> Result<(),String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "La URL de descarga no es válida")?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("La descarga debe usar HTTPS sin credenciales".into());
    }
    Ok(())
}

pub fn validate_pack_url(raw:&str,redirect:bool)->Result<(),String> {
    validate_url(raw)?;
    let url=reqwest::Url::parse(raw).map_err(|e|e.to_string())?;
    let initial=matches!(url.host_str(),Some("cdn.modrinth.com"|"github.com"|"raw.githubusercontent.com"|"gitlab.com"));
    let release_asset=redirect && matches!(url.host_str(),Some("release-assets.githubusercontent.com"|"objects.githubusercontent.com"));
    if url.port().is_some() || raw.chars().any(|c|c.is_whitespace()||c.is_control()) || !(initial||release_asset) {
        return Err("Origen de descarga del modpack no admitido".into());
    }
    Ok(())
}

pub fn replace_file(part: &Path,target: &Path) -> Result<(),String> {
    if !target.exists() { return fs::rename(part,target).map_err(|e| e.to_string()); }
    let previous = target.with_extension(format!("previous-{}",Uuid::new_v4()));
    fs::rename(target,&previous).map_err(|e| e.to_string())?;
    if let Err(error) = fs::rename(part,target) {
        fs::rename(&previous,target).map_err(|rollback| format!("No se pudo instalar ({error}) ni recuperar {previous:?}: {rollback}"))?;
        return Err(error.to_string());
    }
    let _ = fs::remove_file(previous);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read,Write};
    use std::net::TcpListener;
    use std::sync::Mutex;

    struct TestFolder(PathBuf);
    impl TestFolder {
        fn new() -> Self {let path=std::env::temp_dir().join(format!("tch-download-test-{}",Uuid::new_v4()));fs::create_dir_all(&path).unwrap();Self(path)}
        fn target(&self) -> PathBuf {self.0.join("server.jar")}
        fn no_partials(&self) -> bool {fs::read_dir(&self.0).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().contains("part-"))}
    }
    impl Drop for TestFolder {fn drop(&mut self) {let _=fs::remove_dir_all(&self.0);}}

    fn task(events: Arc<Mutex<Vec<Progress>>>) -> DownloadTask {
        let mut task=DownloadTask::new("test","install",move |event|events.lock().unwrap().push(event)).unwrap();
        // Only the private transport is exercised over loopback; the public entry point still requires HTTPS.
        task.client=Client::builder().no_proxy().timeout(Duration::from_secs(4)).build().unwrap();
        task
    }

    #[test]
    fn snapshot_retains_measured_progress_and_terminal_outcomes() {
        let events=Arc::new(Mutex::new(Vec::new()));let task=task(events.clone());
        let initial=task.snapshot().unwrap();assert_eq!(initial.task_id,task.id);assert_eq!(initial.stage,"preparing");assert!(initial.total.is_none());
        task.progress("Java","downloading",32,Some(100),Some(8.0),2,None);
        let active=task.snapshot().unwrap();assert_eq!(active.received,32);assert_eq!(active.total,Some(100));assert_eq!(active.bytes_per_second,Some(8.0));assert_eq!(active.attempt,2);
        task.finish(&Err(CANCELLED.into()));let terminal=task.snapshot().unwrap();assert_eq!(terminal.stage,"cancelled");assert_eq!(terminal.task_id,task.id);assert_eq!(events.lock().unwrap().last().unwrap().stage,terminal.stage);
    }

    #[test]
    fn file_publication_and_cancellation_have_an_exclusive_boundary(){
      for kind in ["backup","restore","minecraft_update","loader_update","content_install","world_import","world_duplicate","world_export","world_activate","world_delete","server_import","content_remove","content_toggle"]{
        let cancelled=DownloadTask::new("test",kind,|_|{}).unwrap();
        assert!(cancelled.request_cancel());assert_eq!(cancelled.begin_publication("Guardando").unwrap_err(),CANCELLED);
        let published=DownloadTask::new("test",kind,|_|{}).unwrap();
        published.begin_publication("Guardando").unwrap();assert!(!published.request_cancel());assert!(!published.cancellation.is_cancelled());
        published.finish(&Ok(()));assert_eq!(published.snapshot().unwrap().stage,"completed");assert!(!published.request_cancel());
      }
    }

    fn fixture(responses: Vec<&'static str>) -> (String,std::thread::JoinHandle<()>) {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let url=format!("http://{}/artifact",listener.local_addr().unwrap());
        let worker=std::thread::spawn(move || {
            for response in responses {
                let (mut stream,_)=listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut request=[0;2048];let _=stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (url,worker)
    }

    #[test]
    fn rejects_insecure_or_credential_bearing_download_urls() {
        for url in ["http://example.com/file","https://user:secret@example.com/file","not a url"] {assert!(validate_url(url).is_err());}
        assert!(validate_url("https://example.com/file").is_ok());
    }

    #[test]
    fn pack_sources_and_redirects_use_explicit_hosts() {
        assert!(validate_pack_url("https://cdn.modrinth.com/data/file.jar",false).is_ok());
        assert!(validate_pack_url("https://github.com/project/releases/file.jar",false).is_ok());
        assert!(validate_pack_url("https://release-assets.githubusercontent.com/file.jar",true).is_ok());
        assert!(validate_pack_url("https://release-assets.githubusercontent.com/file.jar",false).is_err());
        for url in ["http://github.com/file","https://github.com:8443/file","https://github.com.evil.test/file","https://127.0.0.1/file","https://user@cdn.modrinth.com/file","https://example.com/file"] {assert!(validate_pack_url(url,true).is_err());}
    }

    #[test]
    fn scoped_download_limit_stops_chunked_body_and_preserves_old_file() {
        let folder=TestFolder::new();fs::write(folder.target(),b"original").unwrap();
        let (url,worker)=fixture(vec!["HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nnew\r\n0\r\n\r\n"]);
        let mut task=task(Arc::new(Mutex::new(Vec::new())));task.max_bytes=2;
        assert!(tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),None,"Pack")).unwrap_err().contains("tamaño"));worker.join().unwrap();
        assert_eq!(fs::read(folder.target()).unwrap(),b"original");assert!(folder.no_partials());
    }

    #[test]
    fn checksum_failure_preserves_installed_file_and_removes_partial() {
        let folder=TestFolder::new();fs::write(folder.target(),"previous valid server").unwrap();
        let (url,worker)=fixture(vec!["HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nnew"]);
        let task=task(Arc::new(Mutex::new(Vec::new())));
        let result=tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),Some(("sha256","invalid checksum")),"Server"));
        worker.join().unwrap();
        assert!(result.unwrap_err().contains("comprobación"));
        assert_eq!(fs::read_to_string(folder.target()).unwrap(),"previous valid server");
        assert!(folder.no_partials());
    }

    #[test]
    fn retries_transient_server_error_then_publishes_verified_artifact() {
        let folder=TestFolder::new();
        let events=Arc::new(Mutex::new(Vec::new()));let task=task(events.clone());
        let (url,worker)=fixture(vec!["HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n","HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nnew"]);
        let checksum=hex(&Sha256::digest(b"new"));
        tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),Some(("sha256",&checksum)),"Server")).unwrap();
        worker.join().unwrap();
        assert_eq!(fs::read(folder.target()).unwrap(),b"new");assert!(folder.no_partials());
        let events=events.lock().unwrap();
        assert!(events.iter().any(|event|event.stage=="retrying"));
        assert!(events.iter().any(|event|event.stage=="verifying" && event.received==3 && event.attempt==2));
    }

    #[test]
    fn cancelling_a_stalled_body_is_prompt_and_preserves_installed_file() {
        let folder=TestFolder::new();fs::write(folder.target(),b"original").unwrap();
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let url=format!("http://{}/artifact",listener.local_addr().unwrap());
        let (headers_sent,headers_ready)=std::sync::mpsc::channel();
        let (finished,done)=std::sync::mpsc::channel();
        let worker=std::thread::spawn(move || {
            let (mut stream,_)=listener.accept().unwrap();let mut request=[0;2048];let _=stream.read(&mut request);
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\nConnection: close\r\n\r\nabc").unwrap();
            headers_sent.send(()).unwrap();let _=done.recv_timeout(Duration::from_secs(4));
        });
        let task=task(Arc::new(Mutex::new(Vec::new())));let cancellation=task.cancellation.clone();
        let cancel_worker=std::thread::spawn(move || {headers_ready.recv_timeout(Duration::from_secs(3)).unwrap();std::thread::sleep(Duration::from_millis(100));cancellation.cancel();});
        let started=Instant::now();
        let result=tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),None,"Server"));
        assert_eq!(result.unwrap_err(),CANCELLED);assert!(started.elapsed()<Duration::from_secs(2));
        let _=finished.send(());worker.join().unwrap();cancel_worker.join().unwrap();
        assert_eq!(fs::read(folder.target()).unwrap(),b"original");assert!(folder.no_partials());
    }

    #[test]
    fn a_permanent_http_error_does_not_retry() {
        let folder=TestFolder::new();let events=Arc::new(Mutex::new(Vec::new()));let task=task(events.clone());
        let (url,worker)=fixture(vec!["HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"]);
        assert!(tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),None,"Server")).is_err());worker.join().unwrap();
        assert!(!events.lock().unwrap().iter().any(|event|event.stage=="retrying"));assert!(folder.no_partials());
    }

    #[test]
    fn verifies_modrinth_sha512_before_publishing() {
        let folder=TestFolder::new();
        let (url,worker)=fixture(vec!["HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nnew"]);
        let task=task(Arc::new(Mutex::new(Vec::new())));
        let checksum=hex(&Sha512::digest(b"new"));
        tauri::async_runtime::block_on(task.transfer(&url,&folder.target(),Some(("sha512",&checksum)),"Mod")).unwrap();
        worker.join().unwrap();
        assert_eq!(fs::read(folder.target()).unwrap(),b"new");assert!(folder.no_partials());
    }
}
