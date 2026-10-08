import {translatedMessage,t,formatDate} from './locale.ts'
import {useCallback,useEffect,useRef,useState,type FormEvent} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type CrashPolicy,type CrashStatus} from './api'
import {crashPolicyErrors,defaultCrashPolicy,reconcileCrashDraft,sameCrashPolicy} from './crashPolicy'

export function CrashRestartPanel({id}:{id:string}) {
  const [status,setStatus]=useState<CrashStatus|null>(null)
  const [draft,setDraft]=useState<CrashPolicy>({...defaultCrashPolicy})
  const [readError,setReadError]=useState<string|null>(null)
  const [actionError,setActionError]=useState<string|null>(null)
  const [saved,setSaved]=useState(false)
  const [submitted,setSubmitted]=useState(false)
  const [working,setWorking]=useState(false)
  const [loading,setLoading]=useState(false)
  const delayInput=useRef<HTMLInputElement>(null)
  const attemptsInput=useRef<HTMLInputElement>(null)
  const scope=useRef({id,active:false,revision:0,saving:false,status:null as CrashStatus|null})
  const load=useCallback(async()=>{
    const current=scope.current
    if(!current.active||current.saving)return
    const revision=++current.revision;setLoading(true)
    try{
      const value=await api.crashRestart(current.id)
      if(!current.active||scope.current!==current||revision!==current.revision)return
      const previous=current.status?.policy??null
      current.status=value;setStatus(value)
      setDraft(draft=>reconcileCrashDraft(draft,previous,value.policy));setReadError(null)
    }catch(cause){if(current.active&&scope.current===current&&revision===current.revision)setReadError(String(cause))}
    finally{if(current.active&&scope.current===current&&revision===current.revision)setLoading(false)}
  },[])
  useEffect(()=>{
    const current={id,active:true,revision:0,saving:false,status:null as CrashStatus|null};scope.current=current
    setStatus(null);setDraft({...defaultCrashPolicy});setReadError(null);setActionError(null);setSaved(false);setSubmitted(false);setWorking(false)
    void load()
    const subscription=listen<string>('scheduler-updated',event=>{if(current.active&&event.payload===id)void load()})
    void subscription.catch(()=>{if(current.active&&scope.current===current)setReadError(t("No se pudieron recibir cambios automáticos. Usa Actualizar estado."))})
    return()=>{current.active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id,load])
  const dirty=!!status&&!sameCrashPolicy(status.policy,draft)
  const errors=crashPolicyErrors(draft)
  const save=async(policy:CrashPolicy)=>{
    const current=scope.current
    if(current.saving||!current.status)return
    current.saving=true;current.revision++;setLoading(false);setWorking(true);setActionError(null);setSaved(false)
    try{
      await api.saveCrashRestart(current.id,policy)
      if(!current.active||scope.current!==current)return
      // Acknowledged writes remain successful even when the subsequent read fails.
      current.status=null;setStatus(null);setDraft({...policy});setSaved(true);setSubmitted(false)
    }catch(cause){if(current.active&&scope.current===current)setActionError(String(cause))}
    finally{
      current.saving=false
      if(current.active&&scope.current===current){await load();if(current.active&&scope.current===current)setWorking(false)}
    }
  }
  const submit=(event:FormEvent)=>{
    event.preventDefault();setSubmitted(true)
    if(errors.delaySeconds){delayInput.current?.focus();return}
    if(errors.maxAttempts){attemptsInput.current?.focus();return}
    void save(draft)
  }
  const edit=(value:CrashPolicy)=>{setDraft(value);setSaved(false);setActionError(null)}
  return <section className="crash-restart-panel" aria-labelledby="crash-restart-title">
    <div className="crash-restart-heading"><h4 id="crash-restart-title">{t("Reinicio tras un fallo")}</h4><span className="task-state">{status?status.policy.enabled?t("Activado"):t("Desactivado"):readError?t("No disponible"):saved?t("Actualizando estado…"):t("Cargando…")}</span></div>
    <form noValidate onSubmit={submit}>
    <fieldset disabled={working||!status}>
      <label className="schedule-repeat"><input type="checkbox" checked={draft.enabled} onChange={event=>edit({...draft,enabled:event.target.checked})}/>{t("Volver a iniciar si el proceso termina inesperadamente")}</label>
      <p className="field-help">{t("Las paradas solicitadas no se reinician. El contador se restablece tras 10 minutos de funcionamiento estable; guardar estos ajustes también lo restablece.")}</p>
      <div className="backup-policy-fields">
        <label>{t("Espera entre intentos (segundos)")}<input ref={delayInput} type="number" min={5} max={3600} step={1} value={draft.delaySeconds} aria-invalid={submitted&&!!errors.delaySeconds} aria-describedby={submitted&&errors.delaySeconds?'restart-delay-error':undefined} onChange={event=>edit({...draft,delaySeconds:Number(event.target.value)})}/>{submitted&&errors.delaySeconds&&<span id="restart-delay-error" className="form-error">{errors.delaySeconds}</span>}</label>
        <label>{t("Límite de intentos")}<input ref={attemptsInput} type="number" min={1} max={10} step={1} value={draft.maxAttempts} aria-invalid={submitted&&!!errors.maxAttempts} aria-describedby={submitted&&errors.maxAttempts?'restart-attempts-error':undefined} onChange={event=>edit({...draft,maxAttempts:Number(event.target.value)})}/>{submitted&&errors.maxAttempts&&<span id="restart-attempts-error" className="form-error">{errors.maxAttempts}</span>}</label>
      </div>
      <div className="backup-policy-actions"><button className="primary-button" disabled={!dirty} type="submit">{working?t("Guardando…"):t("Guardar reinicio automático")}</button>{status&&status.attempts>0&&<button type="button" disabled={dirty} onClick={()=>void save(status.policy)}>{t("Restablecer intentos")}</button>}</div>
    </fieldset>
    </form>
    <div className="crash-refresh"><span>{dirty?t("Cambios sin guardar"):saved?t("Ajustes guardados"):status?t("Configuración del servidor"):''}</span><button type="button" disabled={working||loading} onClick={()=>void load()}>{loading?t("Consultando…"):t("Actualizar estado")}</button></div>
    {saved&&readError&&<p className="inline-note">{t("Los ajustes se guardaron. No se pudo consultar el estado actualizado; vuelve a intentarlo con «Actualizar estado».")}</p>}
    {status?.policy.enabled&&<p className="crash-status" role="status"><strong>{status.attempts} / {status.policy.maxAttempts}  {t("intentos usados")}</strong>{status.nextAt!==null&&<span>{t("Próximo intento:")} {formatDate(status.nextAt)}</span>}{status.lastResult&&<span>{translatedMessage(status.lastResult)}</span>}</p>}
    {readError&&<p className="form-error" role="alert">{translatedMessage(readError)}</p>}
    {actionError&&<p className="form-error" role="alert">{translatedMessage(actionError)}</p>}
  </section>
}
