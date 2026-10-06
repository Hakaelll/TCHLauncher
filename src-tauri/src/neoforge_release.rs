use serde_json::Value;

#[derive(Debug,PartialEq,Eq)]
pub struct Release {
    pub minecraft:String,
    pub version:String,
    pub artifact:&'static str,
    pub numbers:Vec<u32>,
}
impl Release {
    pub fn parse(version:&str)->Option<Self>{
        if let Some(value)=version.strip_prefix("1.20.1-"){
            let numbers=numbers(value)?;
            if numbers[0]!=47{return None;}
            return Some(Self{minecraft:"1.20.1".into(),version:version.into(),artifact:"forge",numbers});
        }
        let numbers=numbers(version)?;
        if numbers[0]<20{return None;}
        let minecraft=if numbers[0]<26 {
            if numbers[1]==0{format!("1.{}",numbers[0])}else{format!("1.{}.{}",numbers[0],numbers[1])}
        }else if numbers[1]==0{numbers[0].to_string()}else{format!("{}.{}",numbers[0],numbers[1])};
        Some(Self{minecraft,version:version.into(),artifact:"neoforge",numbers})
    }
    // Identity/layout only. Install callers still require exact catalog membership.
    pub fn selected(version:&str)->Option<Self>{
        if let Some(release)=Self::parse(version){return Some(release);}
        crate::fabric::validate_version(version).ok()?;
        let mut release=Self::parse(version.split('-').next()?)?;
        if release.artifact!="neoforge"{return None;}
        release.version=version.into();Some(release)
    }
    pub fn installer_url(&self)->String{
        format!("https://maven.neoforged.net/releases/net/neoforged/{}/{}/{}-{}-installer.jar",self.artifact,self.version,self.artifact,self.version)
    }
    pub fn profile_version(&self)->String{
        if self.artifact=="forge"{format!("1.20.1-forge-{}",self.version.strip_prefix("1.20.1-").unwrap())}else{format!("neoforge-{}",self.version)}
    }
    pub fn library_directory(&self)->String{format!("libraries/net/neoforged/{}/{}",self.artifact,self.version)}
}
fn numbers(version:&str)->Option<Vec<u32>>{
    if version.len()>64||!version.bytes().all(|value|value.is_ascii_digit()||value==b'.'){return None;}
    let values=version.split('.').map(str::parse::<u32>).collect::<Result<Vec<_>,_>>().ok()?;
    (values.len()==3).then_some(values)
}
pub fn stable(data:&Value,minecraft:&str)->Result<Vec<Release>,String>{
    let entries=data["versions"].as_array().ok_or("Catálogo NeoForge no válido")?;
    if entries.len()>100_000{return Err("El catálogo NeoForge supera el límite de versiones".into());}
    let mut releases=entries.iter().filter_map(Value::as_str).filter_map(Release::parse).filter(|release|release.minecraft==minecraft).collect::<Vec<_>>();
    releases.sort_by(|a,b|b.numbers.cmp(&a.numbers));releases.dedup_by(|a,b|a.version==b.version);
    Ok(releases)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn historical_artifacts_keep_their_exact_identity_and_numeric_order(){
        let releases=stable(&serde_json::json!({"versions":["1.20.1-47.1.9","1.20.1-47.1.106","47.1.82","1.20.1-47.1.106","1.20.1-47.1.107-beta","1.20.1-48.0.1"]}),"1.20.1").unwrap();
        assert_eq!(releases.len(),2);let release=&releases[0];assert_eq!(release.version,"1.20.1-47.1.106");
        assert_eq!(release.profile_version(),"1.20.1-forge-47.1.106");
        assert_eq!(release.installer_url(),"https://maven.neoforged.net/releases/net/neoforged/forge/1.20.1-47.1.106/forge-1.20.1-47.1.106-installer.jar");
        assert_eq!(release.library_directory(),"libraries/net/neoforged/forge/1.20.1-47.1.106");
    }
    #[test]
    fn modern_release_routes_and_calendar_version_mapping_remain_distinct(){
        let release=Release::parse("21.4.156").unwrap();assert_eq!(release.minecraft,"1.21.4");assert_eq!(release.artifact,"neoforge");assert_eq!(release.profile_version(),"neoforge-21.4.156");
        assert_eq!(Release::parse("26.1.1").unwrap().minecraft,"26.1");
        assert!(stable(&serde_json::json!({"versions":["21.4.156","1.20.1-47.1.106"]}),"1.19.4").unwrap().is_empty());
        for value in ["../47.1.106","1.20.1-47.1.106@jar","1.20.1-47.1.106/other","1.20.1-47.1.106-alpha","21.4.156-beta","19.4.1","1.20.1-999999999999999999999.1.1"]{assert!(Release::parse(value).is_none(),"accepted {value}");}
        assert!(stable(&serde_json::json!({"versions":null}),"1.20.1").is_err());
    }
}
