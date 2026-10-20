//! Official stable build availability, intersected with Mojang release chronology.
use crate::model::VersionInfo;
use serde_json::Value;
use std::{collections::HashSet,io::Read,time::{Duration,Instant}};

const QUERY:&str=r#"query StableVersions($after:String) { project(key:"paper") { versions(first:100,after:$after,orderBy:{direction:DESC}) { pageInfo { hasNextPage endCursor } edges { node { key builds(filterBy:{channels:[STABLE]},first:1,orderBy:{direction:DESC}) { edges { node { number channel download(key:"server:default") { name url checksums { sha256 } } } } } } } } } }"#;

pub fn versions(client:&reqwest::blocking::Client,official:Vec<VersionInfo>)->Result<Vec<VersionInfo>,String>{
    let started=Instant::now();
    collect(official,|after|{
        let remaining=Duration::from_secs(60).checked_sub(started.elapsed()).filter(|time|!time.is_zero()).ok_or("La consulta de versiones Paper tardó demasiado; vuelve a intentarlo")?;
        let response=client.post("https://fill.papermc.io/graphql").timeout(remaining.min(Duration::from_secs(15)))
            .json(&serde_json::json!({"query":QUERY,"variables":{"after":after}})).send().and_then(|response|response.error_for_status()).map_err(|error|format!("No se pudo consultar el catálogo estable de Paper: {error}"))?;
        let mut bytes=Vec::new();response.take(2_000_001).read_to_end(&mut bytes).map_err(|error|format!("No se pudo leer el catálogo Paper: {error}"))?;
        if bytes.len()>2_000_000{return Err("Respuesta de catálogo Paper demasiado grande".into());}
        serde_json::from_slice(&bytes).map_err(|_|"El catálogo Paper no contiene JSON válido".into())
    })
}

fn stable_download(build:&Value)->bool{
    if build["channel"].as_str()!=Some("STABLE")||build["number"].as_u64().is_none_or(|number|number==0){return false;}
    let download=&build["download"];
    let Some(name)=download["name"].as_str() else{return false;};
    if name.len()>180||!name.ends_with(".jar")||name.chars().any(|ch|ch.is_control()||matches!(ch,'/'|'\\'|':'|'<'|'>'|'"'|'|'|'?'|'*')){return false;}
    let Some(hash)=download["checksums"]["sha256"].as_str() else{return false;};
    if hash.len()!=64||!hash.bytes().all(|byte|byte.is_ascii_hexdigit()){return false;}
    let Some(url)=download["url"].as_str().and_then(|url|reqwest::Url::parse(url).ok()) else{return false;};
    url.scheme()=="https"&&url.host_str()==Some("fill-data.papermc.io")&&url.username().is_empty()&&url.password().is_none()&&url.port().is_none()&&url.fragment().is_none()
}

fn collect(mut official:Vec<VersionInfo>,mut page:impl FnMut(Option<&str>)->Result<Value,String>)->Result<Vec<VersionInfo>,String>{
    let mut supported=HashSet::new();let mut after:Option<String>=None;let mut cursors=HashSet::new();
    for _ in 0..10{
        let data=page(after.as_deref())?;
        if !data["errors"].is_null()&&data["errors"].as_array().is_none_or(|errors|!errors.is_empty()){return Err("Paper no pudo confirmar su catálogo estable; vuelve a intentarlo".into());}
        let connection=&data["data"]["project"]["versions"];
        let edges=connection["edges"].as_array().filter(|edges|edges.len()<=100).ok_or("Catálogo Paper no válido")?;
        for edge in edges{
            let node=&edge["node"];let id=node["key"].as_str().ok_or("Falta la versión de Paper")?;
            crate::fabric::validate_version(id)?;
            let builds=node["builds"]["edges"].as_array().filter(|builds|builds.len()<=1).ok_or("Lista de builds Paper no válida")?;
            if builds.first().is_some_and(|build|stable_download(&build["node"])) {supported.insert(id.to_owned());}
        }
        let next=connection["pageInfo"]["hasNextPage"].as_bool().ok_or("Falta la paginación de Paper")?;
        if !next{official.retain(|version|version.kind=="release"&&supported.contains(&version.id));return Ok(official);}
        let cursor=connection["pageInfo"]["endCursor"].as_str().filter(|cursor|!cursor.is_empty()&&cursor.len()<=1024&&!cursor.chars().any(char::is_control)).ok_or("Cursor de catálogo Paper no válido")?;
        if edges.is_empty()||!cursors.insert(cursor.to_owned()){return Err("La paginación de Paper no avanza".into());}
        after=Some(cursor.to_owned());
    }
    Err("El catálogo Paper supera el límite de consulta; no se mostrará una lista incompleta".into())
}
