use crate::model::Server;
use std::{fs,io::Read,path::{Path,PathBuf}};

pub struct Identity {pub version:String,pub evidence:Vec<PathBuf>}

pub fn checked_file(root:&Path,path:&Path,limit:u64)->Result<(),String> {
    let canonical=root.canonicalize().map_err(|e|e.to_string())?;
    let relative=path.strip_prefix(root).or_else(|_|path.strip_prefix(&canonical)).map_err(|_|"El archivo del loader está fuera del servidor")?;
    if relative.components().any(|part|!matches!(part,std::path::Component::Normal(_))) {return Err("Ruta del loader no válida".into());}
    let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
    if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>limit||path.canonicalize().map_err(|e|e.to_string())?!=canonical.join(relative){return Err("El archivo del loader contiene enlaces o supera el tamaño permitido".into());}Ok(())
}
fn entry(path:&Path,name:&str)->Result<String,String> {
    let mut zip=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|_|"El launcher importado no es un JAR válido")?;
    let item=zip.by_name(name).map_err(|_|format!("Falta {name} en el launcher importado"))?;
    if item.size()>131072{return Err("Los metadatos del loader superan 128 KB".into());}
    let mut text=String::new();item.take(131073).read_to_string(&mut text).map_err(|_|"Metadatos del loader no válidos")?;
    if text.len()>131072{return Err("Los metadatos del loader superan 128 KB".into());}Ok(text)
}
fn attribute(text:&str,key:&str)->Result<String,String> {
    let prefix=format!("{key}: ");let mut values=text.lines().take_while(|line|!line.is_empty()).filter_map(|line|line.strip_prefix(&prefix));
    let value=values.next().ok_or_else(||format!("Falta {key} en el manifiesto"))?;
    if values.next().is_some(){return Err("El manifiesto declara valores contradictorios".into());}Ok(value.into())
}
fn property(text:&str,key:&str)->Result<String,String> {
    let prefix=format!("{key}=");let mut values=text.lines().filter_map(|line|line.trim().strip_prefix(&prefix));
    let value=values.next().ok_or_else(||format!("Falta {key} en los metadatos Fabric"))?;
    crate::fabric::validate_version(value)?;
    if values.next().is_some(){return Err("Metadatos Fabric ambiguos".into());}Ok(value.into())
}
fn artifact_version(path:&str,group:&str,artifact:&str)->Option<String> {
    let suffix=path.strip_prefix(&format!("libraries/{group}/{artifact}/"))?;
    let (version,file)=suffix.split_once('/')?;
    (file==format!("{artifact}-{version}.jar")).then(||version.into())
}
pub fn imported(server:&Server)->Result<Identity,String> {
    let root=Path::new(&server.directory);let jar=Path::new(server.jar_path.as_ref().ok_or("Falta el arranque importado")?);
    checked_file(root,jar,64*1024*1024)?;
    if server.platform=="neoforge" {
        let args=crate::neoforge::launch_arguments(server)?;
        let mut values=args.windows(2).filter(|pair|pair[0]=="--fml.neoForgeVersion").map(|pair|pair[1].clone());
        let version=values.next().ok_or("El arranque no identifica la versión NeoForge")?;
        crate::fabric::validate_version(&version)?;
        crate::neoforge::requested_loader(&serde_json::json!({"versions":[version]}),&server.version,Some(&version))?;
        if values.next().is_some(){return Err("El arranque NeoForge identifica varias versiones".into());}
        let relative=format!("libraries/net/neoforged/neoforge/{version}/{}",jar.file_name().ok_or("Falta el arranque")?.to_string_lossy());
        let canonical=root.canonicalize().map_err(|e|e.to_string())?;
        if jar.canonicalize().map_err(|e|e.to_string())?!=canonical.join(relative){return Err("El arranque NeoForge no coincide con su versión".into());}
        return Ok(Identity{version,evidence:vec![jar.into()]});
    }
    if server.platform!="fabric"{return Err("El servidor no tiene un loader compatible".into());}
    let manifest=entry(jar,"META-INF/MANIFEST.MF")?.replace("\r\n ","").replace("\n ","");
    let main=attribute(&manifest,"Main-Class")?;
    if main=="net.fabricmc.installer.ServerLauncher" {
        let config=entry(jar,"install.properties")?;
        let version=property(&config,"fabric-loader-version")?;
        if property(&config,"game-version")?!=server.version{return Err("El launcher Fabric no corresponde a Minecraft".into());}
        return Ok(Identity{version,evidence:vec![jar.into()]});
    }
    if !["net.fabricmc.loader.impl.launch.server.FabricServerLauncher","net.fabricmc.loader.launch.server.FabricServerLauncher"].contains(&main.as_str()){return Err("El JAR no identifica un launcher de servidor Fabric compatible".into());}
    let classpath=attribute(&manifest,"Class-Path")?;let references=classpath.split_whitespace().collect::<Vec<_>>();
    if references.is_empty()||references.len()>100{return Err("Classpath Fabric no válido".into());}
    let base=jar.parent().ok_or("Falta la carpeta del launcher Fabric")?;let mut loader=None;let mut minecraft=None;let mut evidence=vec![jar.into()];
    for reference in references {
        let path=crate::modpack::relative_path(reference)?;let library=base.join(&path);
        checked_file(root,&library,64*1024*1024)?;
        if let Some(version)=artifact_version(&path,"net/fabricmc","fabric-loader") {
            crate::fabric::validate_version(&version)?;
            let metadata:serde_json::Value=serde_json::from_str(&entry(&library,"fabric.mod.json")?).map_err(|_|"Fabric Loader tiene metadatos no válidos")?;
            if metadata["id"].as_str()!=Some("fabricloader")||metadata["version"].as_str()!=Some(&version)||loader.replace(version).is_some(){return Err("Las bibliotecas Fabric no identifican un único loader".into());}
            evidence.push(library);
        }
        if let Some(version)=artifact_version(&path,"net/fabricmc","intermediary") {
            if minecraft.replace(version).is_some(){return Err("Fabric declara varias versiones de Minecraft".into());}
        }
    }
    if minecraft.as_deref()!=Some(&server.version){return Err("Las bibliotecas Fabric no corresponden a Minecraft".into());}
    Ok(Identity{version:loader.ok_or("No se identificó la versión de Fabric Loader")?,evidence})
}

#[cfg(test)]
mod tests {
    use super::*;use std::io::Write;use uuid::Uuid;
    struct Fixture(PathBuf,Server);
    impl Drop for Fixture {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
    fn fixture(platform:&str)->Fixture {
        let root=std::env::temp_dir().join(format!("tch imported identity {}",Uuid::new_v4()));
        let mut server=crate::storage::create(&root,crate::model::CreateServer{name:"Imported".into(),version:"1.21.4".into(),platform:platform.into(),memory_min_mb:512,memory_max_mb:1024,max_players:2,port:25565,storage_target_gb:2,image_path:None}).unwrap();
        server.jar_path=Some(Path::new(&server.directory).join("renamed.jar").to_string_lossy().into());Fixture(root,server)
    }
    fn jar(path:&Path,entries:&[(&str,&str)]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();let mut zip=zip::ZipWriter::new(fs::File::create(path).unwrap());
        for (name,value) in entries{zip.start_file(*name,zip::write::SimpleFileOptions::default()).unwrap();zip.write_all(value.as_bytes()).unwrap();}zip.finish().unwrap();
    }
    #[test]
    fn unattended_fabric_uses_embedded_identity_and_rejects_mismatch() {
        let fixture=fixture("fabric");let server=&fixture.1;let path=Path::new(server.jar_path.as_ref().unwrap());
        let manifest="Manifest-Version: 1.0\r\nMain-Class: net.fabricmc.installer.ServerLauncher\r\n\r\n";
        jar(path,&[("META-INF/MANIFEST.MF",manifest),("install.properties","fabric-loader-version=0.16.10\ngame-version=1.21.4\n")]);
        assert_eq!(imported(server).unwrap().version,"0.16.10");
        for config in ["fabric-loader-version=0.16.10\ngame-version=1.21.5", "fabric-loader-version=0.16.10\nfabric-loader-version=0.19.5\ngame-version=1.21.4"] {
            jar(path,&[("META-INF/MANIFEST.MF",manifest),("install.properties",config)]);assert!(imported(server).is_err());
        }
        jar(path,&[("META-INF/MANIFEST.MF","Main-Class: net.fabricmc.installer.Main\n\n"),("install.properties","fabric-loader-version=0.16.10\ngame-version=1.21.4")]);assert!(imported(server).is_err());
    }
    #[test]
    fn classic_fabric_reads_loader_metadata_and_checks_referenced_libraries() {
        let fixture=fixture("fabric");let server=&fixture.1;let root=Path::new(&server.directory);let path=Path::new(server.jar_path.as_ref().unwrap());
        let loader="libraries/net/fabricmc/fabric-loader/0.16.10/fabric-loader-0.16.10.jar";let intermediary="libraries/net/fabricmc/intermediary/1.21.4/intermediary-1.21.4.jar";
        jar(&root.join(loader),&[("fabric.mod.json",r#"{"id":"fabricloader","version":"0.16.10"}"#)]);
        jar(&root.join(intermediary),&[("marker","test")]);
        let manifest=format!("Manifest-Version: 1.0\r\nMain-Class: net.fabricmc.loader.impl.launch.server.FabricServerLauncher\r\nClass-Path: {loader}\r\n {intermediary}\r\n\r\n");
        // Manifest continuation concatenates bytes, so retain the separator.
        let manifest=manifest.replace(".jar\r\n ",".jar \r\n ");
        jar(path,&[("META-INF/MANIFEST.MF",&manifest)]);assert_eq!(imported(server).unwrap().version,"0.16.10");
        let first=crate::loader_update::installed(server).unwrap().1;
        jar(&root.join(loader),&[("fabric.mod.json",r#"{"id":"fabricloader","version":"0.16.10","modified":true}"#)]);
        assert_ne!(crate::loader_update::installed(server).unwrap().1,first);
        jar(path,&[("META-INF/MANIFEST.MF",&manifest.replace(intermediary,"../outside.jar"))]);assert!(imported(server).is_err());
        jar(path,&[("META-INF/MANIFEST.MF",&manifest)]);fs::remove_file(root.join(intermediary)).unwrap();assert!(imported(server).is_err());
    }
    #[test]
    fn neoforge_requires_matching_arguments_and_version_directory() {
        let mut fixture=fixture("neoforge");let root=Path::new(&fixture.1.directory);let path=root.join("libraries/net/neoforged/neoforge/21.4.157/win_args.txt");
        let path=if cfg!(windows){path}else{path.with_file_name("unix_args.txt")};
        fs::create_dir_all(path.parent().unwrap()).unwrap();fs::create_dir_all(root.join("libraries/test")).unwrap();fs::write(root.join("libraries/test/bootstrap.jar"),b"test").unwrap();
        fixture.1.jar_path=Some(path.to_string_lossy().into());
        let args="-p libraries/test/bootstrap.jar\nnet.neoforged.fml.startup.Server\n--fml.mcVersion 1.21.4\n--fml.neoForgeVersion 21.4.157\n";
        fs::write(&path,args).unwrap();assert_eq!(imported(&fixture.1).unwrap().version,"21.4.157");
        for text in [args.replace("21.4.157","21.4.158"),format!("{args}--fml.neoForgeVersion 21.4.157\n"),args.replace("1.21.4","1.21.5")] {fs::write(&path,text).unwrap();assert!(imported(&fixture.1).is_err());}
    }
}
