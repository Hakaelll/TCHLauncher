//! Bounded, native-only memory observations; no heap dumps or forced GC.
use crate::model::{Metrics,Server};
use serde::Serialize;
use std::{collections::{BTreeMap,VecDeque},sync::Mutex};

const WINDOW_MS:i64=30*60*1000;
const MAX_SAMPLES:usize=901;
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Sample {
    #[serde(skip)]
    session:String,
    sampled_at:i64,pid:u32,uptime_seconds:u64,resident_bytes:u64,
    heap_used_bytes:Option<u64>,heap_committed_bytes:Option<u64>,collector_events:Option<u64>,
}
#[derive(Default)]
pub struct History(Mutex<BTreeMap<String,VecDeque<Sample>>>);
impl History {
    pub fn record(&self,m:&Metrics,session:&str) {
        let (Some(pid),Some(uptime),Some(resident))=(m.pid,m.uptime_seconds,m.memory_bytes) else{return;};
        let Ok(mut histories)=self.0.lock() else{return;};
        if !histories.contains_key(&m.server_id)&&histories.len()>=32 {
            if let Some(oldest)=histories.iter().min_by_key(|(_,samples)|samples.back().map(|s|s.sampled_at)).map(|(id,_)|id.clone()){histories.remove(&oldest);}
        }
        let samples=histories.entry(m.server_id.clone()).or_default();
        if let Some(last)=samples.back(){
            if m.sampled_at<=last.sampled_at{return;}
            if last.session!=session||last.pid!=pid||uptime<last.uptime_seconds{samples.clear();}
        }
        samples.push_back(Sample{session:session.into(),sampled_at:m.sampled_at,pid,uptime_seconds:uptime,resident_bytes:resident,
            heap_used_bytes:m.heap.as_ref().map(|h|h.used_bytes),heap_committed_bytes:m.heap.as_ref().map(|h|h.committed_bytes),collector_events:m.heap.as_ref().map(|h|h.collections)});
        while samples.len()>MAX_SAMPLES||samples.front().is_some_and(|s|m.sampled_at.saturating_sub(s.sampled_at)>WINDOW_MS){samples.pop_front();}
    }
    pub fn report(&self,server:&Server)->Result<Vec<u8>,String>{
        let histories=self.0.lock().map_err(|e|e.to_string())?;
        let samples=histories.get(&server.id).filter(|s|!s.is_empty()).ok_or("Aún no hay muestras de memoria. Inicia el servidor y deja abierta la aplicación durante la partida.")?;
        serde_json::to_vec_pretty(&serde_json::json!({
            "schemaVersion":1,"launcherVersion":env!("CARGO_PKG_VERSION"),"exportedAt":chrono::Utc::now().timestamp_millis(),
            "minecraftVersion":server.version,"platform":server.platform,
            "configuredInitialHeapMiB":server.memory_min_mb,"configuredMaximumHeapMiB":server.memory_max_mb,
            "configurationNote":"Current launcher configuration, not a measurement of JVM limits at launch.",
            "observationNote":"Native samples while the launcher was open. Gaps are not interpolated. Only the latest observed process session is retained. Collector events include concurrent cycles; used heap includes objects not yet collected. This report does not establish a memory leak.",
            "samples":samples
        })).map_err(|e|e.to_string())
    }
}

pub fn write_report(destination:&std::path::Path,bytes:&[u8])->Result<(),String>{
    use std::{fs,io::Write};
    let temporary=destination.with_extension(format!("{}.part",uuid::Uuid::new_v4()));
    let result=(||{
        let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e|e.to_string())?;
        file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|e|e.to_string())?;
        drop(file);
        // The native save dialog chooses the destination. Preserve it until publication.
        if destination.exists(){
            if !fs::symlink_metadata(destination).map_err(|e|e.to_string())?.is_file(){return Err("El destino debe ser un archivo normal".into());}
            let previous=destination.with_extension(format!("{}.previous",uuid::Uuid::new_v4()));
            fs::rename(destination,&previous).map_err(|e|e.to_string())?;
            if let Err(error)=fs::rename(&temporary,destination){
                if let Err(restore)=fs::rename(&previous,destination){return Err(format!("{error}; no se pudo restaurar el archivo anterior ({restore}); conservado en {}",previous.display()));}
                return Err(error.to_string());
            }
            let _=fs::remove_file(previous);
        }else{fs::rename(&temporary,destination).map_err(|e|e.to_string())?;}
        Ok(())
    })();
    if result.is_err(){let _=fs::remove_file(temporary);}
    result
}

#[cfg(test)]
mod tests{
    use super::*;
    fn metric(time:i64,pid:u32,uptime:u64)->Metrics{
        Metrics{server_id:"one".into(),sampled_at:time,pid:Some(pid),uptime_seconds:Some(uptime),startup_duration_ms:None,startup_phase:None,memory_bytes:Some(4096),heap:Some(crate::heap::Reading{used_bytes:1024,committed_bytes:2048,collections:3,sampled_at:time}),heap_error:Some("PRIVATE PATH".into()),cpu_percent:None,disk_read_bytes:None,disk_written_bytes:None,size_bytes:None,size_error:None,size_sampled_at:None,size_skipped_links:None,storage_warning_percent:80,tps:crate::ticks::Snapshot{reading:None,pending:false}}
    }
    fn server()->Server{Server{id:"one".into(),name:"PRIVATE NAME".into(),version:"1.21.11".into(),platform:"fabric".into(),directory:"PRIVATE PATH".into(),jar_path:None,java_path:None,memory_min_mb:1024,memory_max_mb:12288,storage_target_gb:20,created_at:String::new(),state:"running".into(),image_data:None,pinned:false}}
    #[test]
    fn bounded_sessions_and_private_fields(){
        let history=History::default();let server=server();assert!(history.report(&server).is_err());
        for n in 0..1000{history.record(&metric(n*2000,42,n as u64*2),"a");}
        let json:serde_json::Value=serde_json::from_slice(&history.report(&server).unwrap()).unwrap();
        let samples=json["samples"].as_array().unwrap();assert_eq!(samples.len(),901);assert_eq!(samples[0]["sampledAt"],198000);assert_eq!(samples[900]["collectorEvents"],3);
        assert!(!json.to_string().contains("PRIVATE"));assert_eq!(json["configuredMaximumHeapMiB"],12288);
        let mut unavailable=metric(2000000,42,2000);unavailable.heap=None;history.record(&unavailable,"a");
        let json:serde_json::Value=serde_json::from_slice(&history.report(&server).unwrap()).unwrap();assert!(json["samples"].as_array().unwrap().last().unwrap()["heapUsedBytes"].is_null());
        history.record(&metric(2001000,43,1),"b");history.record(&metric(2000500,42,2000),"a");
        let json:serde_json::Value=serde_json::from_slice(&history.report(&server).unwrap()).unwrap();assert_eq!(json["samples"].as_array().unwrap().len(),1);assert_eq!(json["samples"][0]["pid"],43);
        history.record(&metric(2002000,43,0),"b");assert_eq!(history.0.lock().unwrap()["one"].len(),1);
        history.record(&metric(2003000,43,5),"c");assert_eq!(history.0.lock().unwrap()["one"].len(),1);
    }
    #[test]
    fn publication_preserves_unrelated_files_and_rejects_directories(){
        let root=std::env::temp_dir().join(format!("memory-report-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&root).unwrap();
        let path=root.join("report.json");write_report(&path,b"first").unwrap();write_report(&path,b"second").unwrap();assert_eq!(std::fs::read(&path).unwrap(),b"second");
        assert!(write_report(&root,b"bad").is_err());assert_eq!(std::fs::read_dir(&root).unwrap().count(),1);assert_eq!(std::fs::read(&path).unwrap(),b"second");
        std::fs::remove_dir_all(root).unwrap();
    }
}
