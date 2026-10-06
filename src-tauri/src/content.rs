use crate::{download::DownloadTask, model::Server};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const API: &str = "https://api.modrinth.com/v2";

pub fn icon(root:&std::path::Path,raw:&str)->Result<String,String> {
    use std::io::{Cursor,Read};
    use sha2::{Digest,Sha256};
    use base64::Engine;
    let url=reqwest::Url::parse(raw).map_err(|_|"Icono no válido")?;
    if url.scheme()!="https" || url.host_str()!=Some("cdn.modrinth.com") || !url.username().is_empty() || url.password().is_some() || url.port().is_some() {return Err("Origen de icono no válido".into());}
    let cache=root.join("content-icons");let key=format!("{:x}",Sha256::digest(raw.as_bytes()));let path=cache.join(format!("{key}.png"));
    if let Ok(meta)=std::fs::symlink_metadata(&path) {
        if meta.is_file() && !meta.file_type().is_symlink() && meta.len()<100000 {return Ok(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(std::fs::read(path).map_err(|e|e.to_string())?)));}
    }
    let client=reqwest::blocking::Client::builder().user_agent(concat!("TCHLauncher/",env!("CARGO_PKG_VERSION")," (https://github.com/Hakaelll/TCHLauncher)"))
        .timeout(std::time::Duration::from_secs(20)).redirect(reqwest::redirect::Policy::none()).build().map_err(|e|e.to_string())?;
    let response=client.get(url).send().and_then(|r|r.error_for_status()).map_err(|e|e.to_string())?;
    let mut data=vec![];response.take(2_000_001).read_to_end(&mut data).map_err(|e|e.to_string())?;if data.len()>2_000_000 {return Err("Icono demasiado grande".into());}
    let mut reader=image::ImageReader::new(Cursor::new(data)).with_guessed_format().map_err(|e|e.to_string())?;
    let mut limits=image::Limits::default();limits.max_image_width=Some(2048);limits.max_image_height=Some(2048);limits.max_alloc=Some(32_000_000);reader.limits(limits);
    let image=reader.decode().map_err(|e|e.to_string())?.thumbnail(96,96);let mut output=Cursor::new(vec![]);image.write_to(&mut output,image::ImageFormat::Png).map_err(|e|e.to_string())?;
    let data=output.into_inner();std::fs::create_dir_all(&cache).map_err(|e|e.to_string())?;std::fs::write(path,&data).map_err(|e|e.to_string())?;
    Ok(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(data)))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentFile {
    pub filename: String,
    pub url: String,
    pub hashes: BTreeMap<String, String>,
    pub size: u64,
    pub primary: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub version_type: String,
    pub date_published: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub files: Vec<ContentFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    #[serde(default)]
    pub environment: Vec<String>,
    #[serde(default)]
    pub server_side: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedContent {
    pub project: Project,
    pub version: ContentVersion,
    pub file: ContentFile,
    pub required: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPlan {
    pub minecraft_version: String,
    pub loader: String,
    pub entries: Vec<PlannedContent>,
    pub optional: Vec<Dependency>,
    pub incompatible: Vec<Dependency>,
}

pub fn loader(server: &Server) -> Result<&str, String> {
    match server.platform.as_str() {
        "fabric" => Ok("fabric"), "neoforge" => Ok("neoforge"), "paper" => Ok("paper"),
        _ => Err("Este servidor no admite mods ni plugins. Usa Fabric, NeoForge o Paper.".into()),
    }
}

fn identifier(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_') {
        return Err("Identificador de Modrinth no válido".into());
    }
    Ok(())
}

fn supported_loaders(loader:&str)->&[&str] {
    match loader {"paper"=>&["paper","spigot","bukkit"],"fabric"=>&["fabric"],"neoforge"=>&["neoforge"],_=>&[]}
}

pub struct Modrinth<'a>(pub &'a DownloadTask);

trait Source {
    fn project(&self, id: &str) -> Result<Project, String>;
    fn version(&self, id: &str) -> Result<ContentVersion, String>;
    fn versions(&self, id: &str, minecraft: &str, loader: &str) -> Result<Vec<ContentVersion>, String>;
}

impl Source for Modrinth<'_> {
    fn project(&self, id: &str) -> Result<Project, String> {
        identifier(id)?;
        serde_json::from_value(self.0.json(&format!("{API}/project/{id}"), "Consultando proyecto")?).map_err(|e| e.to_string())
    }
    fn version(&self, id: &str) -> Result<ContentVersion, String> {
        identifier(id)?;
        serde_json::from_value(self.0.json(&format!("{API}/version/{id}"), "Consultando versión")?).map_err(|e| e.to_string())
    }
    fn versions(&self, id: &str, minecraft: &str, loader: &str) -> Result<Vec<ContentVersion>, String> {
        identifier(id)?;
        let mut url = reqwest::Url::parse(&format!("{API}/project/{id}/version")).map_err(|e| e.to_string())?;
        url.query_pairs_mut().append_pair("loaders", &serde_json::json!(supported_loaders(loader)).to_string())
            .append_pair("game_versions", &serde_json::json!([minecraft]).to_string());
        serde_json::from_value(self.0.json(url.as_str(), "Buscando versiones compatibles")?).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SearchOptions {
    #[serde(default)] pub project_kind: String,
    #[serde(default)] pub category: String,
    #[serde(default)] pub sort: String,
    #[serde(default)] pub all_versions: bool,
}

fn search_url(server:&Server,query:&str,offset:u32,options:&SearchOptions)->Result<reqwest::Url,String> {
    let platform = loader(server)?;
    if query.len() > 200 || offset > 10000 { return Err("La búsqueda es demasiado larga".into()); }
    let mut url = reqwest::Url::parse(&format!("{API}/search")).map_err(|e| e.to_string())?;
    let categories:Vec<_>=supported_loaders(platform).iter().map(|loader|format!("categories:{loader}")).collect();
    let sort=if options.sort.is_empty(){"relevance"}else{options.sort.as_str()};
    if !matches!(sort,"relevance"|"downloads"|"follows"|"newest"|"updated") {return Err("Orden de búsqueda no válido".into());}
    if options.category.len()>64 || !options.category.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'-'|b'+')) {return Err("Categoría no válida".into());}
    let kind=if options.project_kind.is_empty(){if platform=="paper"{"plugin"}else{"mod"}}else{options.project_kind.as_str()};
    if !matches!(kind,"mod"|"plugin"|"modpack"|"resourcepack"|"shader"){return Err("Tipo de contenido no válido".into());}
    let mut facets=vec![vec![format!("all_project_types:{kind}")]];
    if matches!(kind,"mod"|"plugin"){facets.insert(0,categories);}
    if !options.all_versions {facets.push(vec![format!("versions:{}",server.version)]);}
    if !options.category.is_empty(){facets.push(vec![format!("categories:{}",options.category)]);}
    url.query_pairs_mut().append_pair("query", query.trim()).append_pair("facets", &serde_json::json!(facets).to_string()).append_pair("index",sort)
        .append_pair("limit", "20").append_pair("offset", &offset.to_string());
    Ok(url)
}

pub fn search(task:&DownloadTask,server:&Server,query:&str,offset:u32,options:&SearchOptions)->Result<serde_json::Value,String> {
    task.json(search_url(server,query,offset,options)?.as_str(),"Buscando en Modrinth")
}

pub fn project_page(project_id:&str)->Result<String,String>{
    identifier(project_id)?;
    Ok(format!("https://modrinth.com/project/{project_id}/versions"))
}

#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Availability {pub version_id:Option<String>,pub version_number:Option<String>,pub channel:Option<String>,pub environment:Option<String>}

pub fn availability(task:&DownloadTask,server:&Server,project_id:&str)->Result<Availability,String> {
    use std::sync::{Mutex,OnceLock};
    use std::time::{Duration,Instant};
    type Cache=HashMap<(String,String,String),(Instant,Availability)>;
    static CACHE:OnceLock<Mutex<Cache>>=OnceLock::new();
    identifier(project_id)?;
    let key=(project_id.to_string(),server.version.clone(),loader(server)?.to_string());
    let cache=CACHE.get_or_init(||Mutex::new(HashMap::new()));
    if let Ok(cache)=cache.lock(){if let Some((at,value))=cache.get(&key){if at.elapsed()<Duration::from_secs(300){return Ok(value.clone());}}}
    let available=versions(task,server,project_id)?;
    let selected=available.iter().find(|v|v.version_type=="release").or_else(||available.first());
    let result=Availability {version_id:selected.map(|v|v.id.clone()),version_number:selected.map(|v|v.version_number.clone()),channel:selected.map(|v|v.version_type.clone()),environment:selected.and_then(|v|v.environment.clone())};
    if let Ok(mut cache)=cache.lock(){cache.retain(|_,(at,_)|at.elapsed()<Duration::from_secs(300));if cache.len()>=512{cache.clear();}cache.insert(key,(Instant::now(),result.clone()));}
    Ok(result)
}

fn server_supported(environment: Option<&str>, legacy: Option<&str>) -> bool {
    match environment {
        Some("client_and_server" | "client_only_server_optional" | "server_only" | "server_only_client_optional" |
            "dedicated_server_only" | "client_or_server" | "client_or_server_prefers_both") => true,
        Some(_) => false,
        None => matches!(legacy, Some("required" | "optional")),
    }
}

fn compatible(version: &ContentVersion, project: &Project, minecraft: &str, loader: &str) -> Result<(), String> {
    if version.project_id != project.id { return Err("La versión pertenece a otro proyecto".into()); }
    if !version.game_versions.iter().any(|v| v == minecraft) { return Err(format!("{} no admite Minecraft {minecraft}", project.title)); }
    if !version.loaders.iter().any(|v| supported_loaders(loader).contains(&v.as_str())) { return Err(format!("{} no admite {loader}", project.title)); }
    let supported = if version.environment.as_deref().is_some_and(|value|value!="unknown") {
        server_supported(version.environment.as_deref(), None)
    } else if !project.environment.is_empty() {
        project.environment.iter().any(|value| server_supported(Some(value), None))
    } else { server_supported(None, project.server_side.as_deref()) };
    if !supported {
        return Err(format!("{} no declara compatibilidad con servidores dedicados", project.title));
    }
    Ok(())
}

fn selected_file(version: &ContentVersion) -> Result<ContentFile, String> {
    let candidates: Vec<_> = version.files.iter().filter(|file| file.filename.ends_with(".jar")).collect();
    let primary: Vec<_> = candidates.iter().filter(|file| file.primary).collect();
    let file = match (primary.len(), candidates.len()) {
        (1, _) => *primary[0], (0, 1) => candidates[0],
        _ => return Err("La versión no identifica un único archivo JAR instalable".into()),
    };
    let name = &file.filename;
    if name.len() > 180 || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')) || name.starts_with('.') {
        return Err("El nombre del archivo no es seguro".into());
    }
    let stem=name.split('.').next().unwrap_or_default().to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") ||
        (stem.len()==4 && (stem.starts_with("COM") || stem.starts_with("LPT")) && matches!(stem.as_bytes()[3],b'1'..=b'9')) {
        return Err("El nombre del archivo está reservado por el sistema".into());
    }
    let url = reqwest::Url::parse(&file.url).map_err(|_| "URL de descarga no válida")?;
    if url.scheme() != "https" || url.host_str() != Some("cdn.modrinth.com") || !url.username().is_empty() || url.password().is_some() || url.port().is_some() {
        return Err("El archivo no procede del CDN oficial de Modrinth".into());
    }
    let sha = file.hashes.get("sha512").ok_or("El archivo no tiene SHA-512 verificable")?;
    if sha.len() != 128 || !sha.bytes().all(|c| c.is_ascii_hexdigit()) || file.size == 0 || file.size > 512_000_000 {
        return Err("El tamaño o el hash del archivo no es válido".into());
    }
    Ok(file.clone())
}

pub fn plan(task: &DownloadTask, server: &Server, project_id: &str, version_id: Option<&str>) -> Result<ContentPlan, String> {
    resolve(&Modrinth(task), &server.version, loader(server)?, project_id, version_id)
}

pub fn versions(task: &DownloadTask, server: &Server, project_id: &str) -> Result<Vec<ContentVersion>, String> {
    let source=Modrinth(task);
    let platform=loader(server)?;
    let project=source.project(project_id)?;
    let mut versions=source.versions(&project.id,&server.version,platform)?;
    versions.retain(|version| compatible(version,&project,&server.version,platform).is_ok() && selected_file(version).is_ok());
    versions.sort_by(|a,b| b.date_published.cmp(&a.date_published).then_with(||a.id.cmp(&b.id)));
    Ok(versions)
}

pub(crate) fn installed_version(task:&DownloadTask,id:&str)->Result<ContentVersion,String> {
    Modrinth(task).version(id)
}

// Minecraft migration resolves one combined graph for the target version.
// Current releases are intentionally not checked against the target: replacing
// those incompatible releases is the purpose of this reviewed operation.
pub(crate) fn migration(task:&DownloadTask,server:&Server,projects:&[String])->Result<ContentPlan,String> {
    migration_plan(&Modrinth(task),&server.version,loader(server)?,projects)
}
fn migration_plan(source:&impl Source,minecraft:&str,loader:&str,projects:&[String])->Result<ContentPlan,String> {
    let roots=projects.iter().map(|project|(project.as_str(),None)).collect::<Vec<_>>();
    resolve_many(source,minecraft,loader,&roots)
}

pub fn check_existing(task:&DownloadTask, files:&[crate::content_storage::InstalledContent], plan:&ContentPlan)->Result<(),String> {
    let selected:HashMap<_,_>=plan.entries.iter().map(|entry|(entry.project.id.as_str(),entry.version.id.as_str())).collect();
    let mut final_versions:HashMap<String,String>=files.iter().filter_map(|file|Some((file.project_id.clone()?,file.version_id.clone()?))).collect();
    for entry in &plan.entries {final_versions.insert(entry.project.id.clone(),entry.version.id.clone());}
    let source=Modrinth(task);
    for file in files {
        let (Some(project),Some(id))=(&file.project_id,&file.version_id) else {continue};
        if selected.contains_key(project.as_str()) || file.modified {continue;}
        let version=source.version(id)?;
        if &version.project_id!=project {return Err("El inventario no coincide con Modrinth".into());}
        for dependency in &version.dependencies {
            let exists=if let Some(id)=&dependency.version_id {final_versions.values().any(|v|v==id)}
                else {dependency.project_id.as_ref().is_some_and(|id|final_versions.contains_key(id))};
            if dependency.dependency_type=="incompatible" && exists {return Err(format!("{} declara una incompatibilidad con este plan",file.title));}
            if dependency.dependency_type=="required" && !exists {return Err(format!("El plan dejaría sin una dependencia obligatoria a {}",file.title));}
        }
    }
    Ok(())
}

fn choose(source: &impl Source, project: &Project, minecraft: &str, loader: &str, version_id: Option<&str>) -> Result<ContentVersion, String> {
    let version = if let Some(id) = version_id { source.version(id)? } else {
        let mut versions = source.versions(&project.id, minecraft, loader)?;
        // Stable releases only by default; a specific beta/alpha requires explicit selection.
        versions.retain(|v| v.version_type == "release" && compatible(v, project, minecraft, loader).is_ok());
        versions.sort_by(|a,b| b.date_published.cmp(&a.date_published).then_with(|| a.id.cmp(&b.id)));
        versions.into_iter().next().ok_or_else(|| format!("No hay una versión estable de {} compatible con Minecraft {minecraft} y {loader}", project.title))?
    };
    compatible(&version, project, minecraft, loader)?;
    Ok(version)
}

struct Resolver<'a, S> {
    source: &'a S, minecraft: &'a str, loader: &'a str,
    chosen: HashMap<String, String>, files: HashMap<String, String>,
    entries: Vec<PlannedContent>, optional: Vec<Dependency>, incompatible: Vec<Dependency>,
}

impl<S: Source> Resolver<'_, S> {
    fn visit(&mut self, project_id: &str, version_id: Option<&str>, required: bool, depth: usize) -> Result<(), String> {
        if depth > 64 || self.chosen.len() >= 128 { return Err("El árbol de dependencias supera el límite de revisión".into()); }
        let project = self.source.project(project_id)?;
        if let Some(chosen) = self.chosen.get(&project.id) {
            if version_id.is_some_and(|id| id != chosen) { return Err(format!("Se requieren versiones diferentes de {}", project.title)); }
            return Ok(()); // Also terminates cycles before recursively visiting dependencies.
        }
        let version = choose(self.source, &project, self.minecraft, self.loader, version_id)?;
        let file = selected_file(&version)?;
        if let Some(other) = self.files.insert(file.filename.to_lowercase(), project.title.clone()) {
            return Err(format!("{} y {other} usan el mismo nombre de archivo", project.title));
        }
        self.chosen.insert(project.id.clone(), version.id.clone());
        for dependency in &version.dependencies {
            match dependency.dependency_type.as_str() {
                "required" => {
                    let id = if let Some(id) = &dependency.project_id { id.clone() }
                        else if let Some(id) = &dependency.version_id { self.source.version(id)?.project_id }
                        else { return Err(format!("{} requiere un archivo externo que no se puede resolver: {}", project.title, dependency.file_name.as_deref().unwrap_or("desconocido"))); };
                    self.visit(&id, dependency.version_id.as_deref(), true, depth + 1)?;
                }
                "optional" => self.optional.push(dependency.clone()),
                "incompatible" => self.incompatible.push(dependency.clone()),
                "embedded" => {},
                _ => return Err("Modrinth ha declarado un tipo de dependencia desconocido".into()),
            }
        }
        self.entries.push(PlannedContent {project, version, file, required});
        Ok(())
    }
}

fn resolve(source: &impl Source, minecraft: &str, loader: &str, project: &str, version: Option<&str>) -> Result<ContentPlan, String> {
    resolve_many(source,minecraft,loader,&[(project,version)])
}

pub fn updates(task:&DownloadTask,server:&Server)->Result<ContentPlan,String> {
    let files=crate::content_storage::list(server)?;
    if files.iter().any(|file|file.modified) {return Err("Hay archivos modificados fuera del launcher. Revísalos antes de actualizar en conjunto.".into());}
    let source=Modrinth(task);let platform=loader(server)?;let mut targets=vec![];
    for file in &files {
        let (Some(project_id),Some(version_id))=(&file.project_id,&file.version_id) else {continue};
        let project=source.project(project_id)?;let current=source.version(version_id)?;
        compatible(&current,&project,&server.version,platform)?;
        let latest=choose(&source,&project,&server.version,platform,None)?;
        // Bulk updates never silently downgrade an explicitly installed beta or alpha.
        let selected=if latest.date_published>current.date_published {latest.id} else {current.id};
        targets.push((project_id.clone(),selected));
    }
    let projects:Vec<_>=targets.iter().map(|(id,version)|(id.as_str(),Some(version.as_str()))).collect();
    if projects.is_empty() {return Err("No hay contenido de Modrinth instalado para buscar actualizaciones".into());}
    resolve_many(&source,&server.version,platform,&projects)
}

fn resolve_many(source: &impl Source, minecraft: &str, loader: &str, projects: &[(&str,Option<&str>)]) -> Result<ContentPlan, String> {
    let mut resolver = Resolver {source, minecraft, loader, chosen: HashMap::new(), files: HashMap::new(), entries: vec![], optional: vec![], incompatible: vec![]};
    for (project,version) in projects {resolver.visit(project,*version,false,0)?;}
    for dependency in &resolver.incompatible {
        let conflict = if let Some(id) = &dependency.version_id { resolver.chosen.values().any(|chosen| chosen == id) }
            else { dependency.project_id.as_ref().is_some_and(|id| resolver.chosen.contains_key(id)) };
        if conflict { return Err("Las dependencias declaran una incompatibilidad entre los archivos del plan".into()); }
    }
    Ok(ContentPlan {minecraft_version: minecraft.into(), loader: loader.into(), entries: resolver.entries,
        optional: resolver.optional, incompatible: resolver.incompatible})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_filters_are_conjunctive_and_allow_other_versions_without_changing_loader() {
        let server:Server=serde_json::from_value(serde_json::json!({"id":"search","name":"Search","version":"1.21.11","platform":"fabric","directory":"unused","jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        let options=SearchOptions {category:"worldgen".into(),sort:"downloads".into(),all_versions:false,..Default::default()};
        let url=search_url(&server," structures ",20,&options).unwrap();
        let params:HashMap<_,_>=url.query_pairs().into_owned().collect();
        let facets:Vec<Vec<String>>=serde_json::from_str(&params["facets"]).unwrap();
        for expected in ["categories:fabric","categories:worldgen","versions:1.21.11","all_project_types:mod"] {assert!(facets.contains(&vec![expected.into()]));}
        assert_eq!(params["index"],"downloads");assert_eq!(params["query"],"structures");assert_eq!(params["offset"],"20");
        let url=search_url(&server,"",0,&SearchOptions {all_versions:true,..options}).unwrap();
        assert!(!url.query_pairs().find(|(key,_)|key=="facets").unwrap().1.contains("versions:"));
        assert!(search_url(&server,"",0,&SearchOptions {category:"worldgen\"],[\"versions:other".into(),..Default::default()}).is_err());
        let url=search_url(&server,"",0,&SearchOptions{project_kind:"resourcepack".into(),category:"16x".into(),..Default::default()}).unwrap();
        assert!(url.query_pairs().find(|(key,_)|key=="facets").unwrap().1.contains("categories:16x"));
    }
    #[test]
    fn discovery_types_do_not_apply_server_loader_filters_to_client_content_or_packs(){
        let server:Server=serde_json::from_value(serde_json::json!({"id":"search","name":"Search","version":"1.21.11","platform":"fabric","directory":"x","jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        for kind in ["modpack","resourcepack","shader"]{
            let url=search_url(&server,"",0,&SearchOptions{project_kind:kind.into(),..Default::default()}).unwrap();
            let facets=url.query_pairs().find(|(key,_)|key=="facets").unwrap().1.into_owned();
            assert!(facets.contains(&format!("all_project_types:{kind}")));assert!(!facets.contains("categories:fabric"));assert!(facets.contains("versions:1.21.11"));
        }
        assert!(search_url(&server,"",0,&SearchOptions{project_kind:"other".into(),..Default::default()}).is_err());
        assert_eq!(project_page("Abc123").unwrap(),"https://modrinth.com/project/Abc123/versions");
        for id in ["../x","x?url=evil","https://evil.test"]{assert!(project_page(id).is_err());}
    }
    struct Fixture { projects: HashMap<String, Project>, versions: HashMap<String, ContentVersion> }
    impl Source for Fixture {
        fn project(&self, id: &str) -> Result<Project,String> {self.projects.get(id).cloned().ok_or("Proyecto ausente".into())}
        fn version(&self, id: &str) -> Result<ContentVersion,String> {self.versions.get(id).cloned().ok_or("Versión ausente".into())}
        fn versions(&self, id: &str, _: &str, _: &str) -> Result<Vec<ContentVersion>,String> {Ok(self.versions.values().filter(|v|v.project_id==id).cloned().collect())}
    }
    fn fixture() -> Fixture {
        let mut result=Fixture {projects:HashMap::new(),versions:HashMap::new()};
        for id in ["A","B","C"] {
            result.projects.insert(id.into(), Project {id:id.into(),title:id.into(),description:String::new(),icon_url:None,environment:vec![],server_side:Some("required".into())});
            result.versions.insert(format!("{id}1"), ContentVersion {id:format!("{id}1"),project_id:id.into(),name:id.into(),version_number:"1".into(),version_type:"release".into(),date_published:"2026-01-01T00:00:00Z".into(),game_versions:vec!["1.21.4".into()],loaders:vec!["fabric".into()],environment:None,changelog:None,dependencies:vec![],files:vec![ContentFile {filename:format!("{id}.jar"),url:format!("https://cdn.modrinth.com/data/{id}/versions/{id}1/{id}.jar"),hashes:BTreeMap::from([("sha512".into(),"a".repeat(128))]),size:100,primary:true}]});
        } result
    }
    fn dependency(project:&str, kind:&str) -> Dependency {Dependency {project_id:Some(project.into()),version_id:None,file_name:None,dependency_type:kind.into()}}
    fn add_migration_release(data:&mut Fixture,project:&str,id:&str) {
        let mut version=data.versions[&format!("{project}1")].clone();
        version.id=id.into();version.version_number=id.into();version.game_versions=vec!["1.21.5".into()];version.date_published="2026-02-01T00:00:00Z".into();
        data.versions.insert(id.into(),version);
    }
    #[test]
    fn migration_replaces_old_minecraft_releases_as_one_dependency_graph() {
        let mut data=fixture();add_migration_release(&mut data,"A","A2");add_migration_release(&mut data,"B","B2");
        let mut required=dependency("B","required");required.version_id=Some("B2".into());
        data.versions.get_mut("A2").unwrap().dependencies=vec![required];
        let plan=migration_plan(&data,"1.21.5","fabric",&["A".into(),"B".into()]).unwrap();
        assert_eq!(plan.entries.len(),2);
        assert!(plan.entries.iter().all(|entry|entry.version.game_versions==vec!["1.21.5"]));
        assert!(plan.entries.iter().any(|entry|entry.version.id=="B2"&&entry.required));
        assert!(migration_plan(&data,"1.21.6","fabric",&["A".into()]).is_err());
    }
    #[test]
    fn migration_never_substitutes_a_newer_client_only_wrong_loader_or_beta_release() {
        let mut data=fixture();add_migration_release(&mut data,"A","A2");
        for (id,environment,loader,kind) in [("client",Some("client_only"),"fabric","release"),("wrong",None,"neoforge","release"),("beta",None,"fabric","beta")] {
            let mut version=data.versions["A2"].clone();version.id=id.into();version.environment=environment.map(str::to_owned);version.loaders=vec![loader.into()];version.version_type=kind.into();version.date_published="2026-03-01T00:00:00Z".into();data.versions.insert(id.into(),version);
        }
        assert_eq!(migration_plan(&data,"1.21.5","fabric",&["A".into()]).unwrap().entries[0].version.id,"A2");
    }
    #[test]
    fn migration_blocks_conflicting_exact_dependency_versions_instead_of_partial_replacement() {
        let mut data=fixture();for id in ["A","B","C"]{add_migration_release(&mut data,id,&format!("{id}2"));}
        add_migration_release(&mut data,"B","B3");
        let mut required=dependency("B","required");required.version_id=Some("B2".into());data.versions.get_mut("A2").unwrap().dependencies=vec![required];
        let mut required=dependency("B","required");required.version_id=Some("B3".into());data.versions.get_mut("C2").unwrap().dependencies=vec![required];
        assert!(migration_plan(&data,"1.21.5","fabric",&["A".into(),"C".into()]).unwrap_err().contains("versiones diferentes"));
    }
    #[test]
    fn resolves_transitive_required_cycles_and_separates_optional() {
        let mut data=fixture();
        data.versions.get_mut("A1").unwrap().dependencies=vec![dependency("B","required"),dependency("C","optional")];
        data.versions.get_mut("B1").unwrap().dependencies=vec![dependency("A","required")];
        let plan=resolve(&data,"1.21.4","fabric","A",None).unwrap();
        assert_eq!(plan.entries.iter().map(|e|e.project.id.as_str()).collect::<Vec<_>>(),vec!["B","A"]);
        assert!(plan.entries[0].required);assert!(!plan.entries[1].required);assert_eq!(plan.optional.len(),1);
    }
    #[test]
    fn never_chooses_newer_incompatible_release_or_client_only_version() {
        let mut data=fixture();let mut newer=data.versions["A1"].clone();newer.id="A2".into();newer.date_published="2026-02-01T00:00:00Z".into();newer.game_versions=vec!["1.22".into()];data.versions.insert("A2".into(),newer);
        assert_eq!(resolve(&data,"1.21.4","fabric","A",None).unwrap().entries[0].version.id,"A1");
        assert!(resolve(&data,"1.21.4","fabric","A",Some("A2")).is_err());
        data.versions.get_mut("A1").unwrap().environment=Some("client_only".into());
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
        assert!(resolve(&data,"1.21.4","paper","B",None).is_err());
    }
    #[test]
    fn rejects_declared_conflicts_missing_dependencies_and_unsafe_files() {
        let mut data=fixture();data.versions.get_mut("A1").unwrap().dependencies=vec![dependency("B","required"),dependency("B","incompatible")];
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
        data.versions.get_mut("A1").unwrap().dependencies=vec![dependency("Missing","required")];
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
        data.versions.get_mut("A1").unwrap().dependencies.clear();
        data.versions.get_mut("A1").unwrap().files[0].filename="../escape.jar".into();
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
        data.versions.get_mut("A1").unwrap().files[0].filename="A.jar".into();
        data.versions.get_mut("A1").unwrap().files[0].filename="CON.jar".into();
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
        data.versions.get_mut("A1").unwrap().files[0].filename="A.jar".into();
        data.versions.get_mut("A1").unwrap().files[0].url="https://example.com/A.jar".into();
        assert!(resolve(&data,"1.21.4","fabric","A",None).is_err());
    }

    #[test]
    #[ignore = "Reads the live official Modrinth API; does not download or execute JARs"]
    fn resolves_live_lithium_release() {
        let task=DownloadTask::new("integration","content_plan",|_| {}).unwrap();
        let plan=resolve(&Modrinth(&task),"1.21.4","fabric","lithium",None).unwrap();
        assert!(!plan.entries.is_empty());
        assert!(plan.entries.iter().any(|entry|entry.project.title=="Lithium"));
        for entry in plan.entries {
            assert!(entry.version.game_versions.contains(&"1.21.4".into()));
            assert!(entry.version.loaders.contains(&"fabric".into()));
            assert_eq!(entry.version.version_type,"release");
            assert_eq!(entry.file.hashes["sha512"].len(),128);
        }
    }

    #[test]
    #[ignore = "Checks release-level availability against the official Modrinth API without installing JARs"]
    fn checks_live_server_availability_before_installation() {
        let task=DownloadTask::new("availability","content_search",|_|{}).unwrap();
        let mut server:Server=serde_json::from_value(serde_json::json!({"id":"availability","name":"Availability","version":"1.21.4","platform":"fabric","directory":"unused","jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        let found=availability(&task,&server,"lithium").unwrap();assert!(found.version_id.is_some());assert_eq!(found.channel.as_deref(),Some("release"));
        let versions=versions(&task,&server,"lithium").unwrap();assert!(versions.iter().any(|version|Some(&version.id)==found.version_id.as_ref()));
        server.version="1.0".into();assert!(availability(&task,&server,"lithium").unwrap().version_id.is_none());
    }

    #[test]
    #[ignore = "Reads public Modrinth search metadata for packs, textures and shaders; downloads no content"]
    fn searches_live_content_divisions_without_server_loader_constraints(){
        let task=DownloadTask::new("discovery-types","content_search",|_|{}).unwrap();
        let server:Server=serde_json::from_value(serde_json::json!({"id":"discovery-types","name":"Discovery","version":"1.21.11","platform":"fabric","directory":"unused","jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        for kind in ["modpack","resourcepack","shader"]{
            let result=search(&task,&server,"",0,&SearchOptions{project_kind:kind.into(),all_versions:true,..Default::default()}).unwrap();
            assert!(result["total_hits"].as_u64().unwrap()>0);
            let hits=result["hits"].as_array().unwrap();assert!(!hits.is_empty());
            assert!(hits.iter().all(|hit|hit["all_project_types"].as_array().is_some_and(|types|types.iter().any(|value|value.as_str()==Some(kind)))));
            println!("{kind}: {} projects; {} returned",result["total_hits"],hits.len());
        }
    }

    #[test]
    #[ignore = "Downloads a real Modrinth release into an isolated temporary directory; never executes it"]
    fn downloads_live_lithium_with_sha512() {
        struct Temp(std::path::PathBuf);
        impl Drop for Temp {fn drop(&mut self) {let _=std::fs::remove_dir_all(&self.0);}}
        let temp=Temp(std::env::temp_dir().join(format!("tch-modrinth-{}",uuid::Uuid::new_v4())));
        std::fs::create_dir_all(&temp.0).unwrap();
        let task=DownloadTask::new("integration","content_install",|_| {}).unwrap();
        let plan=resolve(&Modrinth(&task),"1.21.4","fabric","lithium",None).unwrap();
        for entry in plan.entries {
            let target=temp.0.join(&entry.file.filename);
            task.download(&entry.file.url,&target,Some(("sha512",&entry.file.hashes["sha512"])),&entry.project.title).unwrap();
            assert_eq!(std::fs::metadata(&target).unwrap().len(),entry.file.size);
            let archive=zip::ZipArchive::new(std::fs::File::open(&target).unwrap()).unwrap();
            assert!(!archive.is_empty());
        }
        assert!(std::fs::read_dir(&temp.0).unwrap().all(|entry|entry.unwrap().path().extension().is_some_and(|v|v=="jar")));
    }

    #[test]
    fn rejects_pinned_version_conflicts_and_filename_collisions() {
        let mut data=fixture();
        data.versions.get_mut("A1").unwrap().dependencies=vec![dependency("B","required")];
        let mut old=data.versions["A1"].clone();old.id="A0".into();old.date_published="2025-01-01T00:00:00Z".into();data.versions.insert("A0".into(),old);
        let mut pinned=dependency("A","required");pinned.version_id=Some("A0".into());
        data.versions.get_mut("B1").unwrap().dependencies=vec![pinned];
        assert!(resolve(&data,"1.21.4","fabric","A",None).unwrap_err().contains("versiones diferentes"));
        data.versions.get_mut("B1").unwrap().dependencies.clear();
        data.versions.get_mut("B1").unwrap().files[0].filename="a.jar".into();
        assert!(resolve(&data,"1.21.4","fabric","A",None).unwrap_err().contains("nombre de archivo"));
    }

    #[test]
    fn uses_project_environment_only_when_version_does_not_declare_one() {
        let mut data=fixture();data.projects.get_mut("A").unwrap().environment=vec!["server_only".into()];
        data.versions.get_mut("A1").unwrap().environment=Some("unknown".into());
        data.versions.get_mut("A1").unwrap().loaders=vec!["bukkit".into()];
        assert!(resolve(&data,"1.21.4","paper","A",None).is_ok());
        data.versions.get_mut("A1").unwrap().environment=Some("client_only".into());
        assert!(resolve(&data,"1.21.4","paper","A",None).is_err());
        data.versions.get_mut("A1").unwrap().environment=Some("unknown".into());
        data.projects.get_mut("A").unwrap().environment=vec!["client_only".into()];
        assert!(resolve(&data,"1.21.4","paper","A",None).is_err());
    }
}
