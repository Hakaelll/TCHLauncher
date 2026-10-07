import {translatedMessage,t} from './locale.ts'
import {useId,useRef,useState} from 'react'
import {createPortal} from 'react-dom'
import type {Progress,Server} from './api'
import {InstallProgress} from './InstallProgress'
import {isTaskActive,orderedTasks} from './taskState'

export function BackgroundTasks({tasks,servers,error,onOpen,onDismiss,onOpenChange}:{tasks:Record<string,Progress>;servers:Server[];error:string|null;onOpen:(progress:Progress)=>void;onDismiss:(progress:Progress)=>void;onOpenChange:(open:boolean)=>void}){
  const id=useId()
  const panel=useRef<HTMLDivElement>(null)
  const [open,setOpen]=useState(false)
  const rows=orderedTasks(tasks)
  const active=rows.filter(isTaskActive).length
  const label=active?`${active} ${active===1?t("tarea activa"):t("tareas activas")}`:t("Tareas")
  const review=(progress:Progress)=>{panel.current?.hidePopover();onOpen(progress)}
  return <>
    <button className="rail-tasks" popoverTarget={id} aria-expanded={open} aria-controls={id} aria-label={translatedMessage(label)} title={translatedMessage(label)}><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="M5 6h14M5 12h9M5 18h6"/><path d="m16 17 2 2 4-5"/></svg><span>{translatedMessage(label)}</span>{active>0&&<b className="task-count" aria-hidden="true">{active}</b>}</button>
    {createPortal(<div ref={panel} id={id} popover="auto" role="region" aria-labelledby={`${id}-title`} className="background-tasks" onToggle={event=>{const visible=event.newState==='open';setOpen(visible);onOpenChange(visible)}}>
      <header><div><h2 id={`${id}-title`}>{t("Tareas")}</h2><p>{t("Descargas, importaciones y copias de esta sesión.")}</p></div><button popoverTarget={id} popoverTargetAction="hide" aria-label={t("Cerrar tareas")}><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18"/></svg></button></header>
      {error&&<p className="form-error" role="alert">{t("No se pudieron actualizar las tareas:")} {translatedMessage(error)}{t(". Se volverá a comprobar.")}</p>}
      {!rows.length&&<p className="task-empty">{t("No hay tareas activas ni resultados recientes.")}</p>}
      <div className="background-task-list">{rows.map(progress=>{
        const server=servers.find(item=>item.id===progress.serverId)
        return <section className="background-task" key={progress.taskId}><div className="background-task-server"><strong>{server?.name??t("Servidor en preparación")}</strong><button className="text-button" disabled={!server} onClick={()=>review(progress)}>{t("Ver servidor")}</button></div><InstallProgress progress={progress} busy={!server} retryLabel="Ver opciones" onRetry={()=>review(progress)} onDismiss={()=>onDismiss(progress)}/></section>
      })}</div>
    </div>,document.body)}
  </>
}
