//! Read-only HotSpot performance counters. No JMX port or forced collections.
use serde::Serialize;
use std::{collections::BTreeMap,fs,io::Read,path::Path};
const LIMIT:usize=1_048_576;
#[derive(Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Reading{pub used_bytes:u64,pub committed_bytes:u64,pub collections:u64,pub sampled_at:i64}
fn parse(bytes:&[u8])->Result<Reading,String>{
    let bad=||"Los contadores de memoria de Java no son válidos".to_string();
    if bytes.len()<32||bytes[..4]!=[0xca,0xfe,0xc0,0xc0]||bytes[4]>1||bytes[5]!=2||bytes[7]!=1{return Err(bad());}
    let little=bytes[4]==1;
    let int=|offset:usize|->Result<u32,String>{let value: [u8;4]=bytes.get(offset..offset+4).ok_or_else(bad)?.try_into().map_err(|_|bad())?;Ok(if little{u32::from_le_bytes(value)}else{u32::from_be_bytes(value)})};
    let used=int(8)?as usize;let mut offset=int(24)?as usize;let count=int(28)?as usize;
    if used>bytes.len()||used>LIMIT||offset<32||offset>used||count>8192{return Err(bad());}
    let mut counters=BTreeMap::new();
    for _ in 0..count{
        if offset.checked_add(20).filter(|end|*end<=used).is_none(){return Err(bad());}
        let length=int(offset)?as usize;
        let end=offset.checked_add(length).filter(|end|*end<=used).ok_or_else(bad)?;
        if length<20{return Err(bad());}
        let name_offset=int(offset+4)?as usize;let data_offset=int(offset+16)?as usize;
        if name_offset<20||name_offset>=length{return Err(bad());}
        let name_bytes=&bytes[offset+name_offset..end];let terminator=name_bytes.iter().position(|byte|*byte==0).ok_or_else(bad)?;
        let name=std::str::from_utf8(&name_bytes[..terminator]).map_err(|_|bad())?;
        if bytes[offset+12]==b'J'&&int(offset+8)?==0{
            if data_offset<20||data_offset.checked_add(8).filter(|end|*end<=length).is_none(){return Err(bad());}
            let raw:[u8;8]=bytes[offset+data_offset..offset+data_offset+8].try_into().map_err(|_|bad())?;
            let number=if little{i64::from_le_bytes(raw)}else{i64::from_be_bytes(raw)};
            if number<0{return Err(bad());}
            if counters.insert(name.to_string(),(number as u64,bytes[offset+14])).is_some(){return Err(bad());}
        }
        offset=end;
    }
    let generations=counters.get("sun.gc.policy.generations").map(|counter|counter.0).filter(|count|(1..=8).contains(count)).ok_or("Este recolector no publica generaciones de heap compatibles")?;
    let mut used_bytes=0u64;let mut committed_bytes=0u64;
    let add=|total:&mut u64,value:u64|->Result<(),String>{*total=total.checked_add(value).ok_or_else(bad)?;Ok(())};
    for generation in 0..generations{
        let prefix=format!("sun.gc.generation.{generation}");
        let (capacity,unit)=*counters.get(&format!("{prefix}.capacity")).ok_or_else(bad)?;if unit!=2{return Err(bad());}add(&mut committed_bytes,capacity)?;
        let spaces=counters.get(&format!("{prefix}.spaces")).map(|counter|counter.0).filter(|count|(1..=16).contains(count)).ok_or_else(bad)?;
        for space in 0..spaces{let (value,unit)=*counters.get(&format!("{prefix}.space.{space}.used")).ok_or_else(bad)?;if unit!=2{return Err(bad());}add(&mut used_bytes,value)?;}
    }
    if committed_bytes==0||used_bytes>committed_bytes{return Err("La muestra de Java cambió durante la lectura; se comprobará de nuevo".into());}
    let mut collections=0;let mut collectors=0;
    for (name,(value,unit)) in counters{
        if let Some(number)=name.strip_prefix("sun.gc.collector.").and_then(|v|v.strip_suffix(".invocations")){
            if number.parse::<u8>().is_ok()&&unit==4{add(&mut collections,value)?;collectors+=1;}
        }
    }
    if collectors==0{return Err("Java no publica contadores del recolector".into());}
    Ok(Reading{used_bytes,committed_bytes,collections,sampled_at:chrono::Utc::now().timestamp_millis()})
}
fn read_file(path:&Path)->Result<Reading,String>{
    let meta=fs::symlink_metadata(path).map_err(|e|format!("No se pudieron abrir los contadores Java: {e}"))?;
    if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>LIMIT as u64{return Err("Archivo de contadores Java no válido".into());}
    let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take((LIMIT+1)as u64).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len()>LIMIT{return Err("Los contadores Java superan el tamaño admitido".into());}parse(&bytes)
}
pub fn inspect(pid:u32)->Result<Reading,String>{
    let mut roots=vec![std::env::temp_dir()];
    #[cfg(unix)]if !roots.contains(&std::path::PathBuf::from("/tmp")){roots.push("/tmp".into());}
    for root in roots.drain(..){
        for key in ["USERNAME","USER"]{
            if let Ok(user)=std::env::var(key){
                if !user.is_empty()&&!user.contains(['/', '\\']){
                    let folder=root.join(format!("hsperfdata_{user}"));
                    if fs::symlink_metadata(&folder).is_ok_and(|meta|meta.is_dir()&&!meta.file_type().is_symlink()){
                        let path=folder.join(pid.to_string());if path.exists(){return read_file(&path);}
                    }
                }
            }
        }
        let Ok(entries)=fs::read_dir(root)else{continue};
        for entry in entries.take(512).flatten(){
            if !entry.file_name().to_string_lossy().starts_with("hsperfdata_"){continue;}
            if !entry.file_type().is_ok_and(|kind|kind.is_dir()&&!kind.is_symlink()){continue;}
            let path=entry.path().join(pid.to_string());
            if path.exists(){return read_file(&path);}
        }
    }
    Err("Java no publica contadores de heap accesibles. Puede estar iniciando o tener PerfData desactivado.".into())
}
