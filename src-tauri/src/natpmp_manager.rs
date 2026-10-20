use crate::{natpmp,storage};
use rusqlite::{params,OptionalExtension};
use serde::{Deserialize,Serialize};
use std::{net::Ipv4Addr,path::Path,sync::Mutex,time::Duration};

const LEASE:u32=3600;
fn now()->i64{chrono::Utc::now().timestamp_millis()}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Record{
    pub server_id:String,pub local_ip:Ipv4Addr,pub gateway:Ipv4Addr,pub internal_port:u16,
    pub external_port:Option<u16>,pub external_ip:Option<Ipv4Addr>,pub phase:String,
    pub expires_at:i64,pub confirmed_at:Option<i64>,pub renew_at:i64,pub retry_at:i64,pub last_error:Option<String>,
}
impl Record{
    fn validate(&self)->Result<(),String>{
        let id=uuid::Uuid::parse_str(&self.server_id).map_err(|_|"Registro NAT-PMP no válido")?;
        if id.to_string()!=self.server_id||self.internal_port==0||self.external_port==Some(0)||self.expires_at<0||self.renew_at<0||self.retry_at<0||!matches!(self.phase.as_str(),"pending"|"active"|"closing"|"attention")||self.last_error.as_ref().is_some_and(|error|error.len()>4096){return Err("Registro NAT-PMP no válido".into());}
        natpmp::Client::new(self.local_ip,self.gateway)?;Ok(())
    }
}
fn db(root:&Path)->Result<rusqlite::Connection,String>{
    let db=storage::connect(root)?;db.busy_timeout(Duration::from_secs(2)).map_err(|e|e.to_string())?;
    // No FK: deleting a server must not erase pending network cleanup.
    db.execute_batch("CREATE TABLE IF NOT EXISTS natpmp_mappings(server_id TEXT PRIMARY KEY,gateway TEXT NOT NULL,local_ip TEXT NOT NULL,port INTEGER NOT NULL,payload TEXT NOT NULL,UNIQUE(gateway,local_ip,port));").map_err(|e|e.to_string())?;Ok(db)
}
fn read(row:&rusqlite::Row<'_>)->Result<Record,String>{
    let payload:String=row.get(4).map_err(|e|e.to_string())?;if payload.len()>8192{return Err("Registro NAT-PMP demasiado grande".into());}
    let record:Record=serde_json::from_str(&payload).map_err(|_|"Registro NAT-PMP no válido")?;record.validate()?;
    if record.server_id!=row.get::<_,String>(0).map_err(|e|e.to_string())?||record.gateway.to_string()!=row.get::<_,String>(1).map_err(|e|e.to_string())?||record.local_ip.to_string()!=row.get::<_,String>(2).map_err(|e|e.to_string())?||record.internal_port!=row.get::<_,u16>(3).map_err(|e|e.to_string())?{return Err("El registro NAT-PMP no coincide con su destino".into());}Ok(record)
}
type StoredRecord=(String,Result<Record,String>);
fn records(root:&Path)->Result<Vec<StoredRecord>,String>{
    let db=db(root)?;let mut query=db.prepare("SELECT server_id,gateway,local_ip,port,payload FROM natpmp_mappings ORDER BY server_id").map_err(|e|e.to_string())?;
    let mut rows=query.query([]).map_err(|e|e.to_string())?;let mut result=Vec::new();while let Some(row)=rows.next().map_err(|e|e.to_string())?{let id=row.get::<_,String>(0).ok().and_then(|value|uuid::Uuid::parse_str(&value).ok()).map(|id|id.to_string()).unwrap_or_else(||"invalid-record".into());result.push((id,read(row)));}Ok(result)
}
fn load(root:&Path,id:&str)->Result<Option<Record>,String>{
    db(root)?.query_row("SELECT server_id,gateway,local_ip,port,payload FROM natpmp_mappings WHERE server_id=?1",[id],|row|Ok(read(row))).optional().map_err(|e|e.to_string())?.transpose()
}
fn save(root:&Path,record:&Record,insert:bool)->Result<(),String>{
    record.validate()?;let db=db(root)?;let payload=serde_json::to_string(record).map_err(|e|e.to_string())?;
    if insert{db.execute("INSERT INTO natpmp_mappings VALUES(?1,?2,?3,?4,?5)",params![record.server_id,record.gateway.to_string(),record.local_ip.to_string(),record.internal_port,payload]).map_err(|_|"Ya hay una apertura NAT-PMP registrada para este servidor o puerto")?;}
    else{let changed=db.execute("UPDATE natpmp_mappings SET payload=?5 WHERE server_id=?1 AND gateway=?2 AND local_ip=?3 AND port=?4",params![record.server_id,record.gateway.to_string(),record.local_ip.to_string(),record.internal_port,payload]).map_err(|e|e.to_string())?;if changed!=1{return Err("La apertura NAT-PMP cambió durante la operación".into());}}Ok(())
}
fn forget(root:&Path,record:&Record)->Result<(),String>{
    db(root)?.execute("DELETE FROM natpmp_mappings WHERE server_id=?1 AND gateway=?2 AND local_ip=?3 AND port=?4",params![record.server_id,record.gateway.to_string(),record.local_ip.to_string(),record.internal_port]).map_err(|e|e.to_string())?;Ok(())
}
trait Transport{
    fn address(&self)->Result<natpmp::Address,String>;
    fn open(&self,internal:u16,external:u16,lifetime:u32)->Result<natpmp::Mapping,String>;
    fn close(&self,internal:u16)->Result<(),String>;
}
impl Transport for natpmp::Client{
    fn address(&self)->Result<natpmp::Address,String>{self.address()}
    fn open(&self,i:u16,e:u16,l:u32)->Result<natpmp::Mapping,String>{self.open(i,e,l)}
    fn close(&self,i:u16)->Result<(),String>{self.close(i)}
}
fn close_record(root:&Path,record:&mut Record,transport:&impl Transport,time:i64)->Result<(),String>{
    if time>=record.expires_at{return forget(root,record);}
    record.phase="closing".into();save(root,record,false)?;
    match transport.close(record.internal_port){Ok(())=>forget(root,record),Err(error)=>{record.phase="attention".into();record.last_error=Some(error.clone());record.retry_at=time+15_000;save(root,record,false)?;Err(error)}}
}
fn renew(root:&Path,record:&mut Record,transport:&impl Transport,time:i64)->Result<(),String>{
    // Persist a conservative lease ceiling before a request whose reply can be lost.
    record.phase="pending".into();record.expires_at=time+7_200_000;record.retry_at=time+15_000;save(root,record,false)?;
    let result=transport.open(record.internal_port,record.external_port.unwrap_or(record.internal_port),LEASE);
    match result{
        Ok(mapping)=>{
            record.external_port=Some(mapping.external_port);record.expires_at=time+i64::from(mapping.lifetime_seconds)*1000;
            record.renew_at=time+i64::from(mapping.lifetime_seconds)*500;record.retry_at=0;record.confirmed_at=Some(time);record.phase="active".into();record.last_error=None;
            if let Ok(address)=transport.address(){record.external_ip=Some(address.external_ip);}else{record.external_ip=None;}
            save(root,record,false)
        },
        Err(error)=>{record.phase="attention".into();record.last_error=Some(error.clone());save(root,record,false)?;Err(error)}
    }
}
#[derive(Default)]
pub struct Manager{serial:Mutex<()>}
impl Manager{
    pub fn status(&self,root:&Path,id:&str)->Result<Option<Record>,String>{load(root,id)}
    pub fn detect(local:Ipv4Addr,gateway:Ipv4Addr)->Result<String,String>{natpmp::Client::new(local,gateway)?.address().map(|value|value.external_ip.to_string())}
    pub fn open(&self,root:&Path,id:&str,local:Ipv4Addr,gateway:Ipv4Addr,port:u16,valid:impl Fn()->bool)->Result<Record,String>{
        let _lock=self.serial.lock().map_err(|_|"La configuración NAT-PMP está ocupada")?;
        let transport=natpmp::Client::new(local,gateway)?;
        self.open_inner(root,id,local,gateway,port,&transport,valid,now())
    }
    fn open_inner(&self,root:&Path,id:&str,local:Ipv4Addr,gateway:Ipv4Addr,port:u16,transport:&impl Transport,valid:impl Fn()->bool,time:i64)->Result<Record,String>{
        if !valid(){return Err("El servidor ya no está disponible en el destino confirmado".into());}
        storage::get(root,id)?;
        let mut record=Record{server_id:id.into(),local_ip:local,gateway,internal_port:port,external_port:None,external_ip:None,phase:"pending".into(),expires_at:time+7_200_000,confirmed_at:None,renew_at:time,retry_at:time,last_error:None};
        save(root,&record,true)?;
        let result=renew(root,&mut record,transport,time);
        if result.is_err()||!valid(){let _=close_record(root,&mut record,transport,time);return Err(result.err().unwrap_or_else(||"El servidor se detuvo; se solicitó retirar la apertura".into()));}
        Ok(record)
    }
    pub fn close(&self,root:&Path,id:&str)->Result<(),String>{
        let _lock=self.serial.lock().map_err(|_|"La configuración NAT-PMP está ocupada")?;
        if let Some(mut record)=load(root,id)?{let transport=natpmp::Client::new(record.local_ip,record.gateway)?;close_record(root,&mut record,&transport,now())?;}Ok(())
    }
    pub fn maintain(&self,root:&Path,valid:impl Fn(&str,Ipv4Addr,u16)->bool)->Result<(),String>{
        self.maintain_inner(root,valid,|record|natpmp::Client::new(record.local_ip,record.gateway),now())
    }
    fn maintain_inner<T:Transport>(&self,root:&Path,valid:impl Fn(&str,Ipv4Addr,u16)->bool,transport:impl Fn(&Record)->Result<T,String>,time:i64)->Result<(),String>{
        let Ok(_lock)=self.serial.try_lock()else{return Ok(());};
        let mut errors=Vec::new();
        for (id,loaded) in records(root)?{
            let result=(||{
                let mut record=loaded?;
                if record.retry_at>time{return Ok(());}
                let transport=transport(&record)?;
                if record.phase!="active"||!valid(&record.server_id,record.local_ip,record.internal_port){
                    close_record(root,&mut record,&transport,time)?;
                }else if time>=record.renew_at{
                    let result=renew(root,&mut record,&transport,time);
                    if result.is_err()||!valid(&record.server_id,record.local_ip,record.internal_port){
                        close_record(root,&mut record,&transport,time)?;
                    }
                    result?;
                }
                Ok::<(),String>(())
            })();
            if let Err(error)=result{errors.push(format!("{id}: {error}"));}
        }
        if errors.is_empty(){Ok(())}else{Err(errors.join("; "))}
    }
    fn close_all_inner<T:Transport>(&self,root:&Path,transport:impl Fn(&Record)->Result<T,String>,time:i64)->Result<(),String>{
        let _lock=self.serial.lock().map_err(|_|"La configuración NAT-PMP está ocupada")?;
        let mut errors=Vec::new();
        for (id,loaded) in records(root)?{
            let result=(||{
                let mut record=loaded?;
                let transport=transport(&record)?;
                close_record(root,&mut record,&transport,time)
            })();
            if let Err(error)=result{errors.push(format!("{id}: {error}"));}
        }
        if errors.is_empty(){Ok(())}else{Err(errors.join("; "))}
    }
    pub fn close_all(&self,root:&Path){
        if let Err(error)=self.close_all_inner(root,|record|natpmp::Client::new(record.local_ip,record.gateway),now()){
            eprintln!("NAT-PMP cleanup: {error}");
        }
    }
}
