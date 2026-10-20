use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor,Read};
use std::path::Path;
use std::sync::{Arc,Mutex,OnceLock};
use std::time::{Duration,Instant};

static LOCKS: OnceLock<Mutex<HashMap<String,Arc<Mutex<Option<Instant>>>>>> = OnceLock::new();

fn uuid_key(uuid: &str) -> Result<String,String> {
    uuid::Uuid::parse_str(uuid).map(|id|id.simple().to_string()).map_err(|_|"UUID de jugador no válido".into())
}

fn skin_url(profile: &Value,uuid: &str) -> Result<Option<String>,String> {
    if profile["id"].as_str() != Some(uuid) {return Err("El perfil no coincide con el jugador".into());}
    let Some(encoded)=profile["properties"].as_array().and_then(|items|items.iter().find(|item|item["name"]=="textures")).and_then(|item|item["value"].as_str()) else {return Ok(None);};
    if encoded.len()>32_768 {return Err("El perfil de textura es demasiado grande".into());}
    let data=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|_|"Perfil de textura no válido")?;
    let textures:Value=serde_json::from_slice(&data).map_err(|_|"Perfil de textura no válido")?;
    let Some(raw)=textures["textures"]["SKIN"]["url"].as_str() else {return Ok(None);};
    let mut url=reqwest::Url::parse(raw).map_err(|_|"URL de skin no válida")?;
    let hash=url.path().strip_prefix("/texture/").ok_or("Ruta de skin no válida")?;
    if url.host_str()!=Some("textures.minecraft.net") || !matches!(url.scheme(),"http"|"https") || url.port().is_some() || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() || hash.len()!=64 || !hash.chars().all(|c|c.is_ascii_hexdigit()) {return Err("La skin no procede del servidor oficial de texturas".into());}
    url.set_scheme("https").map_err(|_|"URL de skin no válida")?;
    Ok(Some(url.into()))
}

fn fetch_bytes(client: &reqwest::blocking::Client,url: &str,limit: u64) -> Result<Vec<u8>,String> {
    let response=client.get(url).send().and_then(|response|response.error_for_status()).map_err(|e|e.to_string())?;
    if response.content_length().is_some_and(|size|size>limit) {return Err("Respuesta de skin demasiado grande".into());}
    let mut data=Vec::new();response.take(limit+1).read_to_end(&mut data).map_err(|e|e.to_string())?;
    if data.len() as u64>limit {return Err("Respuesta de skin demasiado grande".into());}
    Ok(data)
}

fn head_png(bytes: &[u8]) -> Result<Vec<u8>,String> {
    let mut reader=image::ImageReader::new(Cursor::new(bytes));reader.set_format(image::ImageFormat::Png);
    let mut limits=image::Limits::default();limits.max_image_width=Some(64);limits.max_image_height=Some(64);limits.max_alloc=Some(512_000);reader.limits(limits);
    let skin=reader.decode().map_err(|_|"La skin no es un PNG válido")?.to_rgba8();
    if skin.width()!=64 || !matches!(skin.height(),32|64) {return Err("Dimensiones de skin no válidas".into());}
    let mut head=image::imageops::crop_imm(&skin,8,8,8,8).to_image();
    let opaque_legacy_hat=skin.height()==32 && (0..32).all(|y|(32..64).all(|x|skin.get_pixel(x,y)[3]==255));
    if !opaque_legacy_hat {let hat=image::imageops::crop_imm(&skin,40,8,8,8).to_image();image::imageops::overlay(&mut head,&hat,0,0);}
    let mut output=Cursor::new(Vec::new());image::DynamicImage::ImageRgba8(head).write_to(&mut output,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    Ok(output.into_inner())
}

pub fn player_head(root: &Path,uuid: &str) -> Result<Option<String>,String> {
    let key=uuid_key(uuid)?;
    let lock=LOCKS.get_or_init(||Mutex::new(HashMap::new())).lock().map_err(|e|e.to_string())?.entry(key.clone()).or_insert_with(||Arc::new(Mutex::new(None))).clone();
    let mut failed=lock.lock().map_err(|e|e.to_string())?;
    let path=root.join("skin-cache").join(format!("{key}.png"));
    let cached=fs::metadata(&path).ok().filter(|meta|meta.len()<=16_384).and_then(|meta|fs::read(&path).ok().map(|bytes|(meta,bytes)));
    let data_url=|bytes:&[u8]|format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes));
    if let Some((meta,bytes))=&cached {if meta.modified().ok().and_then(|time|time.elapsed().ok()).is_some_and(|age|age<Duration::from_secs(86_400)) {return Ok(Some(data_url(bytes)));}}
    if failed.is_some_and(|time|time.elapsed()<Duration::from_secs(300)) {return Ok(cached.map(|(_,bytes)|data_url(&bytes)));}
    let result=(|| -> Result<Option<Vec<u8>>,String> {
        let client=reqwest::blocking::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(12)).user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION"))).build().map_err(|e|e.to_string())?;
        let profile:Value=serde_json::from_slice(&fetch_bytes(&client,&format!("https://sessionserver.mojang.com/session/minecraft/profile/{key}"),65_536)?).map_err(|_|"El servicio devolvió un perfil no válido")?;
        let Some(url)=skin_url(&profile,&key)? else {return Ok(None);};
        let head=head_png(&fetch_bytes(&client,&url,262_144)?)?;
        fs::create_dir_all(path.parent().unwrap()).map_err(|e|e.to_string())?;
        let temporary=path.with_extension(format!("part-{}",uuid::Uuid::new_v4()));
        fs::write(&temporary,&head).map_err(|e|e.to_string())?;
        if let Err(error)=crate::download::replace_file(&temporary,&path) {let _=fs::remove_file(temporary);return Err(error);}
        Ok(Some(head))
    })();
    match result {
        Ok(Some(bytes))=>{*failed=None;Ok(Some(data_url(&bytes)))},
        Ok(None)=>{*failed=Some(Instant::now());Ok(None)},
        Err(error)=>{*failed=Some(Instant::now());if let Some((_,bytes))=cached {Ok(Some(data_url(&bytes)))} else {Err(error)}},
    }
}
