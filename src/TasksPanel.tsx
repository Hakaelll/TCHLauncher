import {translatedMessage,localizedStatic,t,formatDate} from './locale.ts'
import {useCallback,useEffect,useRef,useState,type FormEvent} from 'react'
import {listen} from '@tauri-apps/api/event'
import {retryingSubscription} from './retryingSubscription'
import {api,type ScheduledTask,type TaskAction} from './api'
import {Modal} from './Modal'
import {CrashRestartPanel} from './CrashRestartPanel'
import {scheduledResult,scheduledState} from './scheduledState'
import {PixelIcon} from './PixelIndicators'
import {useServerRead} from './useServerRead'
import {errorText} from './format'

const actions:Record<TaskAction,string>=localizedStatic({backup:'Copia con el servidor detenido',backup_restart:'Copia con parada y reinicio',start:'Iniciar servidor',restart:'Reiniciar servidor',stop:'Detener servidor'})
const taskLabel=(action:ScheduledTask['action'])=>action==='invalid'?t("Tarea no válida"):actions[action]
const localDate=(date:Date)=>new Date(date.getTime()-date.getTimezoneOffset()*60000).toISOString().slice(0,16)
export function TasksPanel({id}:{id:string}) {
  const inventory=useServerRead(id,api.scheduledTasks)
  const tasks=inventory.data??[],loaded=inventory.data!==null
  const [error,setError]=useState<string|null>(null)
  const [eventError,setEventError]=useState<string|null>(null)
  const [adding,setAdding]=useState(false)
  const [working,setWorking]=useState(false)
  const [notice,setNotice]=useState<string|null>(null)
  const [action,setAction]=useState<TaskAction>('backup')
  const [date,setDate]=useState(()=>localDate(new Date(Date.now()+3600000)))
  const [repeat,setRepeat]=useState(false)
  const [minutes,setMinutes]=useState(60)
  const [validation,setValidation]=useState<{field:'date'|'interval';message:string}|null>(null)
  const dateInput=useRef<HTMLInputElement>(null),intervalInput=useRef<HTMLInputElement>(null)
  const scope=useRef({active:false,working:false})
  const load=useCallback(()=>inventory.refresh(id),[id,inventory.refresh])
  const locked=working||inventory.loading||!!inventory.error||!loaded
  useEffect(()=>{
    const current={active:true,working:false};scope.current=current
    setError(null);setEventError(null);setAdding(false);setWorking(false);setNotice(null);setValidation(null)
    let unavailable=false
    const stopEvents=retryingSubscription(()=>listen<string>('scheduler-updated',event=>{if(current.active&&event.payload===id)void load()}),failed=>{
      if(!current.active||scope.current!==current)return
      if(unavailable&&!failed)void load()
      unavailable=failed
      setEventError(failed?t("No se pudieron recibir cambios de tareas. Usa Actualizar para consultar la lista."):null)
    })
    return()=>{current.active=false;stopEvents()}
  },[id,load])
  const mutate=async(operation:()=>Promise<unknown>,message:string,onAcknowledged?:()=>void)=>{
    const current=scope.current;if(!current.active||current.working||locked)return
    current.working=true;setWorking(true);setError(null);setNotice(null)
    try{
      await operation()
      if(!current.active||scope.current!==current)return
      setNotice(message);onAcknowledged?.()
      await load()
    }catch(cause){if(current.active&&scope.current===current)setError(errorText(cause))}
    finally{current.working=false;if(current.active&&scope.current===current)setWorking(false)}
  }
  const submit=async(event:FormEvent)=>{
    event.preventDefault();const nextAt=new Date(date).getTime(),now=Date.now()
    const dateError=!Number.isFinite(nextAt)||nextAt<=now?t("Elige una fecha y hora futuras.")
      :nextAt>now+5*366*24*60*60*1000?t("Elige una fecha dentro de los próximos cinco años.")
      :localDate(new Date(nextAt))!==date?t("Esta fecha y hora no existen en la zona horaria del equipo. Elige otra."):null
    if(dateError){setError(null);setValidation({field:'date',message:dateError});dateInput.current?.focus();return}
    if(repeat&&(!Number.isInteger(minutes)||minutes<1||minutes>525600)){setError(null);setValidation({field:'interval',message:t("Elige un intervalo entero entre 1 y 525600 minutos.")});intervalInput.current?.focus();return}
    setValidation(null)
    await mutate(()=>api.addScheduledTask(id,{action,nextAt,intervalMinutes:repeat?minutes:null}),t("Tarea programada."),()=>setAdding(false))
  }
  return <section className="tasks-panel" aria-labelledby="tasks-title" aria-busy={working||inventory.loading}>
    <div className="section-heading"><div><h3 id="tasks-title">{t("Tareas programadas")}</h3><p>{t("Se ejecutan mientras TCHLauncher está abierto, también en segundo plano.")}</p></div><div className="scheduled-header-actions"><button disabled={working||inventory.loading} onClick={()=>void load()}>{inventory.loading?t("Actualizando…"):t("Actualizar")}</button><button className="primary-button" disabled={locked} onClick={()=>{setDate(localDate(new Date(Date.now()+3600000)));setError(null);setAdding(true)}}>{t("Programar tarea")}</button></div></div>
    <p className="field-help">{t("Si la aplicación estaba cerrada, cada tarea pendiente se ejecuta una vez al abrirla. Los intervalos perdidos no se acumulan.")}</p>
    {eventError&&<p className="field-help" role="status">{translatedMessage(eventError)}</p>}
    {notice&&<p className="runtime-notice" role="status">{translatedMessage(notice)}</p>}{error&&!adding&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    {inventory.error&&<div className="panel-read-state failed"><div><strong>{t("Lista de tareas sin confirmar")}</strong><p role="alert">{translatedMessage(inventory.error)}</p>{loaded&&<p>{t("Se muestran las últimas tareas leídas. Actualiza la lista antes de cambiar tareas.")}</p>}</div><button disabled={working||inventory.loading} onClick={()=>void load()}>{t("Reintentar lectura")}</button></div>}
    {!loaded?<p className="muted">{inventory.error?t("La lista no está disponible. Pulsa Actualizar para volver a consultarla."):t("Cargando tareas…")}</p>:!tasks.length?<div className="scheduled-empty"><span className="scheduled-clock"><PixelIcon kind="clock"/></span><div><strong>{t("Tu próxima tarea")}</strong><p>{t("Programa una copia, un inicio o una parada con «Programar tarea».")}</p></div></div>:<div className="scheduled-list">{tasks.map(task=><article key={task.id} className={`scheduled-row ${task.enabled?'':'paused'} ${scheduledResult(task.lastResult).tone}`}>
      <span className="scheduled-clock"><PixelIcon kind="clock"/></span>
      <div className="scheduled-copy"><div><strong>{taskLabel(task.action)}</strong><span className={`task-state ${task.running?'running':''}`}>{scheduledState(task)}</span></div><p>{task.invalid?t("No se ejecutará. Quita esta tarea y programa otra."):task.enabled?<><span className="scheduled-next-label">{t("Próxima ejecución")}</span><time dateTime={new Date(task.nextAt).toISOString()}>{formatDate(task.nextAt)}</time>{task.intervalMinutes&&` · ${t("cada {0} min",task.intervalMinutes)}`}</>:task.intervalMinutes?t("Cada {0} min",task.intervalMinutes):t("Una vez")}</p>{task.lastResult&&!task.running&&<div className={`scheduled-result ${scheduledResult(task.lastResult).tone}`}><span>{translatedMessage(scheduledResult(task.lastResult).label)}{task.lastAt!==null&&` · ${formatDate(task.lastAt)}`}</span><p>{translatedMessage(task.lastResult)}</p></div>}</div>
      <div className="scheduled-actions">{!task.invalid&&(task.enabled||task.intervalMinutes!==null||task.nextAt>Date.now())&&<button disabled={locked||task.running} onClick={()=>void mutate(()=>api.setScheduledTaskEnabled(id,task.id,!task.enabled),task.enabled?t("Tarea pausada."):t("Tarea activada."))}>{task.enabled?t("Pausar"):t("Activar")}</button>}<button className="text-button" disabled={locked||task.running} aria-label={t("Quitar tarea: {0}",taskLabel(task.action))} onClick={()=>void mutate(()=>api.removeScheduledTask(id,task.id),t("Tarea retirada."))}>{t("Quitar")}</button></div>
    </article>)}</div>}
    <CrashRestartPanel key={id} id={id}/>
    {adding&&<Modal labelledBy="schedule-title" busy={working} onClose={()=>{setAdding(false);setError(null);setValidation(null)}}>
      <h2 id="schedule-title">{t("Programar tarea")}</h2>
      <form className="schedule-form" noValidate onSubmit={submit}>
        <label>{t("Acción")}<select value={action} onChange={event=>{const value=event.target.value;if(value==='backup'||value==='backup_restart'||value==='start'||value==='restart'||value==='stop')setAction(value)}} disabled={working}>{Object.entries(actions).map(([value,label])=><option key={value} value={value}>{translatedMessage(label)}</option>)}</select></label>
        {action==='backup_restart'&&<p className="inline-note">{t("Interrumpirá a los jugadores: detendrá el servidor, guardará la copia e intentará iniciarlo de nuevo. Si ya está detenido, solo hará la copia.")}</p>}
        {action==='backup'&&<p className="field-help">{t("Si el servidor está en ejecución, la copia se omitirá y aparecerá en el resultado de la tarea.")}</p>}
        <label>{t("Primera ejecución")}<input ref={dateInput} type="datetime-local" required value={date} aria-invalid={validation?.field==='date'} aria-describedby={validation?.field==='date'?'schedule-date-error':undefined} onChange={event=>{setDate(event.target.value);if(validation?.field==='date')setValidation(null)}} disabled={working}/>{validation?.field==='date'&&<span id="schedule-date-error" className="form-error" role="alert">{translatedMessage(validation.message)}</span>}</label>
        <label className="schedule-repeat"><input type="checkbox" checked={repeat} onChange={event=>{setRepeat(event.target.checked);if(validation?.field==='interval')setValidation(null)}} disabled={working}/>{t("Repetir")}</label>
        {repeat&&<label>{t("Intervalo en minutos")}<input ref={intervalInput} type="number" required min={1} max={525600} step={1} value={minutes} aria-invalid={validation?.field==='interval'} aria-describedby={validation?.field==='interval'?'schedule-interval-error':undefined} onChange={event=>{setMinutes(Number(event.target.value));if(validation?.field==='interval')setValidation(null)}} disabled={working}/>{validation?.field==='interval'&&<span id="schedule-interval-error" className="form-error" role="alert">{translatedMessage(validation.message)}</span>}</label>}
        {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
        {inventory.error&&<div className="panel-read-state failed"><p role="alert">{translatedMessage(inventory.error)}</p><button type="button" disabled={working||inventory.loading} onClick={()=>void load()}>{t("Reintentar lectura")}</button></div>}
        <div className="modal-actions"><button type="button" data-modal-close disabled={working}>{t("Cancelar")}</button><button className="primary-button" disabled={locked}>{working?t("Programando…"):t("Programar")}</button></div>
      </form>
    </Modal>}
  </section>
}
