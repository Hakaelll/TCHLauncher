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
    pinned: HashMap<String, ContentVersion>,
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
        let version = if let Some(pinned) = self.pinned.remove(&project.id) {
            if version_id.is_some_and(|id| id != pinned.id) { return Err(format!("Se requieren versiones diferentes de {}", project.title)); }
            compatible(&pinned, &project, self.minecraft, self.loader)?;
            pinned
        } else { choose(self.source, &project, self.minecraft, self.loader, version_id)? };
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
    if projects.len() > 128 { return Err("El árbol de dependencias supera el límite de revisión".into()); }
    let mut pinned: HashMap<String, ContentVersion> = HashMap::new();
    // Root selections must also constrain dependencies visited before that root.
    for (_, version) in projects {
        if let Some(id) = version {
            let version = source.version(id)?;
            if version.id != *id { return Err("El inventario no coincide con Modrinth".into()); }
            if let Some(previous) = pinned.get(&version.project_id) {
                if previous.id != version.id { return Err(format!("Se requieren versiones diferentes de {}", version.project_id)); }
            }
            pinned.insert(version.project_id.clone(), version);
        }
    }
    let mut resolver = Resolver {source, minecraft, loader, pinned, chosen: HashMap::new(), files: HashMap::new(), entries: vec![], optional: vec![], incompatible: vec![]};
    for (project,version) in projects {resolver.visit(project,*version,false,0)?;}
    for dependency in &resolver.incompatible {
        let conflict = if let Some(id) = &dependency.version_id { resolver.chosen.values().any(|chosen| chosen == id) }
            else { dependency.project_id.as_ref().is_some_and(|id| resolver.chosen.contains_key(id)) };
        if conflict { return Err("Las dependencias declaran una incompatibilidad entre los archivos del plan".into()); }
    }
    Ok(ContentPlan {minecraft_version: minecraft.into(), loader: loader.into(), entries: resolver.entries,
        optional: resolver.optional, incompatible: resolver.incompatible})
}
