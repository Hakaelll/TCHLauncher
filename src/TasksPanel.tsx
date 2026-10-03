import {useEffect,useState,type FormEvent} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type ScheduledTask,type TaskAction} from './api'
import {Modal} from './Modal'
import {CrashRestartPanel} from './CrashRestartPanel'

const actions:Record<TaskAction,string>={backup:'Copia con el servidor detenido',backup_restart:'Copia con parada y reinicio',start:'Iniciar servidor',restart:'Reiniciar servidor',stop:'Detener servidor'}
const taskLabel=(action:ScheduledTask['action'])=>action==='invalid'?'Tarea no válida':actions[action]
const localDate=(date:Date)=>new Date(date.getTime()-date.getTimezoneOffset()*60000).toISOString().slice(0,16)
export function TasksPanel({id}:{id:string}) {
  const [tasks,setTasks]=useState<ScheduledTask[]>([])
  const [loaded,setLoaded]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [adding,setAdding]=useState(false)
  const [working,setWorking]=useState(false)
  const [action,setAction]=useState<TaskAction>('backup')
  const [date,setDate]=useState(()=>localDate(new Date(Date.now()+3600000)))
  const [repeat,setRepeat]=useState(false)
  const [minutes,setMinutes]=useState(60)
  useEffect(()=>{
    let active=true
    const load=()=>{void api.scheduledTasks(id).then(value=>{if(active){setTasks(value);setLoaded(true)}}).catch(cause=>{if(active)setError(String(cause))})}
    load()
    const subscription=listen<string>('scheduler-updated',event=>{if(event.payload===id)load()})
    return()=>{active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[id])
  const mutate=async(operation:()=>Promise<unknown>)=>{setWorking(true);setError(null);try{await operation();setTasks(await api.scheduledTasks(id));return true}catch(cause){setError(String(cause));return false}finally{setWorking(false)}}
  const submit=async(event:FormEvent)=>{event.preventDefault();const nextAt=new Date(date).getTime();if(!Number.isFinite(nextAt)||nextAt<=Date.now()){setError('Elige una fecha y hora futuras.');return}if(await mutate(()=>api.addScheduledTask(id,{action,nextAt,intervalMinutes:repeat?minutes:null})))setAdding(false)}
  return <section className="tasks-panel" aria-labelledby="tasks-title">
    <div className="section-heading"><div><h3 id="tasks-title">Tareas programadas</h3><p>Se ejecutan mientras TCHLauncher está abierto, también en segundo plano.</p></div><button disabled={working||!loaded} onClick={()=>{setDate(localDate(new Date(Date.now()+3600000)));setError(null);setAdding(true)}}>Programar tarea</button></div>
    <p className="field-help">Si la aplicación estaba cerrada, cada tarea pendiente se ejecuta una vez al abrirla. Los intervalos perdidos no se acumulan.</p>
    {error&&!adding&&<p className="form-error" role="alert">{error}</p>}
    {!loaded?<p className="muted">Cargando tareas…</p>:!tasks.length?<p className="empty-inline">No hay tareas programadas.</p>:<div className="scheduled-list">{tasks.map(task=><article key={task.id} className={`scheduled-row ${task.enabled?'':'paused'}`}>
      <div className="scheduled-copy"><div><strong>{taskLabel(task.action)}</strong><span className={`task-state ${task.running?'running':''}`}>{task.invalid?'Datos inválidos':task.running?'En ejecución':task.enabled?'Programada':task.lastAt&&!task.intervalMinutes?'Finalizada':'Pausada'}</span></div><p>{task.invalid?'No se ejecutará. Quita esta tarea y programa otra.':task.enabled?<><time dateTime={new Date(task.nextAt).toISOString()}>{new Date(task.nextAt).toLocaleString()}</time>{task.intervalMinutes&&` · cada ${task.intervalMinutes} min`}</>:task.intervalMinutes?`Cada ${task.intervalMinutes} min`:'Una vez'}</p>{task.lastResult&&<p className={task.lastResult.startsWith('Error:')?'form-error':'task-result'}>{task.lastAt&&`${new Date(task.lastAt).toLocaleString()} · `}{task.lastResult}</p>}</div>
      <div className="scheduled-actions">{!task.invalid&&(task.enabled||task.intervalMinutes!==null||task.nextAt>Date.now())&&<button disabled={working||task.running} onClick={()=>void mutate(()=>api.setScheduledTaskEnabled(id,task.id,!task.enabled))}>{task.enabled?'Pausar':'Activar'}</button>}<button className="text-button" disabled={working||task.running} aria-label={`Quitar tarea: ${taskLabel(task.action)}`} onClick={()=>void mutate(()=>api.removeScheduledTask(id,task.id))}>Quitar</button></div>
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
