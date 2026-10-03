import {useEffect,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type CrashPolicy,type CrashStatus} from './api'

export function CrashRestartPanel({id}:{id:string}) {
  const [status,setStatus]=useState<CrashStatus|null>(null)
  const [draft,setDraft]=useState<CrashPolicy>({enabled:false,delaySeconds:30,maxAttempts:3})
  const [error,setError]=useState<string|null>(null)
  const [working,setWorking]=useState(false)
  useEffect(()=>{
    let active=true
    void api.crashRestart(id).then(value=>{if(active){setStatus(value);setDraft(value.policy)}}).catch(cause=>{if(active)setError(String(cause))})
    const subscription=listen<string>('scheduler-updated',event=>{if(event.payload===id)void api.crashRestart(id).then(value=>{if(active)setStatus(value)}).catch(cause=>{if(active)setError(String(cause))})})
    return()=>{active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id])
  const dirty=JSON.stringify(status?.policy)!==JSON.stringify(draft)
  const save=async(policy:CrashPolicy)=>{setWorking(true);setError(null);try{await api.saveCrashRestart(id,policy);const value=await api.crashRestart(id);setStatus(value);setDraft(value.policy)}catch(cause){setError(String(cause))}finally{setWorking(false)}}
  return <div className="crash-restart-panel">
    <div className="crash-restart-heading"><h4>Reinicio tras un fallo</h4><span className="task-state">{status?status.policy.enabled?'Activado':'Desactivado':'Cargando…'}</span></div>
    <fieldset disabled={working||!status}>
      <label className="schedule-repeat"><input type="checkbox" checked={draft.enabled} onChange={event=>setDraft({...draft,enabled:event.target.checked})}/>Volver a iniciar si el proceso termina inesperadamente</label>
      <p className="field-help">Las paradas solicitadas no se reinician. El contador se restablece tras 10 minutos de funcionamiento estable; guardar estos ajustes también lo restablece.</p>
      <div className="backup-policy-fields"><label>Espera entre intentos (segundos)<input type="number" min={5} max={3600} step={1} value={draft.delaySeconds} onChange={event=>setDraft({...draft,delaySeconds:Number(event.target.value)})}/></label><label>Límite de intentos<input type="number" min={1} max={10} step={1} value={draft.maxAttempts} onChange={event=>setDraft({...draft,maxAttempts:Number(event.target.value)})}/></label></div>
      <div className="backup-policy-actions"><button disabled={!dirty} onClick={()=>void save(draft)}>Guardar reinicio automático</button>{status&&status.attempts>0&&<button disabled={dirty} onClick={()=>void save(status.policy)}>Restablecer intentos</button>}</div>
    </fieldset>
    {status?.policy.enabled&&<p className="crash-status" role="status"><strong>{status.attempts} / {status.policy.maxAttempts} intentos usados</strong>{status.nextAt&&<span>Próximo intento: {new Date(status.nextAt).toLocaleString()}</span>}{status.lastResult&&<span>{status.lastResult}</span>}</p>}
    {error&&<p className="form-error" role="alert">{error}</p>}
  </div>
}
