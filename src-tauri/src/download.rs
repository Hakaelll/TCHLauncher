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

fn shared_client(kind:&str)->Result<Client,String> {
    static SEARCH:std::sync::OnceLock<Client>=std::sync::OnceLock::new();
    static TRANSFER:std::sync::OnceLock<Client>=std::sync::OnceLock::new();
    let shared=if kind=="content_search" {&SEARCH}else{&TRANSFER};
    if let Some(client)=shared.get(){return Ok(client.clone());}
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
    let _=shared.set(client.clone());
    Ok(shared.get().cloned().unwrap_or(client))
}

impl DownloadTask {
    pub fn new(server_id: &str, kind: &str, emit: impl Fn(Progress) + Send + Sync + 'static) -> Result<Self, String> {
        let client=shared_client(kind)?;
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
        let (success,cancelled,failed)=match self.kind.as_str(){
            "world_delete"=>("Mundo eliminado","Eliminación cancelada","No se pudo eliminar el mundo"),
            "world_activate"=>("Mundo principal actualizado","Cambio de mundo cancelado","No se pudo cambiar el mundo principal"),
            "world_export"=>("Exportación completada","Exportación cancelada","No se pudo exportar el mundo"),
            "world_duplicate"=>("Mundo duplicado","Duplicado cancelado","No se pudo duplicar el mundo"),
            "world_import"=>("Mundo importado","Importación cancelada","No se pudo importar el mundo"),
            "server_import"=>("Servidor importado","Importación cancelada","No se pudo importar el servidor"),
            "content_remove"=>("Contenido retirado","Retirada cancelada","No se pudo retirar el contenido"),
            "content_toggle"=>("Estado del contenido actualizado","Cambio de contenido cancelado","No se pudo cambiar el estado del contenido"),
            "backup"=>("Copia guardada","Copia cancelada","No se pudo guardar la copia"),
            "restore"=>("Copia restaurada","Restauración cancelada","No se pudo restaurar la copia"),
            "install"=>("Instalación completada","Instalación cancelada","No se pudo instalar el servidor"),
            "content_install"=>("Contenido instalado","Instalación de contenido cancelada","No se pudo instalar el contenido"),
            "pack_install"=>("Modpack instalado","Instalación de modpack cancelada","No se pudo instalar el modpack"),
            "loader_update"=>("Loader actualizado","Actualización de loader cancelada","No se pudo actualizar el loader"),
            "minecraft_update"=>("Minecraft actualizado","Actualización de Minecraft cancelada","No se pudo actualizar Minecraft"),
            "runtime"|"runtime_reinstall"=>("Java preparado","Preparación de Java cancelada","No se pudo preparar Java"),
            _=>("Tarea completada","Tarea cancelada","No se pudo completar la tarea"),
        };
        match result {
            Ok(())=>self.report(success,"completed"),
            Err(error) if error==CANCELLED=>self.report(cancelled,"cancelled"),
            Err(error)=>{
                let attempt=self.latest.lock().map(|progress|progress.attempt).unwrap_or(1);
                self.progress(failed,"failed",0,None,None,attempt,Some(error.clone()));
            }
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

    async fn chunk(&self,response: &mut Response) -> Result<Option<impl std::ops::Deref<Target=[u8]>>,Failure> {
        tokio::select! {
            _ = self.cancellation.wait() => Err(Failure::local(CANCELLED)),
            result = response.chunk() => result.map_err(Failure::http),
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
        let mut verification=match hash {
            Some((kind,expected))=>{
                let digest:Box<dyn sha2::digest::DynDigest+Send>=match kind {
                    "sha1"=>Box::new(Sha1::new()),
                    "sha256"=>Box::new(Sha256::new()),
                    "sha512"=>Box::new(Sha512::new()),
                    _=>return Err(Failure::local("Algoritmo de hash no compatible")),
                };
                Some((expected,digest))
            }
            None=>None,
        };
        let mut response = self.request(url).await?;
        let total = response.content_length();
        if total.is_some_and(|bytes| bytes > self.max_bytes) { return Err(Failure::local("La descarga supera el tamaño permitido")); }
        let part = PartialFile(target.with_extension(format!("part-{}",self.id)));
        let mut file = File::create(&part.0).map_err(Failure::io)?;
        let mut received = 0u64;
        let mut last = Instant::now();
        let mut last_bytes = 0;
        self.progress(label,"downloading",0,total,None,attempt,None);
        while let Some(chunk) = self.chunk(&mut response).await? {
            received += chunk.len() as u64;
            if received > self.max_bytes { return Err(Failure::local("La descarga supera el tamaño permitido")); }
            file.write_all(&chunk).map_err(Failure::io)?;
            if let Some((_,digest))=&mut verification {digest.update(&chunk);}
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
        if let Some((expected,digest))=verification {
            let actual=hex(&digest.finalize());
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
