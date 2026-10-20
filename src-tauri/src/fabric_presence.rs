//! Static presence evidence only. Fabric Loader remains the version/selection authority.
use crate::{content_diagnostics::{inspect_fabric,Dependency},content_storage::InstalledContent};
use std::{collections::{HashMap,HashSet},fs,io::{Read,Seek,Cursor},path::Path};

pub struct Review {pub issues:HashMap<String,Vec<String>>,pub blocking:HashMap<String,Vec<String>>,pub note:String}
struct Candidate {owner:String,id:String,version:String,top_level:bool,provides:Vec<String>,dependencies:Vec<Dependency>}
struct Budget {nodes:usize,bytes:u64}
impl Budget {
    fn claim(&mut self,size:u64)->Result<(),String>{
        if size>32_000_000||self.bytes.saturating_add(size)>128_000_000 {return Err("Se alcanzó el límite de lectura de dependencias incluidas".into());}
        self.bytes+=size;Ok(())
    }
}
fn read_entry<R:Read+Seek>(archive:&mut zip::ZipArchive<R>,name:&str,limit:u64,budget:&mut Budget)->Result<Vec<u8>,String>{
    if archive.file_names().filter(|entry|*entry==name).count()!=1 {return Err("Entrada de metadatos ausente o repetida".into());}
    let file=archive.by_name(name).map_err(|_|"No se pudo leer el JAR incluido")?;
    let size=file.size();if size>limit{return Err("Entrada de metadatos demasiado grande".into());}
    budget.claim(size)?;
    let mut data=Vec::new();file.take(limit+1).read_to_end(&mut data).map_err(|_|"No se pudo leer el JAR incluido")?;
    if data.len() as u64!=size {return Err("El tamaño de la entrada no coincide".into());}Ok(data)
}
fn scan<R:Read+Seek>(archive:&mut zip::ZipArchive<R>,owner:&str,depth:usize,budget:&mut Budget,nodes:&mut Vec<Candidate>)->Result<(),String>{
    budget.nodes+=1;
    if depth>8||budget.nodes>2048||archive.len()>100_000{return Err("Demasiados niveles o archivos de dependencias incluidas".into());}
    let data=read_entry(archive,"fabric.mod.json",1_000_000,budget)?;
    let value:serde_json::Value=serde_json::from_slice(&data).map_err(|_|"Metadatos Fabric no legibles")?;
    let descriptor=inspect_fabric(&value)?;
    let id=descriptor.id.ok_or("Descriptor Fabric antiguo sin identidad comprobable")?;
    // Client-only parents and their nested candidates are not server providers.
    if descriptor.environment.as_deref()==Some("client"){return Ok(());}
    let mut provides=Vec::new();
    if let Some(value)=value.get("provides"){
        let aliases=value.as_array().filter(|aliases|aliases.len()<=512).ok_or("Alias Fabric no válidos")?;
        for alias in aliases {
            let alias=alias.as_str().filter(|alias|alias.len()>=2&&alias.len()<=64&&alias.as_bytes()[0].is_ascii_lowercase()&&alias.bytes().all(|c|c.is_ascii_lowercase()||c.is_ascii_digit()||matches!(c,b'-'|b'_'))).ok_or("Alias Fabric no válido")?;
            provides.push(alias.into());
        }
    }
    nodes.push(Candidate{owner:owner.into(),id,version:descriptor.version.ok_or("Version Fabric no comprobable")?,top_level:depth==0,provides,dependencies:descriptor.dependencies});
    if let Some(value)=value.get("jars"){
        let jars=value.as_array().filter(|jars|jars.len()<=512).ok_or("Lista de JAR incluidos no válida")?;
        for jar in jars {
            let name=jar.get("file").and_then(serde_json::Value::as_str).filter(|name|name.len()<=512&&name.ends_with(".jar")&&!name.contains(['\\',':'])&&!name.chars().any(char::is_control)&&name.split('/').all(|part|!part.is_empty()&&part!="."&&part!="..")).ok_or("Ruta de JAR incluido no válida")?;
            let bytes=read_entry(archive,name,32_000_000,budget)?;
            let mut nested=zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_|"JAR incluido no legible")?;
            scan(&mut nested,owner,depth+1,budget,nodes)?;
        }
    }
    Ok(())
}

// Nested JARs are candidates, not automatically selected mods. Follow only
// uniquely required providers; alternatives remain the loader's decision.
fn required_candidates(nodes:&[Candidate],providers:&HashMap<&str,Vec<usize>>)->HashSet<usize> {
    let mut required:HashSet<_>=nodes.iter().enumerate().filter_map(|(index,node)|node.top_level.then_some(index)).collect();
    let mut pending:std::collections::VecDeque<_>=required.iter().copied().collect();
    while let Some(index)=pending.pop_front() {
        for dependency in nodes[index].dependencies.iter().filter(|dependency|dependency.kind=="depends") {
            if matches!(dependency.id.as_str(),"minecraft"|"java"|"fabricloader") {continue;}
            let eligible:Vec<_>=providers.get(dependency.id.as_str()).into_iter().flatten().copied()
                .filter(|&index|crate::fabric_version::matches(&nodes[index].version,&dependency.alternatives)!=Some(false)).collect();
            if eligible.iter().any(|index|required.contains(index)) {continue;}
            if eligible.len()==1&&required.insert(eligible[0]) {pending.push_back(eligible[0]);}
        }
    }
    required
}

pub fn review(base:&Path,files:&[InstalledContent])->Review{
    review_at(base,&base.join("mods"),files)
}

pub(crate) fn review_at(base:&Path,content:&Path,files:&[InstalledContent])->Review{
    let mut nodes=Vec::new();let mut budget=Budget{nodes:0,bytes:0};let mut complete=true;
    let mut issues:HashMap<String,Vec<String>>=HashMap::new();
    let mut blocking:HashMap<String,Vec<String>>=HashMap::new();
    for file in files {
        let path=content.join(&file.filename);
        let result=(||{
            let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
            if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>512_000_000{return Err("JAR no regular".into());}
            let mut archive=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|_|"JAR no legible")?;
            scan(&mut archive,&file.filename,0,&mut budget,&mut nodes)
        })();
        if let Err(error)=result{complete=false;issues.entry(file.filename.clone()).or_default().push(format!("Revisión de dependencias incompleta: {error}"));}
    }
    // Overrides can replace/remove depends. Preserve them and defer to the loader.
    let overrides=base.join("config/fabric_loader_dependencies.json");
    match fs::symlink_metadata(overrides){Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},_=>complete=false}
    if complete {
        let mut available:HashSet<&str>=["minecraft","java","fabricloader"].into_iter().collect();
        for node in &nodes{available.insert(&node.id);for alias in &node.provides{available.insert(alias);}}
        let mut providers:HashMap<&str,Vec<usize>>=HashMap::new();
        for (index,node) in nodes.iter().enumerate() {
            for id in std::iter::once(&node.id).chain(&node.provides) {
                let entries=providers.entry(id).or_default();if !entries.contains(&index) {entries.push(index);}
            }
        }
        let required=required_candidates(&nodes,&providers);
        for (_,node) in nodes.iter().enumerate().filter(|(index,_)|required.contains(index)) {
            for dependency in &node.dependencies {
                if dependency.kind=="depends"&&!available.contains(dependency.id.as_str()) {
                    let message=format!("Dependencia obligatoria sin proveedor encontrado: {} (requerida por {}). Rango declarado: {}",dependency.id,node.id,dependency.alternatives.join(" o "));
                    blocking.entry(node.owner.clone()).or_default().push(message.clone());
                    let row=issues.entry(node.owner.clone()).or_default();if !row.contains(&message){row.push(message);}
                }
                if matches!(dependency.kind.as_str(),"depends"|"breaks"|"conflicts"){
                    let providers:Vec<_>=providers.get(dependency.id.as_str()).into_iter().flatten().map(|&index|(index,&nodes[index])).collect();
                    let outcomes:Vec<_>=providers.iter().map(|(_,candidate)|crate::fabric_version::matches(&candidate.version,&dependency.alternatives)).collect();
                    let mismatch=dependency.kind=="depends"&&!providers.is_empty()&&outcomes.iter().all(|result|*result==Some(false));
                    // Optional nested candidates are not evidence that a forbidden mod is selected.
                    let incompatible=dependency.kind!="depends"&&providers.iter().zip(&outcomes).any(|((index,_),result)|required.contains(index)&&*result==Some(true));
                    if mismatch||incompatible{
                        let message=if mismatch{format!("Ninguna versión encontrada de {} cumple el requisito de {}: {}. Encontradas: {}",dependency.id,node.id,dependency.alternatives.join(" o "),providers.iter().map(|(_,candidate)|candidate.version.as_str()).collect::<Vec<_>>().join(", "))}else{format!("{} declara {} con {} en el rango {}",node.id,if dependency.kind=="breaks"{"incompatibilidad"}else{"conflicto"},dependency.id,dependency.alternatives.join(" o "))};
                        if mismatch||dependency.kind=="breaks" {blocking.entry(node.owner.clone()).or_default().push(message.clone());}
                        let row=issues.entry(node.owner.clone()).or_default();if !row.contains(&message){row.push(message);}
                    }
                }
            }
        }
    }
    Review{issues,blocking,note:if complete{"Dependencias Fabric: presencia y rangos reconocidos revisados para mods principales y proveedores incluidos obligatorios sin alternativas. Java, Minecraft, versión del loader y selección entre candidatos alternativos no comprobados; esto no confirma la carga."}else{"Dependencias Fabric: revisión incompleta por metadatos/JAR incluidos no legibles, límites de lectura o overrides. No se deducen dependencias ausentes ni incompatibilidades de versión."}.into()}
}
