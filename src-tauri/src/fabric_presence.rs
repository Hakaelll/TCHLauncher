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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn jar(value:serde_json::Value,nested:Vec<(&str,Vec<u8>)>)->Vec<u8>{
        let mut archive=zip::ZipWriter::new(Cursor::new(Vec::new()));
        archive.start_file("fabric.mod.json",zip::write::SimpleFileOptions::default()).unwrap();
        archive.write_all(value.to_string().as_bytes()).unwrap();
        for(name,bytes)in nested{archive.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();archive.write_all(&bytes).unwrap();}
        archive.finish().unwrap().into_inner()
    }
    fn root()->std::path::PathBuf {
        let root=std::env::temp_dir().join(format!("tch-presence-{}",uuid::Uuid::new_v4()));fs::create_dir_all(root.join("mods")).unwrap();root
    }
    fn audit(root:&Path)->crate::content_diagnostics::Audit{
        let server=serde_json::from_value(serde_json::json!({"id":"presence","name":"Presence","version":"1.21.11","platform":"fabric","directory":root.to_string_lossy(),"jarPath":null,"javaPath":null,"memoryMinMb":1024,"memoryMaxMb":2048,"storageTargetGb":20,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap();
        crate::content_diagnostics::audit(&server).unwrap()
    }
    #[test]
    fn nested_providers_aliases_and_transitive_missing_requirements_are_read_only(){
        let root=root();
        let inner=jar(serde_json::json!({"schemaVersion":1,"id":"library","version":"1","provides":["aliaslib"],"depends":{"missingtransitive":"*"}}),vec![]);
        let outer=jar(serde_json::json!({"schemaVersion":1,"id":"backpack","version":"1","jars":[{"file":"META-INF/jars/library.jar"}],"depends":{"aliaslib":"*","minecraft":">=1.21","fabricloader":">=0.16","java":">=21"},"recommends":{"optional":"*"}}),vec![("META-INF/jars/library.jar",inner)]);
        fs::write(root.join("mods/backpack.jar"),&outer).unwrap();
        let result=audit(&root);let issues=&result.checks[0].issues;
        assert_eq!(issues.len(),1);assert!(issues[0].contains("missingtransitive"));assert!(issues[0].contains("requerida por library"));
        assert!(result.log_note.contains("presencia y rangos"));assert_eq!(fs::read(root.join("mods/backpack.jar")).unwrap(),outer);
        fs::write(root.join("mods/provider.jar"),jar(serde_json::json!({"schemaVersion":1,"id":"missingtransitive","version":"1"}),vec![])).unwrap();
        assert!(audit(&root).checks.iter().all(|check|check.issues.is_empty()));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn client_only_provider_does_not_satisfy_server_dependency(){
        let root=root();
        for(name,value)in [("dependent.jar",serde_json::json!({"schemaVersion":1,"id":"dependent","version":"1","depends":{"clientlib":"*"}})),("client.jar",serde_json::json!({"schemaVersion":1,"id":"clientlib","version":"1","environment":"client"}))]{fs::write(root.join("mods").join(name),jar(value,vec![])).unwrap();}
        let result=audit(&root);assert!(result.checks.iter().find(|c|c.filename=="dependent.jar").unwrap().issues.iter().any(|issue|issue.contains("clientlib")));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn partial_metadata_and_dependency_overrides_never_become_negative_evidence(){
        let root=root();
        let clean=jar(serde_json::json!({"schemaVersion":1,"id":"dependent","version":"1","depends":{"absent":"*"}}),vec![]);
        fs::write(root.join("mods/dependent.jar"),&clean).unwrap();
        let bad=jar(serde_json::json!({"schemaVersion":1,"id":"partial","version":"1","jars":[{"file":"not-found.jar"}]}),vec![]);
        fs::write(root.join("mods/partial.jar"),bad).unwrap();
        let result=audit(&root);assert!(result.log_note.contains("incompleta"));assert!(!result.checks.iter().any(|check|check.issues.iter().any(|s|s.contains("sin proveedor"))));
        fs::remove_file(root.join("mods/partial.jar")).unwrap();fs::create_dir(root.join("config")).unwrap();fs::write(root.join("config/fabric_loader_dependencies.json"),"{}").unwrap();
        assert!(audit(&root).checks[0].issues.is_empty());assert_eq!(fs::read(root.join("mods/dependent.jar")).unwrap(),clean);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn incompatible_versions_and_declared_breaks_are_related_to_the_requiring_file(){
        let root=root();
        let dependent=jar(serde_json::json!({"schemaVersion":1,"id":"backpack","version":"1","depends":{"library":[">=2 <3","4.x"]},"breaks":{"oldmod":"<2"},"conflicts":{"othermod":"*"}}),vec![]);
        fs::write(root.join("mods/backpack.jar"),&dependent).unwrap();
        for(id,version)in[("library","1.9"),("oldmod","1.0"),("othermod","2")]{fs::write(root.join("mods").join(format!("{id}.jar")),jar(serde_json::json!({"schemaVersion":1,"id":id,"version":version}),vec![])).unwrap();}
        let result=audit(&root);let issues=&result.checks.iter().find(|check|check.filename=="backpack.jar").unwrap().issues;
        assert_eq!(issues.len(),3);assert!(issues.iter().any(|issue|issue.contains("Ninguna versión")&&issue.contains("1.9")));assert!(issues.iter().any(|issue|issue.contains("incompatibilidad")&&issue.contains("oldmod")));assert!(issues.iter().any(|issue|issue.contains("conflicto")&&issue.contains("othermod")));
        fs::write(root.join("mods/library.jar"),jar(serde_json::json!({"schemaVersion":1,"id":"library","version":"4.1"}),vec![])).unwrap();
        assert!(!audit(&root).checks.iter().any(|check|check.issues.iter().any(|issue|issue.contains("Ninguna versión"))));
        fs::create_dir(root.join("config")).unwrap();fs::write(root.join("config/fabric_loader_dependencies.json"),"{}").unwrap();
        assert!(audit(&root).checks.iter().all(|check|check.issues.is_empty()));assert_eq!(fs::read(root.join("mods/backpack.jar")).unwrap(),dependent);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn optional_nested_candidates_never_prove_a_declared_break_is_selected(){
        let root=root();
        let nested=jar(serde_json::json!({"schemaVersion":1,"id":"optional","version":"1"}),vec![]);
        let main=jar(serde_json::json!({"schemaVersion":1,"id":"mainmod","version":"1","jars":[{"file":"optional.jar"}],"breaks":{"optional":"*"}}),vec![("optional.jar",nested)]);
        fs::write(root.join("mods/main.jar"),main).unwrap();assert!(audit(&root).checks[0].issues.is_empty());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unused_nested_dependency_does_not_block_but_unique_required_provider_does(){
        let root=root();
        let nested=jar(serde_json::json!({"schemaVersion":1,"id":"optional","version":"1","depends":{"absent":"*"}}),vec![]);
        for required in [false,true] {
            let mut value=serde_json::json!({"schemaVersion":1,"id":"mainmod","version":"1","jars":[{"file":"optional.jar"}]});
            if required {value["depends"]=serde_json::json!({"optional":"*"});}
            let main=jar(value,vec![("optional.jar",nested.clone())]);
            fs::write(root.join("mods/main.jar"),&main).unwrap();
            let files=crate::content_storage::list(&serde_json::from_value(serde_json::json!({"id":"x","name":"x","version":"1.21.4","platform":"fabric","directory":root.to_string_lossy(),"jarPath":null,"javaPath":null,"memoryMinMb":512,"memoryMaxMb":1024,"storageTargetGb":2,"createdAt":"","state":"stopped","imageData":null,"pinned":false})).unwrap()).unwrap();
            let result=review(&root,&files);
            assert_eq!(!result.blocking.is_empty(),required);
            assert_eq!(!audit(&root).checks[0].issues.is_empty(),required);
            assert_eq!(fs::read(root.join("mods/main.jar")).unwrap(),main);
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn alternative_nested_versions_do_not_require_each_candidates_dependencies(){
        let root=root();
        let bad=jar(serde_json::json!({"schemaVersion":1,"id":"library","version":"1","depends":{"absent":"*"}}),vec![]);
        let good=jar(serde_json::json!({"schemaVersion":1,"id":"library","version":"2"}),vec![]);
        for (range,has_issue) in [("*",false),(">=2",false),("<2",true)] {
            let main=jar(serde_json::json!({"schemaVersion":1,"id":"mainmod","version":"1","jars":[{"file":"bad.jar"},{"file":"good.jar"}],"depends":{"library":range}}),vec![("bad.jar",bad.clone()),("good.jar",good.clone())]);
            fs::write(root.join("mods/main.jar"),main).unwrap();
            assert_eq!(!audit(&root).checks[0].issues.is_empty(),has_issue,"{range}");
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn nested_paths_depth_and_decompression_budgets_are_bounded(){
        let unsafe_jar=jar(serde_json::json!({"schemaVersion":1,"id":"unsafe","version":"1","jars":[{"file":"../outside.jar"}]}),vec![]);
        let mut archive=zip::ZipArchive::new(Cursor::new(unsafe_jar)).unwrap();let mut nodes=Vec::new();let mut budget=Budget{nodes:0,bytes:0};
        assert!(scan(&mut archive,"unsafe.jar",0,&mut budget,&mut nodes).is_err());
        assert!(scan(&mut archive,"unsafe.jar",9,&mut budget,&mut nodes).is_err());
        let mut budget=Budget{nodes:0,bytes:127_999_999};assert!(budget.claim(2).is_err());assert_eq!(budget.bytes,127_999_999);
        assert!(budget.claim(32_000_001).is_err());
    }
}
