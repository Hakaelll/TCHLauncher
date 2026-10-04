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

#[cfg(test)]mod tests{
    use super::*;
    fn fixture(little:bool)->Vec<u8>{
        let mut bytes=vec![0;32];bytes[..4].copy_from_slice(&[0xca,0xfe,0xc0,0xc0]);bytes[4]=u8::from(little);bytes[5]=2;bytes[7]=1;
        let entries=[("sun.gc.policy.generations",1,1),("sun.gc.generation.0.capacity",1024,2),("sun.gc.generation.0.spaces",1,1),("sun.gc.generation.0.space.0.used",256,2),("sun.gc.collector.0.invocations",3,4)];
        let integer=|value:u32|if little{value.to_le_bytes()}else{value.to_be_bytes()};
        for (name,value,unit)in entries{
            let offset=bytes.len();let data_offset=20+name.len()+1;let length=data_offset+8;bytes.resize(offset+length,0);
            bytes[offset..offset+4].copy_from_slice(&integer(length as u32));bytes[offset+4..offset+8].copy_from_slice(&integer(20));bytes[offset+12]=b'J';bytes[offset+14]=unit;bytes[offset+16..offset+20].copy_from_slice(&integer(data_offset as u32));bytes[offset+20..offset+20+name.len()].copy_from_slice(name.as_bytes());
            let value:i64=value;bytes[offset+data_offset..offset+length].copy_from_slice(&if little{value.to_le_bytes()}else{value.to_be_bytes()});
        }
        let used=bytes.len()as u32;bytes[8..12].copy_from_slice(&integer(used));bytes[24..28].copy_from_slice(&integer(32));bytes[28..32].copy_from_slice(&integer(entries.len()as u32));bytes
    }
    #[test]fn reads_endianness_and_rejects_incomplete_or_corrupt_data(){
        for little in [true,false]{let bytes=fixture(little);let sample=parse(&bytes).unwrap();assert_eq!((sample.used_bytes,sample.committed_bytes,sample.collections),(256,1024,3));for length in 0..bytes.len(){assert!(parse(&bytes[..length]).is_err());}}
        let mut bytes=fixture(true);bytes[7]=0;assert!(parse(&bytes).is_err());bytes[7]=1;bytes[24..28].copy_from_slice(&u32::MAX.to_le_bytes());assert!(parse(&bytes).is_err());
    }
    #[test]
    #[ignore="Requires TCH_TEST_JAVA; allocates a private 64 MiB Java test array, never Minecraft"]
    fn reads_actual_hotspot_heap_without_jdk_tools(){
        use std::{process::{Command,Stdio},time::{Duration,Instant}};
        let java=std::env::var("TCH_TEST_JAVA").expect("Set TCH_TEST_JAVA");
        let root=std::env::temp_dir().join(format!("tch-heap-{}",uuid::Uuid::new_v4()));fs::create_dir(&root).unwrap();
        // Authored Java 8 class: allocate byte[64 MiB], sleep 5 s, use array.length.
        // Holding the local across sleep keeps the allocation observable without forcing GC.
        let class=[202,254,186,190,0,0,0,52,0,17,1,0,9,72,101,97,112,80,114,111,98,101,7,0,1,1,0,16,106,97,118,97,47,108,97,110,103,47,79,98,106,101,99,116,7,0,3,1,0,4,109,97,105,110,1,0,22,40,91,76,106,97,118,97,47,108,97,110,103,47,83,116,114,105,110,103,59,41,86,1,0,4,67,111,100,101,1,0,16,106,97,118,97,47,108,97,110,103,47,84,104,114,101,97,100,7,0,8,1,0,5,115,108,101,101,112,1,0,4,40,74,41,86,12,0,10,0,11,10,0,9,0,12,5,0,0,0,0,0,0,19,136,3,4,0,0,0,0,33,0,2,0,4,0,0,0,0,0,1,0,9,0,5,0,6,0,1,0,7,0,0,0,27,0,2,0,2,0,0,0,15,18,16,188,8,76,20,0,14,184,0,13,43,190,87,177,0,0,0,0,0,0];
        fs::write(root.join("HeapProbe.class"),class).unwrap();
        struct Probe{child:std::process::Child,root:std::path::PathBuf}
        impl Drop for Probe{fn drop(&mut self){let _=self.child.kill();let _=self.child.wait();let _=fs::remove_dir_all(&self.root);}}
        let mut command=Command::new(java);
        #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
        let errors=fs::File::create(root.join("java-error.log")).unwrap();
        let child=command.args(["-Xms128M","-Xmx256M","-XX:+UseG1GC","-cp",".","HeapProbe"]).current_dir(&root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(errors).spawn().unwrap();
        let mut probe=Probe{child,root};let deadline=Instant::now()+Duration::from_secs(4);let mut last=String::new();
        loop{
            if let Ok(reading)=inspect(probe.child.id()){
                if reading.used_bytes>=64*1024*1024{assert!(reading.committed_bytes>=reading.used_bytes);assert!(reading.committed_bytes<=256*1024*1024);break;}
                last=format!("Heap too small: {}",reading.used_bytes);
            }else if let Err(error)=inspect(probe.child.id()){last=error;}
            assert!(probe.child.try_wait().unwrap().is_none(),"Java stopped: {last}. {}",fs::read_to_string(probe.root.join("java-error.log")).unwrap_or_default());
            assert!(Instant::now()<deadline,"No usable heap reading: {last}");std::thread::sleep(Duration::from_millis(50));
        }
    }
}
