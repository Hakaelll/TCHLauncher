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

#[cfg(test)]
mod tests{
    use super::*;
    fn node(id:&str,channel:&str)->Value{serde_json::json!({"node":{"key":id,"builds":{"edges":[{"node":{"number":1,"channel":channel,"download":{"name":"paper.jar","url":"https://fill-data.papermc.io/v1/objects/file/paper.jar","checksums":{"sha256":"a".repeat(64)}}}}]}}})}
    fn page(edges:Vec<Value>,next:bool,cursor:Option<&str>)->Value{serde_json::json!({"data":{"project":{"versions":{"edges":edges,"pageInfo":{"hasNextPage":next,"endCursor":cursor}}}}})}
    fn official()->Vec<VersionInfo>{[("26.2","release"),("1.21.8","release"),("1.21.5","release"),("1.13-pre7","snapshot"),("1.8.8","release")].into_iter().map(|(id,kind)|VersionInfo{id:id.into(),kind:kind.into()}).collect()}
    #[test]
    fn intersects_all_pages_with_mojang_releases_and_preserves_chronology(){
        let mut requests=vec![];
        let result=collect(official(),|after|{requests.push(after.map(str::to_owned));Ok(if after.is_none(){page(vec![node("1.21.8","STABLE"),node("1.21.5","EXPERIMENTAL"),node("1.13-pre7","STABLE")],true,Some("older"))}else{page(vec![node("1.8.8","STABLE"),node("26.2","STABLE"),node("99.0","STABLE")],false,None)})}).unwrap();
        assert_eq!(requests,vec![None,Some("older".into())]);assert_eq!(result.iter().map(|version|version.id.as_str()).collect::<Vec<_>>(),vec!["26.2","1.21.8","1.8.8"]);
    }
    #[test]
    fn rejects_partial_errors_broken_pagination_and_unverified_downloads(){
        let mut broken=page(vec![node("26.2","STABLE")],false,None);broken["errors"]=serde_json::json!([{"message":"partial failure"}]);assert!(collect(official(),|_|Ok(broken.clone())).is_err());
        assert!(collect(official(),|_|Ok(page(vec![node("26.2","STABLE")],true,Some("same")))).is_err());
        assert!(collect(official(),|_|Ok(page(vec![],true,Some("empty")))).is_err());
        assert!(collect(official(),|_|Ok(page(vec![],true,None))).is_err());
        assert!(collect(official(),|_|Err("offline".into())).is_err());
        for (key,value) in [("url","http://fill-data.papermc.io/file"),("url","https://other.test/file"),("url","https://user@fill-data.papermc.io/file"),("name","../paper.jar"),("name","paper.zip")]{let mut invalid=node("26.2","STABLE");invalid["node"]["builds"]["edges"][0]["node"]["download"][key]=value.into();assert!(collect(official(),|_|Ok(page(vec![invalid.clone()],false,None))).unwrap().is_empty());}
        let mut invalid=node("26.2","STABLE");invalid["node"]["builds"]["edges"][0]["node"]["download"]["checksums"]=Value::Null;assert!(collect(official(),|_|Ok(page(vec![invalid.clone()],false,None))).unwrap().is_empty());
        let mut count=0;assert!(collect(official(),|_|{count+=1;Ok(page(vec![node("26.2","STABLE")],true,Some(&format!("page-{count}"))))}).is_err());assert_eq!(count,10);
    }
    #[test]
    #[ignore="Reads official Mojang/Paper stable catalogs and verifies REST build evidence without downloads or Java"]
    fn live_catalog_contains_only_downloadable_stable_paper_releases(){
        let versions=crate::install::platform_versions_with_options("paper",false).unwrap();assert!(!versions.is_empty());
        let task=crate::download::DownloadTask::new("paper-catalog-test","install",|_|{}).unwrap();
        for version in [&versions[0],versions.iter().find(|version|version.id=="1.8.8").unwrap()]{let builds=task.json(&format!("https://fill.papermc.io/v3/projects/paper/versions/{}/builds",version.id),"Comprobando disponibilidad Paper").unwrap();assert!(builds.as_array().unwrap().iter().any(|build|build["channel"]=="STABLE"&&build["downloads"]["server:default"]["checksums"]["sha256"].as_str().is_some_and(|hash|hash.len()==64)));}
    }
}
