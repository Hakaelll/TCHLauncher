import {useEffect,useState} from 'react'
import {api,type BackupPolicy,type BackupRetentionPlan} from './api'
import {Modal} from './Modal'
import {StopBackupPanel} from './StopBackupPanel'

const gb=1024**3
const bytes=(value:number)=>`${(value/gb).toFixed(2)} GB`
export function BackupPolicyPanel({id,disabled,onChanged}:{id:string;disabled:boolean;onChanged:()=>Promise<void>}) {
  const [saved,setSaved]=useState<BackupPolicy|null>(null)
  const [draft,setDraft]=useState<BackupPolicy>({enabled:false,maxCount:10,maxBytes:20*gb})
  const [working,setWorking]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [plan,setPlan]=useState<BackupRetentionPlan|null>(null)
  useEffect(()=>{let active=true;void api.backupPolicy(id).then(value=>{if(active){setSaved(value);setDraft(value)}}).catch(cause=>{if(active)setError(String(cause))});return()=>{active=false}},[id])
  const dirty=JSON.stringify(saved)!==JSON.stringify(draft)
  const save=async()=>{setWorking(true);setError(null);try{await api.saveBackupPolicy(id,draft);setSaved({...draft})}catch(cause){setError(String(cause))}finally{setWorking(false)}}
  const preview=async()=>{setWorking(true);setError(null);try{setPlan(await api.previewBackupRetention(id))}catch(cause){setError(String(cause))}finally{setWorking(false)}}
  const apply=async()=>{if(!plan)return;setWorking(true);setError(null);try{await api.applyBackupRetention(id,plan.fingerprint);await onChanged();setPlan(null)}catch(cause){setError(String(cause))}finally{setWorking(false)}}
  return <>
    <StopBackupPanel key={id} id={id} disabled={disabled} onChanged={onChanged}/>
    <details className="backup-policy"><summary><span>Retención de copias</span><small>{saved?saved.enabled?'Activada':'Desactivada':'Cargando…'}</small></summary>
      <fieldset aria-busy={working||undefined} disabled={disabled||working||!saved}>
        <label className="backup-policy-toggle"><input type="checkbox" checked={draft.enabled} onChange={event=>setDraft({...draft,enabled:event.target.checked})}/>Eliminar automáticamente las copias más antiguas</label>
        <p className="muted">Se aplicará al crear nuevas copias. Siempre se conserva la más reciente; la copia usada para restaurar queda protegida durante la operación.</p>
        <div className="backup-policy-fields"><label>Máximo de copias<input type="number" min={1} max={1000} placeholder="Sin límite" value={draft.maxCount??''} onChange={event=>setDraft({...draft,maxCount:event.target.value===''?null:Number(event.target.value)})}/></label><label>Espacio máximo (GB)<input type="number" min={1/1024} max={2048} step="any" placeholder="Sin límite" value={draft.maxBytes===null?'':draft.maxBytes/gb} onChange={event=>setDraft({...draft,maxBytes:event.target.value===''?null:Math.round(Number(event.target.value)*gb)})}/></label></div>
        <div className="backup-policy-actions"><button disabled={!dirty} onClick={save}>Guardar política</button><button disabled={dirty||!saved?.enabled} onClick={preview}>Revisar limpieza de copias</button></div>
      </fieldset>
      {error&&!plan&&<p className="form-error" role="alert">{error}</p>}
    </details>
    {plan&&<Modal labelledBy="retention-title" busy={working} onClose={()=>{setPlan(null);setError(null)}}>
      <h2 id="retention-title">Limpiar copias antiguas</h2>
      <p>{plan.remove.length?`Se eliminarán ${plan.remove.length} copias. Esta acción no se puede deshacer.`:'No hay copias que eliminar con esta política.'}</p>
      <p className="muted">Espacio: {bytes(plan.beforeBytes)} → {bytes(plan.afterBytes)}</p>
      <div className="retention-list">{plan.remove.map(copy=><div key={copy.id}><span>{new Date(copy.createdAt).toLocaleString()}</span><strong>{bytes(copy.sizeBytes)}</strong></div>)}</div>
      {plan.warnings.map(message=><p className="inline-note" key={message}>{message}</p>)}
      {error&&<p className="form-error" role="alert">{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>Cancelar</button><button className="danger-button" disabled={working||disabled||!plan.remove.length} onClick={apply}>{working?'Eliminando…':'Eliminar copias'}</button></div>
    </Modal>}
  </>
}
