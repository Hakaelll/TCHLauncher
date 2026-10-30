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
        let (Some(pid),Some(uptime))=(m.pid,m.uptime_seconds) else{return;};
        let Ok(mut histories)=self.0.lock() else{return;};
        if let Some(samples)=histories.get_mut(&m.server_id){
            if let Some(last)=samples.back(){
                if last.session!=session||last.pid!=pid||uptime<last.uptime_seconds{samples.clear();}
                else if m.sampled_at<=last.sampled_at{return;}
            }
        }
        let Some(resident)=m.memory_bytes else{return;};
        if !histories.contains_key(&m.server_id)&&histories.len()>=32 {
            if let Some(oldest)=histories.iter().min_by_key(|(_,samples)|samples.back().map(|s|s.sampled_at)).map(|(id,_)|id.clone()){histories.remove(&oldest);}
        }
        let samples=histories.entry(m.server_id.clone()).or_default();
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
