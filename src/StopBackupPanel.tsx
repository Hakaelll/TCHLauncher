import {translatedMessage,t} from './locale.ts'
import {useEffect,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type StopBackupStatus} from './api'
export function StopBackupPanel({id,disabled,onChanged}:{id:string;disabled:boolean;onChanged:()=>Promise<void>}){
  const [status,setStatus]=useState<StopBackupStatus|null>(null)
  const [working,setWorking]=useState(false),[error,setError]=useState('')
  useEffect(()=>{
    let alive=true
    const load=()=>void api.stopBackup(id).then(value=>{if(alive)setStatus(value)},reason=>{if(alive)setError(String(reason))})
    load();const subscription=listen<string>('stop-backup-updated',event=>{if(event.payload===id){load();void onChanged().catch(reason=>{if(alive)setError(String(reason))})}})
    return()=>{alive=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id])
  const change=async(enabled:boolean)=>{
    setWorking(true);setError('')
    try{await api.setStopBackup(id,enabled);setStatus(await api.stopBackup(id))}catch(reason){setError(String(reason))}finally{setWorking(false)}
  }
  return <section className="stop-backup-policy" aria-labelledby="stop-backup-title">
    <h3 id="stop-backup-title">{t("Copia al detener")}</h3>
    <label className="backup-policy-toggle"><input type="checkbox" checked={status?.enabled??false} disabled={disabled||working||!status||status.phase==='copying'} onChange={event=>void change(event.target.checked)}/>{t("Guardar una copia después de una parada limpia")}</label>
    <p className="field-help">{t("Solo después de que Java termine correctamente y si el servidor llegó a estar listo. También al reiniciar; «Detener y salir» espera a que termine la copia. Se aplica tu política de retención.")}</p>
    {working&&<p role="status">{t("Guardando preferencia…")}</p>}
    {status?.lastResult&&<p className={status.phase==='error'?'form-error':'field-help'} role={status.phase==='copying'?'status':undefined}>{status.lastAt&&`${new Date(status.lastAt).toLocaleString()} · `}{translatedMessage(status.lastResult)}</p>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
  </section>
}
