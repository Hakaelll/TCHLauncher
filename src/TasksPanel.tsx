import {useCallback,useEffect,useRef,useState,type FormEvent} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type ScheduledTask,type TaskAction} from './api'
import {Modal} from './Modal'
import {CrashRestartPanel} from './CrashRestartPanel'
import {scheduledResult,scheduledState} from './scheduledState'
import {PixelIcon} from './PixelIndicators'

const actions:Record<TaskAction,string>={backup:'Copia con el servidor detenido',backup_restart:'Copia con parada y reinicio',start:'Iniciar servidor',restart:'Reiniciar servidor',stop:'Detener servidor'}
const taskLabel=(action:ScheduledTask['action'])=>action==='invalid'?'Tarea no válida':actions[action]
const localDate=(date:Date)=>new Date(date.getTime()-date.getTimezoneOffset()*60000).toISOString().slice(0,16)
export function TasksPanel({id}:{id:string}) {
  const [tasks,setTasks]=useState<ScheduledTask[]>([])
  const [loaded,setLoaded]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [adding,setAdding]=useState(false)
  const [working,setWorking]=useState(false)
  const [notice,setNotice]=useState<string|null>(null)
  const [action,setAction]=useState<TaskAction>('backup')
  const [date,setDate]=useState(()=>localDate(new Date(Date.now()+3600000)))
  const [repeat,setRepeat]=useState(false)
  const [minutes,setMinutes]=useState(60)
  const scope=useRef({id,active:false,revision:0,working:false})
  const load=useCallback(async()=>{
    const current=scope.current;const revision=++current.revision
    try{
      const value=await api.scheduledTasks(current.id)
      if(!current.active||scope.current!==current||revision!==current.revision)return
      setTasks(value);setLoaded(true);setError(null)
    }catch(cause){if(current.active&&scope.current===current&&revision===current.revision)setError(String(cause))}
  },[])
  useEffect(()=>{
    const current={id,active:true,revision:0,working:false};scope.current=current
    setTasks([]);setLoaded(false);setError(null);setAdding(false);setWorking(false);setNotice(null)
    void load()
    const subscription=listen<string>('scheduler-updated',event=>{if(current.active&&event.payload===id)void load()})
    void subscription.catch(()=>{if(current.active&&scope.current===current)setError('No se pudieron recibir cambios de tareas. Usa Actualizar para consultar la lista.')})
    return()=>{current.active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id,load])
  const mutate=async(operation:()=>Promise<unknown>,message:string,onAcknowledged?:()=>void)=>{
    const current=scope.current;if(!current.active||current.working)return
    current.working=true;setWorking(true);setError(null);setNotice(null)
    try{
      await operation()
      if(!current.active||scope.current!==current)return
      setNotice(message);onAcknowledged?.()
      await load()
    }catch(cause){if(current.active&&scope.current===current)setError(String(cause))}
    finally{current.working=false;if(current.active&&scope.current===current)setWorking(false)}
  }
  const submit=async(event:FormEvent)=>{
    event.preventDefault();const nextAt=new Date(date).getTime()
    if(!Number.isFinite(nextAt)||nextAt<=Date.now()){setError('Elige una fecha y hora futuras.');return}
    if(repeat&&(!Number.isInteger(minutes)||minutes<1||minutes>525600)){setError('Elige un intervalo entero entre 1 y 525600 minutos.');return}
    await mutate(()=>api.addScheduledTask(id,{action,nextAt,intervalMinutes:repeat?minutes:null}),'Tarea programada.',()=>setAdding(false))
  }
  return <section className="tasks-panel" aria-labelledby="tasks-title">
    <div className="section-heading"><div><h3 id="tasks-title">Tareas programadas</h3><p>Se ejecutan mientras TCHLauncher está abierto, también en segundo plano.</p></div><div className="scheduled-header-actions"><button disabled={working} onClick={()=>void load()}>Actualizar</button><button className="primary-button" disabled={working||!loaded} onClick={()=>{setDate(localDate(new Date(Date.now()+3600000)));setError(null);setAdding(true)}}>Programar tarea</button></div></div>
    <p className="field-help">Si la aplicación estaba cerrada, cada tarea pendiente se ejecuta una vez al abrirla. Los intervalos perdidos no se acumulan.</p>
    {notice&&<p className="runtime-notice" role="status">{notice}</p>}{error&&!adding&&<p className="form-error" role="alert">{error}{notice?' La acción se completó; usa Actualizar para consultar la lista.':''}</p>}
    {!loaded?<p className="muted">{error?'La lista no está disponible. Pulsa Actualizar para volver a consultarla.':'Cargando tareas…'}</p>:!tasks.length?<div className="scheduled-empty"><span className="scheduled-clock"><PixelIcon kind="clock"/></span><div><strong>Tu próxima tarea</strong><p>Programa una copia, un inicio o una parada con «Programar tarea».</p></div></div>:<div className="scheduled-list">{tasks.map(task=><article key={task.id} className={`scheduled-row ${task.enabled?'':'paused'} ${scheduledResult(task.lastResult).tone}`}>
      <span className="scheduled-clock"><PixelIcon kind="clock"/></span>
      <div className="scheduled-copy"><div><strong>{taskLabel(task.action)}</strong><span className={`task-state ${task.running?'running':''}`}>{scheduledState(task)}</span></div><p>{task.invalid?'No se ejecutará. Quita esta tarea y programa otra.':task.enabled?<><span className="scheduled-next-label">Próxima ejecución</span><time dateTime={new Date(task.nextAt).toISOString()}>{new Date(task.nextAt).toLocaleString()}</time>{task.intervalMinutes&&` · cada ${task.intervalMinutes} min`}</>:task.intervalMinutes?`Cada ${task.intervalMinutes} min`:'Una vez'}</p>{task.lastResult&&!task.running&&<div className={`scheduled-result ${scheduledResult(task.lastResult).tone}`}><span>{scheduledResult(task.lastResult).label}{task.lastAt!==null&&` · ${new Date(task.lastAt).toLocaleString()}`}</span><p>{task.lastResult}</p></div>}</div>
      <div className="scheduled-actions">{!task.invalid&&(task.enabled||task.intervalMinutes!==null||task.nextAt>Date.now())&&<button disabled={working||task.running} onClick={()=>void mutate(()=>api.setScheduledTaskEnabled(id,task.id,!task.enabled),task.enabled?'Tarea pausada.':'Tarea activada.')}>{task.enabled?'Pausar':'Activar'}</button>}<button className="text-button" disabled={working||task.running} aria-label={`Quitar tarea: ${taskLabel(task.action)}`} onClick={()=>void mutate(()=>api.removeScheduledTask(id,task.id),'Tarea retirada.')}>Quitar</button></div>
    </article>)}</div>}
    <CrashRestartPanel key={id} id={id}/>
    {adding&&<Modal labelledBy="schedule-title" busy={working} onClose={()=>{setAdding(false);setError(null)}}>
      <h2 id="schedule-title">Programar tarea</h2>
      <form className="schedule-form" onSubmit={submit}>
        <label>Acción<select value={action} onChange={event=>setAction(event.target.value as TaskAction)} disabled={working}>{Object.entries(actions).map(([value,label])=><option key={value} value={value}>{label}</option>)}</select></label>
        {action==='backup_restart'&&<p className="inline-note">Interrumpirá a los jugadores: detendrá el servidor, guardará la copia e intentará iniciarlo de nuevo. Si ya está detenido, solo hará la copia.</p>}
        {action==='backup'&&<p className="field-help">Si el servidor está en ejecución, la copia se omitirá y aparecerá en el resultado de la tarea.</p>}
        <label>Primera ejecución<input type="datetime-local" required value={date} onChange={event=>setDate(event.target.value)} disabled={working}/></label>
        <label className="schedule-repeat"><input type="checkbox" checked={repeat} onChange={event=>setRepeat(event.target.checked)} disabled={working}/>Repetir</label>
        {repeat&&<label>Intervalo en minutos<input type="number" required min={1} max={525600} step={1} value={minutes} onChange={event=>setMinutes(Number(event.target.value))} disabled={working}/></label>}
        {error&&<p className="form-error" role="alert">{error}</p>}
        <div className="modal-actions"><button type="button" data-modal-close disabled={working}>Cancelar</button><button className="primary-button" disabled={working}>{working?'Programando…':'Programar'}</button></div>
      </form>
    </Modal>}
  </section>
}
