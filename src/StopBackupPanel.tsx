import {translatedMessage,t,formatDate} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api} from './api'
import {useServerRead} from './useServerRead'
export function StopBackupPanel({id,disabled,onChanged}:{id:string;disabled:boolean;onChanged:()=>Promise<void>}){
  const inventory=useServerRead(id,api.stopBackup),status=inventory.data
  const [working,setWorking]=useState(false),[error,setError]=useState('')
  const [eventError,setEventError]=useState('')
  const [relatedError,setRelatedError]=useState('')
  const owner=useRef({active:true,working:false})
  const changed=useRef(onChanged);changed.current=onChanged
  useEffect(()=>{
    const current={active:true,working:false};owner.current=current
    setWorking(false);setError('');setEventError('');setRelatedError('')
    const subscription=listen<string>('stop-backup-updated',event=>{
      if(current.active&&event.payload===id){
        void inventory.refresh(id)
        void changed.current().then(()=>{if(current.active)setRelatedError('')},reason=>{if(current.active)setRelatedError(String(reason))})
      }
    })
    void subscription.catch(()=>{if(current.active)setEventError(t("No se pudieron recibir cambios automáticos. Usa Actualizar estado."))})
    return()=>{current.active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id,inventory.refresh])
  const change=async(enabled:boolean)=>{
    const current=owner.current
    if(!current.active||current.working||disabled||inventory.loading||inventory.error||!status||status.phase==='copying')return
    current.working=true;setWorking(true);setError('')
    try{const value=await api.setStopBackup(id,enabled);if(current.active)inventory.acknowledge(id,value)}
    catch(reason){if(current.active)setError(String(reason))}
    finally{current.working=false;if(current.active)setWorking(false)}
  }
  return <section className="stop-backup-policy" aria-labelledby="stop-backup-title">
    <div className="stop-backup-heading"><h3 id="stop-backup-title">{t("Copia al detener")}</h3><button disabled={working||inventory.loading} onClick={()=>void inventory.refresh(id)}>{inventory.loading?t("Consultando…"):t("Actualizar estado")}</button></div>
    <label className="backup-policy-toggle"><input type="checkbox" checked={status?.enabled??false} disabled={disabled||working||inventory.loading||!!inventory.error||!status||status.phase==='copying'} onChange={event=>void change(event.target.checked)}/>{t("Guardar una copia después de una parada limpia")}</label>
    <p className="field-help">{t("Solo después de que Java termine correctamente y si el servidor llegó a estar listo. También al reiniciar; «Detener y salir» espera a que termine la copia. Se aplica tu política de retención.")}</p>
    {working&&<p role="status">{t("Guardando preferencia…")}</p>}
    {inventory.error&&<p className="form-error" role="alert">{translatedMessage(inventory.error)}{status?' '+t("Se muestra la última preferencia conocida."):''}</p>}
    {eventError&&<p className="inline-note">{translatedMessage(eventError)}</p>}
    {relatedError&&<p className="form-error" role="alert">{translatedMessage(relatedError)}</p>}
    {status?.lastResult&&<p className={status.phase==='error'?'form-error':'field-help'} role={status.phase==='error'?'alert':status.phase==='copying'?'status':undefined}>{status.lastAt&&`${formatDate(status.lastAt)} · `}{translatedMessage(status.lastResult)}</p>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
  </section>
}
