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
        let flag=if server.version=="1.20.1"{"--fml.forgeVersion"}else{"--fml.neoForgeVersion"};
        let mut values=args.windows(2).filter(|pair|pair[0]==flag).map(|pair|if server.version=="1.20.1"{format!("1.20.1-{}",pair[1])}else{pair[1].clone()});
        let version=values.next().ok_or("El arranque no identifica la versión NeoForge")?;
        crate::fabric::validate_version(&version)?;
        crate::neoforge::requested_loader(&serde_json::json!({"versions":[version]}),&server.version,Some(&version))?;
        if values.next().is_some(){return Err("El arranque NeoForge identifica varias versiones".into());}
        let release=crate::neoforge_release::Release::selected(&version).ok_or("Identidad NeoForge no válida")?;
        let relative=Path::new(&release.library_directory()).join(jar.file_name().ok_or("Falta el arranque")?);
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
