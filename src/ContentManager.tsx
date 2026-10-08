import {errorText} from './format'
import {translatedMessage,localizedStatic,t,formatDate} from './locale.ts'
import { useEffect, useRef, useState } from 'react'
import { api, type ContentReview, type ContentVersion, type InstalledContent, type Progress, type LogLine, type Server } from './api'
import { isTaskActive } from './taskState'
import { Modal } from './Modal'
import { InstallProgress } from './InstallProgress'
import {ContentDiscovery} from './ContentDiscovery'
import {InstalledContentView} from './InstalledContentView'
import {DisabledContentView} from './DisabledContentView'
import {useServerRead} from './useServerRead'
const actionLabels=localizedStatic({install:'Añadir',update:'Actualizar',present:'Ya instalado'})
const environmentLabels:Record<string,string>=localizedStatic({client_and_server:'Cliente y servidor',client_only_server_optional:'Cliente necesario · servidor opcional',server_only:'Solo servidor',server_only_client_optional:'Servidor · cliente opcional',dedicated_server_only:'Servidor dedicado',client_or_server:'Cliente o servidor',client_or_server_prefers_both:'Cliente y servidor recomendados'})
export function ContentManager({server,progress,logs,onConsole,onInstalled,onDismissProgress,onImportPack}:{server:Server;progress:Progress|null;logs:LogLine[];onConsole:(query:string)=>void;onInstalled:()=>Promise<void>;onDismissProgress:()=>void;onImportPack:()=>void}) {
  const plugins=server.platform==='paper'
  const [mode,setMode]=useState<'installed'|'disabled'|'discover'>('installed')
  const inventory=useServerRead(server.id,api.auditContent)
  const audit=inventory.data,files=audit?.files??[],disabled=audit?.disabled??[],loading=inventory.loading
  const [hasBrowsed,setHasBrowsed]=useState(false)
  const [review,setReview]=useState<ContentReview|null>(null)
  const [versions,setVersions]=useState<ContentVersion[]>([])
  const [preparing,setPreparing]=useState<string|null>(null)
  const [installing,setInstalling]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [reviewError,setReviewError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [bulk,setBulk]=useState(false)
  const [syncError,setSyncError]=useState<string|null>(null)
  const owner=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};owner.current=current;return()=>{current.active=false;reviewSequence.current++}},[server.id])
  const [removeTarget,setRemoveTarget]=useState<InstalledContent|null>(null)
  const reviewSequence=useRef(0)
  const stopped=['stopped','error','crashed'].includes(server.state)
  const locked=installing||preparing!==null||!!progress&&isTaskActive(progress)
  const owns=(current:typeof owner.current)=>current.active&&owner.current===current
  const observedState=useRef(server.state)
  useEffect(()=>{
    if(observedState.current===server.state)return
    observedState.current=server.state;reviewSequence.current++;setPreparing(null)
    if(!owner.current.working){setReview(null);setRemoveTarget(null);setReviewError(null)}
    void inventory.refresh(server.id)
  },[server.id,server.state,inventory.refresh])
  const openProject=async(projectId:string)=>{
    const current=owner.current
    try{await api.openContentProject(projectId)}catch(e){if(owns(current))setError(errorText(e))}
  }
  const browse=()=>{setHasBrowsed(true);setMode('discover')}
  const refresh=async()=>{await inventory.refresh(server.id)}
  const sync=async(current=owner.current)=>{
    await inventory.refresh(server.id)
    if(!owns(current))return
    try{await onInstalled();if(owns(current))setSyncError(null)}
    catch(e){if(owns(current))setSyncError(t("La operación terminó, pero no se pudieron actualizar los datos del servidor: {0}", errorText(e)))}
  }
  const retrySync=async()=>{
    const current=owner.current;if(current.working||locked||loading)return
    current.working=true;setInstalling(true)
    try{await sync(current)}finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const prepare=async(projectId:string,versionId:string|null=null)=>{
    if(locked||owner.current.working||!stopped)return
    setBulk(false);const sequence=++reviewSequence.current;setPreparing(projectId);setError(null);setReviewError(null)
    try {
      const available=await api.contentVersions(server.id,projectId)
      if(sequence!==reviewSequence.current)return
      setVersions(available)
      const selectedId=versionId??available.find(version=>version.version_type==='release')?.id??available[0]?.id
      if(!selectedId)throw new Error(t("No hay una publicación compatible con este servidor."))
      const value=await api.planContent(server.id,projectId,selectedId)
      if(sequence!==reviewSequence.current)return
      setReview(value)
    }catch(e){if(sequence===reviewSequence.current){if(review)setReviewError(errorText(e));else setError(errorText(e))}}
    finally{if(sequence===reviewSequence.current)setPreparing(null)}
  }
  const prepareUpdates=async()=>{
    if(locked||owner.current.working||!stopped)return
    const sequence=++reviewSequence.current;setPreparing('updates');setError(null);setReviewError(null);setBulk(true);setVersions([])
    try {const value=await api.planContentUpdates(server.id);if(sequence===reviewSequence.current)setReview(value)}
    catch(e){if(sequence===reviewSequence.current){if(review)setReviewError(errorText(e));else setError(errorText(e))}}
    finally{if(sequence===reviewSequence.current)setPreparing(null)}
  }
  const remove=async()=>{
    const current=owner.current
    if(!removeTarget||current.working||locked||!stopped)return
    current.working=true;setInstalling(true);setReviewError(null);setSyncError(null)
    try {
      await api.removeContent(server.id,removeTarget.filename,removeTarget.sha512)
      if(!owns(current))return
      setRemoveTarget(null);setNotice(t("Archivo retirado. Se ha guardado una copia del servidor."))
      await sync(current)
    }catch(e){if(owns(current)&&errorText(e)!=='Instalación cancelada')setReviewError(errorText(e))}
    finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const toggle=async(file:InstalledContent,enabled:boolean)=>{
    const current=owner.current
    if(current.working||locked||!stopped||loading||inventory.error)return
    if(progress&&!isTaskActive(progress))onDismissProgress()
    current.working=true;setInstalling(true);setError(null);setNotice(null);setSyncError(null)
    try{
      await api.setContentEnabled(server.id,file.filename,file.sha512,enabled)
      if(!owns(current))return
      setNotice(enabled?t("Archivo reactivado. Se cargar\u00e1 en el pr\u00f3ximo inicio."):t("Archivo desactivado. Su configuraci\u00f3n se conserva."))
      await sync(current)
    }catch(e){if(owns(current))setError(errorText(e))}
    finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const install=async()=>{
    const current=owner.current
    if(!review||current.working||locked||!stopped)return
    current.working=true;setInstalling(true);setReviewError(null);setNotice(null);setSyncError(null)
    try {
      await api.installContent(server.id,review.token)
      if(!owns(current))return
      setReview(null);setNotice(plugins?t("Plugins instalados. Se cargarán en el próximo inicio."):t("Mods instalados. Reinicia para comprobar su carga."))
      await sync(current)
    }catch(e){
      if(owns(current)){setReviewError(errorText(e));await inventory.refresh(server.id)}
    }
    finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const main=review?.plan.entries.find(entry=>!entry.required)
  const changes=review?.changes.filter(change=>change.action!=='present')??[]
  return <section className="content-manager" aria-busy={loading||locked}>
    <div className="section-heading"><div><h2>{t("Contenido")}</h2><p>{server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'} · Minecraft {server.version}</p></div><div className="content-toolbar"><button disabled={!stopped||locked||loading||!!inventory.error||!files.some(file=>file.projectId)} onClick={()=>void prepareUpdates()}>{preparing==='updates'?t("Comprobando…"):t("Revisar actualizaciones")}</button><button disabled={locked} onClick={browse}>{t("Explorar Modrinth")} <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="M5 12h14m-6-6 6 6-6 6"/></svg></button></div></div>
    <div className="content-mode" role="group" aria-label={t("Vista de contenido")}><button aria-pressed={mode==='installed'} disabled={locked} onClick={()=>setMode('installed')}>{t("Instalados")} <span>{audit?files.length:'—'}</span></button><button aria-pressed={mode==='disabled'} disabled={locked} onClick={()=>setMode('disabled')}>{t("Desactivados")} <span>{audit?disabled.length:'—'}</span></button><button aria-pressed={mode==='discover'} disabled={locked} onClick={browse}>{t("Explorar")}</button></div>
    {!stopped&&<p className="inline-note">{t("Puedes explorar mientras juega tu servidor. Detén el servidor para instalar o actualizar")} {plugins?'plugins':'mods'}.</p>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    {notice&&<p className="content-success" role="status">{translatedMessage(notice)}</p>}
    {inventory.error&&<div className="panel-read-state failed"><div><strong>{t("Inventario sin confirmar")}</strong><p>{translatedMessage(inventory.error)}{audit?' Se conservan los últimos archivos leídos.':''}</p></div><button disabled={loading||locked} onClick={()=>void refresh()}>{t("Reintentar lectura")}</button></div>}
    {syncError&&<div className="loader-sync-error"><p className="form-error" role="alert">{translatedMessage(syncError)}</p><button disabled={loading||locked} onClick={()=>void retrySync()}>{t("Actualizar datos")}</button></div>}
    {!review&&!removeTarget&&(progress?.kind==='content_install'||progress?.kind==='content_remove'||progress?.kind==='content_toggle')&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} retryLabel={progress.kind==='content_toggle'?t("Revisar archivos"):t("Reintentar")} onRetry={()=>{onDismissProgress();if(progress?.kind==='content_toggle'){void refresh();return;}if(progress?.kind==='content_remove')setMode('installed');else browse()}} onDismiss={onDismissProgress}/>}
    <div hidden={mode!=='installed'}>
      <div className="content-inventory-heading"><div><h3>{t("Inventario del servidor")}</h3><p>{t("Iconos, versiones y avisos de cada archivo.")}</p></div><button disabled={loading||locked} onClick={()=>void refresh()}>{loading?t("Revisando…"):t("Revisar archivos")}</button></div>
      {audit&&files.length>0&&<div className="content-audit-summary"><details><summary>{t("Revisión de carga")}{server.platform==='fabric'?` · ${audit.checks.filter(check=>check.listedInLog===true).length} presentes en el último log`: ''}</summary><p>{translatedMessage(audit.logNote)}{audit.logModifiedAt?` Log actualizado: ${formatDate(audit.logModifiedAt)}.`:''}</p><p>{t("Revisamos el descriptor, el cargador y los identificadores duplicados. En Fabric se revisan rangos reconocidos entre mods principales; los requisitos del runtime y la selección de archivos incluidos siguen a cargo del loader. Un mod de estructuras puede afectar solo a terreno nuevo; algunos mods también deben instalarse en los clientes.")}</p></details></div>}
      {!audit?<p role="status">{loading?t("Revisando archivos y log…"):t("Reintenta la lectura para ver los archivos del servidor.")}</p>:<InstalledContentView onDisable={file=>{void toggle(file,false)}} active={mode==='installed'} onProject={id=>{void openProject(id)}} server={server} files={files} audit={audit} logs={logs} locked={locked||loading||!!inventory.error} stopped={stopped} onBrowse={browse} onConsole={onConsole} onPrepare={id=>{setVersions([]);void prepare(id)}} onRemove={file=>{if(!stopped||locked||loading||inventory.error)return;if(progress&&!isTaskActive(progress))onDismissProgress();setRemoveTarget(file);setReviewError(null)}}/>}
    </div>
    <div hidden={mode!=='disabled'}>{audit?<DisabledContentView files={disabled} locked={locked||loading||!!inventory.error} stopped={stopped} onEnable={file=>{void toggle(file,true)}} onInstalled={()=>setMode('installed')}/>:<p role="status">{loading?t("Revisando archivos y log…"):t("Reintenta la lectura para ver los archivos del servidor.")}</p>}</div>
    <div hidden={mode!=='discover'}><ContentDiscovery onImportPack={onImportPack} server={server} active={hasBrowsed&&mode==='discover'} files={files} locked={locked} stopped={stopped} preparing={preparing} onPrepare={(id,version)=>{setVersions([]);void prepare(id,version)}}/></div>
    {preparing&&!review&&<p className="inline-note" role="status">{t("Comprobando versión y dependencias…")}</p>}
    {removeTarget&&<Modal labelledBy="content-remove-title" busy={locked} onClose={()=>setRemoveTarget(null)}><h2 id="content-remove-title">{t("Retirar")} {removeTarget.title}</h2><p>{t("Se guardará una copia del servidor y se retirará este archivo. Sus carpetas de configuración se conservarán.")}</p><p className="inline-note">{removeTarget.filename}</p>{progress?.kind==='content_remove'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} allowRetry={false} onRetry={()=>{}} onDismiss={onDismissProgress}/>}{!removeTarget.projectId&&<p className="inline-note">{t("Este archivo se añadió manualmente. No se conocen sus dependencias.")}</p>}{reviewError&&<p className="form-error" role="alert">{translatedMessage(reviewError)}</p>}<div className="dialog-actions"><button data-modal-close disabled={locked}>{t("Cancelar")}</button><button className="danger-button" disabled={locked||!stopped||loading||!!inventory.error} onClick={()=>void remove()}>{installing?t("Retirando…"):t("Sí, retirar archivo")}</button></div></Modal>}
    {review&&<Modal labelledBy="content-review-title" describedBy="content-review-description" busy={locked} className="content-review-dialog" onClose={()=>{setReview(null);setReviewError(null);reviewSequence.current++}}><h2 id="content-review-title">{bulk?t("Revisar actualizaciones"):main?.project.title??t("Revisar instalación")}</h2><p id="content-review-description">{review.plan.loader==='paper'?t("Plugin para Paper"):t("Mod para ")+review.plan.loader} · Minecraft {review.plan.minecraftVersion}</p>
      {installing&&progress?.kind==='content_install'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} onRetry={()=>{}} onDismiss={()=>{}}/>}
      {!bulk&&main&&main.version.version_type!=='release'&&<p className="inline-note">{t("Publicación")} {main.version.version_type}{t(": puede ser experimental. Revisa los cambios antes de instalarla.")}</p>}
      {!bulk&&main&&<p className="content-environment">{environmentLabels[main.version.environment??'']??t("Compatible con servidor")}</p>}
      {!bulk&&main&&versions.length>0&&<label className="content-version-label">{t("Versión compatible")}<select value={main.version.id} disabled={locked} onChange={event=>void prepare(main.project.id,event.target.value)}>{versions.map(version=><option key={version.id} value={version.id}>{version.version_number}{version.version_type==='release'?'':` · ${version.version_type}`}</option>)}</select></label>}
      <div className="content-review-changes">{review.changes.map(change=><div key={change.filename}><div><strong>{change.title}</strong><span>{change.previousVersion&&change.action==='update'?`${change.previousVersion} → `:''}{change.version}</span></div><span className={`content-change-action ${change.action}`}>{actionLabels[change.action]}</span></div>)}</div>
      {review.plan.entries.filter(entry=>entry.required).length>0&&<p className="inline-note">{t("Las dependencias obligatorias están incluidas en esta revisión.")}</p>}
      {review.plan.optional.length>0&&<details className="content-dependencies"><summary>{t("Dependencias opcionales (")}{review.plan.optional.length})</summary>{review.plan.optional.map((dependency,index)=><p key={index}>{dependency.file_name??dependency.project_id??dependency.version_id??t("Dependencia externa")}</p>)}</details>}
      {review.plan.incompatible.length>0&&<details className="content-dependencies"><summary>{t("Incompatibilidades declaradas (")}{review.plan.incompatible.length})</summary>{review.plan.incompatible.map((dependency,index)=><p key={index}>{dependency.file_name??dependency.project_id??dependency.version_id??t("Archivo incompatible")}</p>)}</details>}
      {review.warnings.map(warning=><p className="inline-note" key={warning}>{translatedMessage(warning)}</p>)}
      {review.changes.some(change=>change.action==='update')&&<p className="content-backup-note">{t("Se guardará una copia del servidor antes de actualizar.")}</p>}
      {main?.version.changelog&&<details className="content-changelog"><summary>{t("Cambios de esta versión")}</summary><div>{main.version.changelog}</div></details>}
      {reviewError&&<p className="form-error" role="alert">{translatedMessage(reviewError)}</p>}
      <div className="dialog-actions"><button data-modal-close disabled={locked}>{t("Cancelar")}</button>{reviewError?<button disabled={locked||!stopped} onClick={()=>{if(bulk)void prepareUpdates();else if(main)void prepare(main.project.id,main.version.id)}}>{t("Revisar de nuevo")}</button>:<button className="primary-button" disabled={locked||!stopped||changes.length===0} onClick={()=>void install()}>{installing?t("Instalando…"):preparing?t("Revisando…"):changes.length===0?t("Ya instalado"):t("Instalar {0} {1}", changes.length, changes.length===1?'archivo':'archivos')}</button>}</div>
    </Modal>}
  </section>
}
