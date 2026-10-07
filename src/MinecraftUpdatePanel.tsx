import {translatedMessage,localizedStatic,t} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type MinecraftReview,type Progress,type Server,type Version} from './api'
import {Modal} from './Modal'
import {InstallProgress} from './InstallProgress'
import {verifyStartup} from './startupVerification'

const statusLabel=localizedStatic({compatible:'Compatible',incompatible:'Incompatible',unknown:'Sin verificar',update:'Actualizar'})

export function MinecraftUpdatePanel({server,busy,onChanged,currentProgress,onConsole}:{server:Server;busy:boolean;onChanged:()=>Promise<void>;currentProgress?:Progress|null;onConsole:()=>void}) {
  const [open,setOpen]=useState(false)
  const [startAfter,setStartAfter]=useState(true)
  const [checkingStartup,setCheckingStartup]=useState(false)
  const [startupIssue,setStartupIssue]=useState<string|null>(null)
  const observer=useRef<AbortController|null>(null)
  const mounted=useRef(true)
  useEffect(()=>{mounted.current=true;return()=>{mounted.current=false;observer.current?.abort()}},[])

  const [catalogLoaded,setCatalogLoaded]=useState(false)
  const requestBusy=useRef(false)
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
    let active=true
    const subscription=listen<Progress>('install-progress',event=>{
      if(active&&event.payload.serverId===server.id&&event.payload.kind==='minecraft_update')setProgress(event.payload)
    })
    return()=>{active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[server.id])
  if(!server.jarPath)return null
  const check=async()=>{
    if(requestBusy.current||working||checkingStartup||busy||!['stopped','crashed','error'].includes(server.state))return
    requestBusy.current=true;setCatalogLoaded(false);setVersions([]);setTarget('')
    setOpen(true);setWorking(true);setReview(null);setConfirmed(false);setError(null);setNotice(null);setProgress(null);setStartupIssue(null)
    try{
      const catalog=await api.versions(server.platform)
      if(!mounted.current)return
      const current=catalog.findIndex(version=>version.id===server.version)
      const candidates=(current<0?catalog:catalog.slice(0,current)).filter(version=>version.kind==='release'&&version.id!==server.version)
      setVersions(candidates);setTarget(candidates[0]?.id??'');setCatalogLoaded(true)
    }catch(cause){if(mounted.current)setError(String(cause))}finally{requestBusy.current=false;if(mounted.current)setWorking(false)}
  }
  const inspect=async()=>{
    if(!target||!catalogLoaded||requestBusy.current||working||busy)return
    requestBusy.current=true
    setWorking(true);setError(null);setReview(null);setConfirmed(false)
    try{const value=await api.minecraftReview(server.id,target);if(mounted.current)setReview(value)}catch(cause){if(mounted.current)setError(String(cause))}finally{requestBusy.current=false;if(mounted.current)setWorking(false)}
  }
  const update=async()=>{
    if(!review||!confirmed||review.blockers.length||requestBusy.current||working||checkingStartup||busy||!['stopped','crashed','error'].includes(server.state))return
    requestBusy.current=true
    setWorking(true);setError(null);setProgress(null)
    try{
      const warning=await api.updateMinecraft(server.id,review.targetMinecraft,review.fingerprint,confirmed)
      const updatedVersion=review.targetMinecraft
      await onChanged().catch(()=>{});setOpen(false);setReview(null);setWorking(false)
      const suffix=warning?` ${warning}`:''
      if(!startAfter) {
        setNotice(t("Minecraft actualizado a {0}. Arranque pendiente de comprobar.{1}", updatedVersion, suffix))
        return
      }
      setCheckingStartup(true);setStartupIssue(null)
      setNotice(t("Minecraft {0} instalado. Iniciando y esperando a que termine de cargar.{1}", updatedVersion, suffix))
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
        if(result.kind==='ready')setNotice(t("Minecraft {0} ha terminado de cargar. El servidor sigue en marcha; comprueba tus mundos y mods desde el juego.{1}", updatedVersion, suffix))
        else if(result.kind==='halted')setStartupIssue(t("La actualización está instalada, pero el servidor se detuvo antes de estar listo. Revisa la consola. Puedes volver a la versión anterior desde Copias."))
        else setStartupIssue(t("La actualización está instalada, pero no se ha podido confirmar el arranque. Comprueba el estado y la consola antes de volver a iniciar.{0}", result.error?` ${result.error}`:''))
      } catch(cause) {
        if(mounted.current)setStartupIssue(t("Minecraft se actualizó, pero no se pudo iniciar: {0}. La copia previa está disponible en Copias.", String(cause)))
      } finally {
        if(mounted.current){setCheckingStartup(false);await onChanged().catch(()=>{})}
      }
    }catch(cause){if(mounted.current)setError(String(cause));await onChanged().catch(()=>{})}finally{requestBusy.current=false;if(mounted.current)setWorking(false)}
  }
  return <section className="loader-panel minecraft-update-panel">
    <div className="section-heading"><div><h3>Minecraft</h3><p>{t("Versión")} {server.version}  {t("· cambia de versión con una copia previa.")}</p></div><button disabled={working||checkingStartup||busy||!['stopped','crashed','error'].includes(server.state)} onClick={()=>void check()}>{t("Cambiar versión")}</button></div>
    {notice&&!startupIssue&&<p className="inline-note" role="status">{translatedMessage(notice)}</p>}
    {(checkingStartup||startupIssue)&&<div className="minecraft-startup-result" data-failed={!!startupIssue}><p role={startupIssue?'alert':'status'}>{startupIssue??t("Comprobando el arranque real de Minecraft…")}</p><button className="text-button" onClick={onConsole}>{t("Ver consola")}</button></div>}
    {progress&&!open&&<InstallProgress progress={progress} busy={working} onRetry={()=>void check()} onDismiss={()=>setProgress(null)}/>}
    {open&&<Modal className="minecraft-update-dialog" labelledBy="minecraft-update-title" describedBy="minecraft-update-description" busy={working} onClose={()=>{setOpen(false);setError(null)}}>
      <h2 id="minecraft-update-title">{t("Cambiar Minecraft")}</h2><p id="minecraft-update-description" className="field-help">{server.name} · {server.platform}  {t("· versión actual")} {server.version}</p>
      <label className="loader-version">{t("Nueva versión estable")}<select value={target} disabled={working||!versions.length} onChange={event=>{setTarget(event.target.value);setReview(null);setConfirmed(false);setError(null);setProgress(null)}}>{!versions.length&&<option value="">{working?t("Consultando versiones…"):!catalogLoaded?t("Catálogo no disponible"):t("No hay versiones posteriores en el catálogo")}</option>}{versions.map(version=><option key={version.id}>{version.id}</option>)}</select></label>
      {!catalogLoaded&&!working&&<button disabled={busy} onClick={()=>void check()}>{t("Reintentar catálogo")}</button>}
      {!review&&<p className="field-help">{t("La revisión comprobará Java, el motor y la compatibilidad declarada de tus mods o plugins.")}</p>}
      {working&&!progress&&<p role="status">{t("Consultando los catálogos oficiales…")}</p>}
      {review&&<div className="minecraft-review">
        <dl className="minecraft-review-summary"><div><dt>Minecraft</dt><dd>{review.currentMinecraft} → {review.targetMinecraft}</dd></div><div><dt>Java {review.javaMajor}</dt><dd>{review.javaAction==='retain_configured'?t("Conservar runtime compatible"):t("Preparar Temurin compatible")}</dd></div>{review.targetLoader&&<div><dt>{server.platform==='paper'?'Paper build':'Loader'}</dt><dd>{review.targetLoader}</dd></div>}</dl>
        {review.content.length>0&&<section aria-label={t("Compatibilidad del contenido")}><h3>{server.platform==='paper'?t("Plugins instalados"):t("Mods instalados")}</h3><ul className="minecraft-content-review">{review.content.map(file=><li key={file.filename}><div><strong>{file.title}</strong><span data-status={file.status}>{statusLabel[file.status]}</span></div><p>{translatedMessage(file.reason)}</p></li>)}</ul></section>}
        {review.changes.length>0&&<section aria-label={t("Cambios de contenido")}><h3>{t("Plan de cambios")}</h3><ul className="minecraft-content-review">{review.changes.map(change=><li key={change.filename}><div><strong>{change.title}</strong><span data-status={change.action==='present'?'compatible':'update'}>{change.action==='present'?t("Conservar"):change.action==='install'?t("Añadir dependencia"):t("Actualizar")}</span></div><p>{change.previousVersion?`${change.previousVersion} → ${change.version}`:change.version}</p></li>)}</ul></section>}
        {review.migration?.entries.some(entry=>entry.version.changelog)&&<details className="minecraft-migration-changelog"><summary>{t("Notas de las versiones propuestas")}</summary>{review.migration.entries.filter(entry=>entry.version.changelog).map(entry=><section key={entry.version.id}><h3>{entry.project.title} · {entry.version.version_number}</h3><p>{entry.version.changelog}</p></section>)}</details>}
        {review.blockers.length>0&&<div className="form-error" role="alert"><strong>{t("No se puede conservar este contenido al cambiar de versión.")}</strong><ul>{review.blockers.map(blocker=><li key={blocker}>{blocker}</li>)}</ul><p>{t("Cierra esta revisión para gestionar los archivos incompatibles y vuelve a comprobar la actualización.")}</p></div>}
        {review.warnings.length>0&&<ul className="minecraft-update-warnings">{review.warnings.map(warning=><li key={warning}>{translatedMessage(warning)}</li>)}</ul>}
        {!review.blockers.length&&<label className="minecraft-update-consent"><input type="checkbox" checked={confirmed} disabled={working} onChange={event=>setConfirmed(event.target.checked)}/><span>{t("He revisado los avisos y el plan de cambios. Para volver a esta versión restauraré la copia previa, incluidos los mundos.")}</span></label>}
        {!review.blockers.length&&<label className="minecraft-update-consent"><input type="checkbox" checked={startAfter} disabled={working} onChange={event=>setStartAfter(event.target.checked)}/><span>{t("Iniciar y comprobar al terminar. Minecraft puede convertir los mundos a la nueva versión; el servidor quedará en marcha.")}</span></label>}
      </div>}
      {progress&&<InstallProgress progress={progress} busy={working} onRetry={()=>void inspect()} onDismiss={()=>setProgress(null)}/>}
      {error&&<p ref={errorRef} className="form-error" role="alert" tabIndex={-1}>{translatedMessage(error)}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>{t("Cancelar")}</button>{!review?<button className="primary-button" disabled={working||busy||!catalogLoaded||!target} onClick={()=>void inspect()}>{t("Revisar cambio")}</button>:<button className="primary-button" disabled={working||busy||!['stopped','crashed','error'].includes(server.state)||!confirmed||!!review.blockers.length} onClick={()=>void update()}>{working?t("Actualizando…"):t("Crear copia y actualizar")}</button>}</div>
    </Modal>}
  </section>
}
