import {translatedMessage,t} from './locale.ts'
import {useCallback,useEffect,useRef,useState,type FormEvent} from 'react'
import {api,type BackupPolicy,type BackupRetentionPlan} from './api'
import {Modal} from './Modal'
import {StopBackupPanel} from './StopBackupPanel'
import {retentionPolicyErrors,sameRetentionPolicy} from './retentionPolicy'

const gb=1024**3
const bytes=(value:number)=>`${(value/gb).toFixed(2)} GB`
export function BackupPolicyPanel({id,disabled,onChanged}:{id:string;disabled:boolean;onChanged:()=>Promise<void>}) {
  const [saved,setSaved]=useState<BackupPolicy|null>(null)
  const [draft,setDraft]=useState<BackupPolicy>({enabled:false,maxCount:10,maxBytes:20*gb})
  const [working,setWorking]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [readError,setReadError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [loading,setLoading]=useState(false)
  const [submitted,setSubmitted]=useState(false)
  const [plan,setPlan]=useState<BackupRetentionPlan|null>(null)
  const countInput=useRef<HTMLInputElement>(null),spaceInput=useRef<HTMLInputElement>(null)
  const scope=useRef({id,active:false,revision:0,busy:false,saved:null as BackupPolicy|null})
  const load=useCallback(async(force=false)=>{
    const current=scope.current;if(!current.active||current.busy&&!force)return
    const revision=++current.revision;setLoading(true)
    try{
      const value=await api.backupPolicy(current.id)
      if(!current.active||scope.current!==current||revision!==current.revision)return
      const previous=current.saved;current.saved=value;setSaved(value)
      setDraft(draft=>!previous||sameRetentionPolicy(draft,previous)?{...value}:draft);setReadError(null)
    }catch(cause){if(current.active&&scope.current===current&&revision===current.revision)setReadError(String(cause))}
    finally{if(current.active&&scope.current===current&&revision===current.revision)setLoading(false)}
  },[])
  useEffect(()=>{
    const current={id,active:true,revision:0,busy:false,saved:null as BackupPolicy|null};scope.current=current
    setSaved(null);setDraft({enabled:false,maxCount:10,maxBytes:20*gb});setWorking(false);setError(null);setReadError(null);setNotice(null);setSubmitted(false);setPlan(null)
    void load();return()=>{current.active=false}
  },[id,load])
  const dirty=!!saved&&!sameRetentionPolicy(saved,draft)
  const errors=retentionPolicyErrors(draft)
  const operation=async(action:()=>Promise<void>)=>{
    const current=scope.current;if(current.busy)return
    current.busy=true;current.revision++;setLoading(false);setWorking(true);setError(null);setNotice(null)
    try{await action()}catch(cause){if(current.active&&scope.current===current)setError(String(cause))}
    finally{current.busy=false;if(current.active&&scope.current===current)setWorking(false)}
  }
  const save=(event:FormEvent)=>{
    event.preventDefault();setSubmitted(true)
    if(errors.maxCount){countInput.current?.focus();return}
    if(errors.maxBytes){spaceInput.current?.focus();return}
    if(errors.limits){countInput.current?.focus();return}
    const current=scope.current;const policy={...draft}
    void operation(async()=>{await api.saveBackupPolicy(current.id,policy);if(!current.active||scope.current!==current)return;current.saved=policy;setSaved(policy);setReadError(null);setSubmitted(false);setNotice(t("Política guardada."))})
  }
  const preview=()=>{
    const current=scope.current
    void operation(async()=>{const value=await api.previewBackupRetention(current.id);if(current.active&&scope.current===current)setPlan(value)})
  }
  const apply=()=>{
    if(!plan)return
    const current=scope.current;const fingerprint=plan.fingerprint
    void operation(async()=>{
      const result=await api.applyBackupRetention(current.id,fingerprint)
      if(!current.active||scope.current!==current)return
      setPlan(null);setNotice(`Limpieza completada: ${result.remove.length} copias eliminadas.${result.warnings.length?' '+result.warnings.join(' '):''}`)
      try{await onChanged()}catch(cause){if(current.active&&scope.current===current)setError(t("La limpieza terminó, pero no se pudo actualizar la lista: {0}", String(cause)))}
    })
  }
  const edit=(value:BackupPolicy)=>{setDraft(value);setNotice(null);setError(null)}
  const refresh=()=>{
    const current=scope.current
    void operation(async()=>{await load(true);if(!current.active||scope.current!==current)return;try{await onChanged()}catch(cause){throw new Error(t("No se pudo actualizar la lista de copias: {0}", String(cause)))}})
  }
  return <>
    <StopBackupPanel key={id} id={id} disabled={disabled} onChanged={onChanged}/>
    <details className="backup-policy"><summary><span>{t("Retención de copias")}</span><small>{saved?saved.enabled?t("Activada"):t("Desactivada"):readError?t("No disponible"):t("Cargando…")}</small></summary>
      <form noValidate onSubmit={save}><fieldset aria-busy={working||undefined} disabled={disabled||working||!saved}>
        <label className="backup-policy-toggle"><input type="checkbox" checked={draft.enabled} onChange={event=>edit({...draft,enabled:event.target.checked})}/>{t("Eliminar automáticamente las copias más antiguas")}</label>
        <p className="muted">{t("Se aplicará al crear nuevas copias. Siempre se conserva la más reciente; la copia usada para restaurar queda protegida durante la operación.")}</p>
        <div className="backup-policy-fields"><label>{t("Máximo de copias")}<input ref={countInput} type="number" min={1} max={1000} step={1} placeholder={t("Sin límite")} value={draft.maxCount??''} aria-invalid={submitted&&!!(errors.maxCount||errors.limits)} aria-describedby={submitted&&(errors.maxCount||errors.limits)?'retention-count-error':undefined} onChange={event=>edit({...draft,maxCount:event.target.value===''?null:Number(event.target.value)})}/>{submitted&&(errors.maxCount||errors.limits)&&<span className="form-error" id="retention-count-error">{errors.maxCount??errors.limits}</span>}</label><label>{t("Espacio máximo (GB)")}<input ref={spaceInput} type="number" min={1/1024} max={2048} step="any" placeholder={t("Sin límite")} value={draft.maxBytes===null?'':draft.maxBytes/gb} aria-invalid={submitted&&!!errors.maxBytes} aria-describedby={submitted&&errors.maxBytes?'retention-space-error':undefined} onChange={event=>edit({...draft,maxBytes:event.target.value===''?null:Math.round(Number(event.target.value)*gb)})}/>{submitted&&errors.maxBytes&&<span className="form-error" id="retention-space-error">{errors.maxBytes}</span>}</label></div>
        <div className="backup-policy-actions"><button className="primary-button" type="submit" disabled={!dirty}>{working?t("Procesando…"):t("Guardar política")}</button><button type="button" disabled={dirty||!saved?.enabled||!!readError} onClick={preview}>{t("Revisar limpieza de copias")}</button></div>
      </fieldset></form>
      <div className="backup-policy-refresh"><span>{dirty?t("Cambios sin guardar"):''}</span><button disabled={working||loading} onClick={refresh}>{loading?t("Consultando…"):t("Actualizar")}</button></div>
      {notice&&<p className="field-help" role="status">{translatedMessage(notice)}</p>}
      {readError&&<p className="form-error" role="alert">{translatedMessage(readError)}</p>}
      {error&&!plan&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    </details>
    {plan&&<Modal labelledBy="retention-title" busy={working} onClose={()=>{setPlan(null);setError(null)}}>
      <h2 id="retention-title">{t("Limpiar copias antiguas")}</h2>
      <p>{plan.remove.length?t("Se eliminarán {0} copias. Esta acción no se puede deshacer.", plan.remove.length):t("No hay copias que eliminar con esta política.")}</p>
      <p className="muted">{t("Espacio:")} {bytes(plan.beforeBytes)} → {bytes(plan.afterBytes)}</p>
      <div className="retention-list">{plan.remove.map(copy=><div key={copy.id}><span><time dateTime={new Date(copy.createdAt).toISOString()}>{new Date(copy.createdAt).toLocaleString()}</time><small>{copy.minecraftVersion?`${copy.platform??t("Motor no registrado")} · Minecraft ${copy.minecraftVersion}`:t("Versión no registrada")}</small></span><strong>{bytes(copy.sizeBytes)}</strong></div>)}</div>
      {plan.warnings.map(message=><p className="inline-note" key={message}>{translatedMessage(message)}</p>)}
      {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>{t("Cancelar")}</button><button className="danger-button" disabled={working||disabled||!plan.remove.length} onClick={apply}>{working?t("Eliminando…"):t("Eliminar copias")}</button></div>
    </Modal>}
  </>
}
