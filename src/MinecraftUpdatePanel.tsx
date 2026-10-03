import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type MinecraftReview,type Progress,type Server,type Version} from './api'
import {Modal} from './Modal'
import {InstallProgress} from './InstallProgress'
import {verifyStartup} from './startupVerification'

const statusLabel={compatible:'Compatible',incompatible:'Incompatible',unknown:'Sin verificar',update:'Actualizar'}

export function MinecraftUpdatePanel({server,busy,onChanged,currentProgress,onConsole}:{server:Server;busy:boolean;onChanged:()=>Promise<void>;currentProgress?:Progress|null;onConsole:()=>void}) {
  const [open,setOpen]=useState(false)
  const [startAfter,setStartAfter]=useState(true)
  const [checkingStartup,setCheckingStartup]=useState(false)
  const [startupIssue,setStartupIssue]=useState<string|null>(null)
  const observer=useRef<AbortController|null>(null)
  const mounted=useRef(true)
  useEffect(()=>{mounted.current=true;return()=>{mounted.current=false;observer.current?.abort()}},[])

  const [versions,setVersions]=useState<Version[]>([])
  const [target,setTarget]=useState('')
  const [review,setReview]=useState<MinecraftReview|null>(null)
  const [working,setWorking]=useState(false)
  const [confirmed,setConfirmed]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [progress,setProgress]=useState<Progress|null>(null)
  const errorRef=useRef<HTMLParagraphElement>(null)
  useEffect(()=>{if(currentProgress?.kind==='minecraft_update')setProgress(currentProgress)},[currentProgress])
  useEffect(()=>{if(error&&open)errorRef.current?.focus()},[error,open])
  useEffect(()=>{
    const subscription=listen<Progress>('install-progress',event=>{
      if(event.payload.serverId===server.id&&event.payload.kind==='minecraft_update')setProgress(event.payload)
    })
    return()=>{void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[server.id])
  if(!server.jarPath)return null
  const check=async()=>{
    setOpen(true);setWorking(true);setReview(null);setConfirmed(false);setError(null);setNotice(null);setProgress(null);setStartupIssue(null)
    try{
      const catalog=await api.versions(server.platform)
      const current=catalog.findIndex(version=>version.id===server.version)
      const candidates=(current<0?catalog:catalog.slice(0,current)).filter(version=>version.kind==='release'&&version.id!==server.version)
      setVersions(candidates);setTarget(candidates[0]?.id??'')
    }catch(cause){setError(String(cause))}finally{setWorking(false)}
  }
  const inspect=async()=>{
    if(!target)return
    setWorking(true);setError(null);setReview(null);setConfirmed(false)
    try{setReview(await api.minecraftReview(server.id,target))}catch(cause){setError(String(cause))}finally{setWorking(false)}
  }
  const update=async()=>{
    if(!review||!confirmed||review.blockers.length)return
    setWorking(true);setError(null);setProgress(null)
    try{
      const warning=await api.updateMinecraft(server.id,review.targetMinecraft,review.fingerprint,confirmed)
      const updatedVersion=review.targetMinecraft
      await onChanged().catch(()=>{});setOpen(false);setReview(null);setWorking(false)
      const suffix=warning?` ${warning}`:''
      if(!startAfter) {
        setNotice(`Minecraft actualizado a ${updatedVersion}. Arranque pendiente de comprobar.${suffix}`)
        return
      }
      setCheckingStartup(true);setStartupIssue(null)
      setNotice(`Minecraft ${updatedVersion} instalado. Iniciando y esperando a que termine de cargar.${suffix}`)
      try {
        observer.current=new AbortController()
        await api.start(server.id)
        if(!mounted.current)return
        const result=await verifyStartup({
          readState:async()=>{const servers=await api.listServers();return servers.find(item=>item.id===server.id)?.state??null},
          wait:()=>new Promise(resolve=>window.setTimeout(resolve,1500)),
          now:()=>performance.now(),timeoutMs:180000,signal:observer.current.signal,
        })
        if(!mounted.current)return
        if(result.kind==='ready')setNotice(`Minecraft ${updatedVersion} ha terminado de cargar. El servidor sigue en marcha; comprueba tus mundos y mods desde el juego.${suffix}`)
        else if(result.kind==='halted')setStartupIssue('La actualización está instalada, pero el servidor se detuvo antes de estar listo. Revisa la consola. Puedes volver a la versión anterior desde Copias.')
        else setStartupIssue(`La actualización está instalada, pero no se ha podido confirmar el arranque. Comprueba el estado y la consola antes de volver a iniciar.${result.error?` ${result.error}`:''}`)
      } catch(cause) {
        if(mounted.current)setStartupIssue(`Minecraft se actualizó, pero no se pudo iniciar: ${String(cause)}. La copia previa está disponible en Copias.`)
      } finally {
        if(mounted.current){setCheckingStartup(false);await onChanged().catch(()=>{})}
      }
    }catch(cause){setError(String(cause));await onChanged().catch(()=>{})}finally{setWorking(false)}
  }
  return <section className="loader-panel minecraft-update-panel">
    <div className="section-heading"><div><h3>Minecraft</h3><p>Versión {server.version} · cambia de versión con una copia previa.</p></div><button disabled={working||checkingStartup||busy||!['stopped','crashed','error'].includes(server.state)} onClick={()=>void check()}>Cambiar versión</button></div>
    {notice&&!startupIssue&&<p className="inline-note" role="status">{notice}</p>}
    {(checkingStartup||startupIssue)&&<div className="minecraft-startup-result" data-failed={!!startupIssue}><p role={startupIssue?'alert':'status'}>{startupIssue??'Comprobando el arranque real de Minecraft…'}</p><button className="text-button" onClick={onConsole}>Ver consola</button></div>}
    {progress&&!open&&<InstallProgress progress={progress} busy={working} onRetry={()=>void check()} onDismiss={()=>setProgress(null)}/>}
    {open&&<Modal className="minecraft-update-dialog" labelledBy="minecraft-update-title" describedBy="minecraft-update-description" busy={working} onClose={()=>{setOpen(false);setError(null)}}>
      <h2 id="minecraft-update-title">Cambiar Minecraft</h2><p id="minecraft-update-description" className="field-help">{server.name} · {server.platform} · versión actual {server.version}</p>
      <label className="loader-version">Nueva versión estable<select value={target} disabled={working||!versions.length} onChange={event=>{setTarget(event.target.value);setReview(null);setConfirmed(false);setError(null);setProgress(null)}}>{!versions.length&&<option value="">{working?'Consultando versiones…':'No hay versiones posteriores en el catálogo'}</option>}{versions.map(version=><option key={version.id}>{version.id}</option>)}</select></label>
      {!review&&<p className="field-help">La revisión comprobará Java, el motor y la compatibilidad declarada de tus mods o plugins.</p>}
      {working&&!progress&&<p role="status">Consultando los catálogos oficiales…</p>}
      {review&&<div className="minecraft-review">
        <dl className="minecraft-review-summary"><div><dt>Minecraft</dt><dd>{review.currentMinecraft} → {review.targetMinecraft}</dd></div><div><dt>Java {review.javaMajor}</dt><dd>{review.javaAction==='retain_configured'?'Conservar runtime compatible':'Preparar Temurin compatible'}</dd></div>{review.targetLoader&&<div><dt>{server.platform==='paper'?'Paper build':'Loader'}</dt><dd>{review.targetLoader}</dd></div>}</dl>
        {review.content.length>0&&<section aria-label="Compatibilidad del contenido"><h3>{server.platform==='paper'?'Plugins instalados':'Mods instalados'}</h3><ul className="minecraft-content-review">{review.content.map(file=><li key={file.filename}><div><strong>{file.title}</strong><span data-status={file.status}>{statusLabel[file.status]}</span></div><p>{file.reason}</p></li>)}</ul></section>}
        {review.changes.length>0&&<section aria-label="Cambios de contenido"><h3>Plan de cambios</h3><ul className="minecraft-content-review">{review.changes.map(change=><li key={change.filename}><div><strong>{change.title}</strong><span data-status={change.action==='present'?'compatible':'update'}>{change.action==='present'?'Conservar':change.action==='install'?'Añadir dependencia':'Actualizar'}</span></div><p>{change.previousVersion?`${change.previousVersion} → ${change.version}`:change.version}</p></li>)}</ul></section>}
        {review.migration?.entries.some(entry=>entry.version.changelog)&&<details className="minecraft-migration-changelog"><summary>Notas de las versiones propuestas</summary>{review.migration.entries.filter(entry=>entry.version.changelog).map(entry=><section key={entry.version.id}><h3>{entry.project.title} · {entry.version.version_number}</h3><p>{entry.version.changelog}</p></section>)}</details>}
        {review.blockers.length>0&&<div className="form-error" role="alert"><strong>No se puede conservar este contenido al cambiar de versión.</strong><ul>{review.blockers.map(blocker=><li key={blocker}>{blocker}</li>)}</ul><p>Cierra esta revisión para gestionar los archivos incompatibles y vuelve a comprobar la actualización.</p></div>}
        {review.warnings.length>0&&<ul className="minecraft-update-warnings">{review.warnings.map(warning=><li key={warning}>{warning}</li>)}</ul>}
        {!review.blockers.length&&<label className="minecraft-update-consent"><input type="checkbox" checked={confirmed} disabled={working} onChange={event=>setConfirmed(event.target.checked)}/><span>He revisado los avisos y el plan de cambios. Para volver a esta versión restauraré la copia previa, incluidos los mundos.</span></label>}
        {!review.blockers.length&&<label className="minecraft-update-consent"><input type="checkbox" checked={startAfter} disabled={working} onChange={event=>setStartAfter(event.target.checked)}/><span>Iniciar y comprobar al terminar. Minecraft puede convertir los mundos a la nueva versión; el servidor quedará en marcha.</span></label>}
      </div>}
      {progress&&<InstallProgress progress={progress} busy={working} onRetry={()=>void inspect()} onDismiss={()=>setProgress(null)}/>}
      {error&&<p ref={errorRef} className="form-error" role="alert" tabIndex={-1}>{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>Cancelar</button>{!review?<button className="primary-button" disabled={working||!target} onClick={()=>void inspect()}>Revisar cambio</button>:<button className="primary-button" disabled={working||!confirmed||!!review.blockers.length} onClick={()=>void update()}>{working?'Actualizando…':'Crear copia y actualizar'}</button>}</div>
    </Modal>}
  </section>
}
