import { useEffect, useRef, useState } from 'react'
import { api, type ContentReview, type ContentVersion, type InstalledContent, type Progress, type LogLine, type Server } from './api'
import { isTaskActive } from './taskState'
import { Modal } from './Modal'
import { InstallProgress } from './InstallProgress'
import {ContentDiscovery} from './ContentDiscovery'
import {InstalledContentView} from './InstalledContentView'
import {useServerRead} from './useServerRead'

const errorText=(e:unknown)=>e instanceof Error?e.message:String(e)
const actionLabels={install:'Añadir',update:'Actualizar',present:'Ya instalado'}
const environmentLabels:Record<string,string>={client_and_server:'Cliente y servidor',client_only_server_optional:'Cliente necesario · servidor opcional',server_only:'Solo servidor',server_only_client_optional:'Servidor · cliente opcional',dedicated_server_only:'Servidor dedicado',client_or_server:'Cliente o servidor',client_or_server_prefers_both:'Cliente y servidor recomendados'}

export function ContentManager({server,progress,logs,onConsole,onInstalled,onDismissProgress,onImportPack}:{server:Server;progress:Progress|null;logs:LogLine[];onConsole:(query:string)=>void;onInstalled:()=>Promise<void>;onDismissProgress:()=>void;onImportPack:()=>void}) {
  const plugins=server.platform==='paper'
  const [mode,setMode]=useState<'installed'|'discover'>('installed')
  const inventory=useServerRead(server.id,api.auditContent)
  const audit=inventory.data,files=audit?.files??[],loading=inventory.loading
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
  const browse=()=>{setHasBrowsed(true);setMode('discover')}
  const refresh=async()=>{await inventory.refresh(server.id)}
  const sync=async(current=owner.current)=>{
    await inventory.refresh(server.id)
    if(!owns(current))return
    try{await onInstalled();if(owns(current))setSyncError(null)}
    catch(e){if(owns(current))setSyncError(`La operación terminó, pero no se pudieron actualizar los datos del servidor: ${errorText(e)}`)}
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
      if(!selectedId)throw new Error('No hay una publicación compatible con este servidor.')
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
      setRemoveTarget(null);setNotice('Archivo retirado. Se ha guardado una copia del servidor.')
      await sync(current)
    }catch(e){if(owns(current)&&errorText(e)!=='Instalación cancelada')setReviewError(errorText(e))}
    finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const install=async()=>{
    const current=owner.current
    if(!review||current.working||locked||!stopped)return
    current.working=true;setInstalling(true);setReviewError(null);setNotice(null);setSyncError(null)
    try {
      await api.installContent(server.id,review.token)
      if(!owns(current))return
      setReview(null);setNotice(plugins?'Plugins instalados. Se cargarán en el próximo inicio.':'Mods instalados. Reinicia para comprobar su carga.')
      await sync(current)
    }catch(e){
      if(owns(current)){setReviewError(errorText(e));await inventory.refresh(server.id)}
    }
    finally{current.working=false;if(owns(current))setInstalling(false)}
  }
  const main=review?.plan.entries.find(entry=>!entry.required)
  const changes=review?.changes.filter(change=>change.action!=='present')??[]
  return <section className="content-manager" aria-busy={loading||locked}>
    <div className="section-heading"><div><h2>Contenido</h2><p>{server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'} · Minecraft {server.version}</p></div><div className="content-toolbar"><button disabled={!stopped||locked||loading||!!inventory.error||!files.some(file=>file.projectId)} onClick={()=>void prepareUpdates()}>{preparing==='updates'?'Comprobando…':'Revisar actualizaciones'}</button><button disabled={locked} onClick={browse}>Explorar Modrinth <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="M5 12h14m-6-6 6 6-6 6"/></svg></button></div></div>
    <div className="content-mode" role="group" aria-label="Vista de contenido"><button aria-pressed={mode==='installed'} disabled={locked} onClick={()=>setMode('installed')}>Instalados <span>{audit?files.length:'—'}</span></button><button aria-pressed={mode==='discover'} disabled={locked} onClick={browse}>Explorar</button></div>
    {!stopped&&<p className="inline-note">Puedes explorar mientras juega tu servidor. Detén el servidor para instalar o actualizar {plugins?'plugins':'mods'}.</p>}
    {error&&<p className="form-error" role="alert">{error}</p>}
    {notice&&<p className="content-success" role="status">{notice}</p>}
    {inventory.error&&<div className="panel-read-state failed"><div><strong>Inventario sin confirmar</strong><p>{inventory.error}{audit?' Se conservan los últimos archivos leídos.':''}</p></div><button disabled={loading||locked} onClick={()=>void refresh()}>Reintentar lectura</button></div>}
    {syncError&&<div className="loader-sync-error"><p className="form-error" role="alert">{syncError}</p><button disabled={loading||locked} onClick={()=>void retrySync()}>Actualizar datos</button></div>}
    {!review&&!removeTarget&&(progress?.kind==='content_install'||progress?.kind==='content_remove')&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} onRetry={()=>{onDismissProgress();if(progress?.kind==='content_remove')setMode('installed');else browse()}} onDismiss={onDismissProgress}/>}
    <div hidden={mode!=='installed'}>
      <div className="content-inventory-heading"><div><h3>Inventario del servidor</h3><p>Iconos, versiones y avisos de cada archivo.</p></div><button disabled={loading||locked} onClick={()=>void refresh()}>{loading?'Revisando…':'Revisar archivos'}</button></div>
      {audit&&files.length>0&&<div className="content-audit-summary"><details><summary>Revisión de carga{server.platform==='fabric'?` · ${audit.checks.filter(check=>check.listedInLog===true).length} presentes en el último log`: ''}</summary><p>{audit.logNote}{audit.logModifiedAt?` Log actualizado: ${new Date(audit.logModifiedAt).toLocaleString('es-ES')}.`:''}</p><p>Revisamos el descriptor, el cargador y los identificadores duplicados. En Fabric se revisan rangos reconocidos entre mods principales; los requisitos del runtime y la selección de archivos incluidos siguen a cargo del loader. Un mod de estructuras puede afectar solo a terreno nuevo; algunos mods también deben instalarse en los clientes.</p></details></div>}
      {!audit?<p role="status">{loading?'Revisando archivos y log…':'Reintenta la lectura para ver los archivos del servidor.'}</p>:<InstalledContentView server={server} files={files} audit={audit} logs={logs} locked={locked||loading||!!inventory.error} stopped={stopped} onBrowse={browse} onConsole={onConsole} onPrepare={id=>{setVersions([]);void prepare(id)}} onRemove={file=>{if(!stopped||locked||loading||inventory.error)return;if(progress&&!isTaskActive(progress))onDismissProgress();setRemoveTarget(file);setReviewError(null)}}/>}
    </div>
    <div hidden={mode!=='discover'}><ContentDiscovery onImportPack={onImportPack} server={server} active={hasBrowsed} files={files} locked={locked} stopped={stopped} preparing={preparing} onPrepare={(id,version)=>{setVersions([]);void prepare(id,version)}}/></div>
    {preparing&&!review&&<p className="inline-note" role="status">Comprobando versión y dependencias…</p>}
    {removeTarget&&<Modal labelledBy="content-remove-title" busy={locked} onClose={()=>setRemoveTarget(null)}><h2 id="content-remove-title">Retirar {removeTarget.title}</h2><p>Se guardará una copia del servidor y se retirará este archivo. Sus carpetas de configuración se conservarán.</p><p className="inline-note">{removeTarget.filename}</p>{progress?.kind==='content_remove'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} allowRetry={false} onRetry={()=>{}} onDismiss={onDismissProgress}/>}{!removeTarget.projectId&&<p className="inline-note">Este archivo se añadió manualmente. No se conocen sus dependencias.</p>}{reviewError&&<p className="form-error" role="alert">{reviewError}</p>}<div className="dialog-actions"><button data-modal-close disabled={locked}>Cancelar</button><button className="danger-button" disabled={locked||!stopped||loading||!!inventory.error} onClick={()=>void remove()}>{installing?'Retirando…':'Sí, retirar archivo'}</button></div></Modal>}
    {review&&<Modal labelledBy="content-review-title" describedBy="content-review-description" busy={locked} className="content-review-dialog" onClose={()=>{setReview(null);setReviewError(null);reviewSequence.current++}}><h2 id="content-review-title">{bulk?'Revisar actualizaciones':main?.project.title??'Revisar instalación'}</h2><p id="content-review-description">{review.plan.loader==='paper'?'Plugin para Paper':'Mod para '+review.plan.loader} · Minecraft {review.plan.minecraftVersion}</p>
      {installing&&progress?.kind==='content_install'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} onRetry={()=>{}} onDismiss={()=>{}}/>}
      {!bulk&&main&&main.version.version_type!=='release'&&<p className="inline-note">Publicación {main.version.version_type}: puede ser experimental. Revisa los cambios antes de instalarla.</p>}
      {!bulk&&main&&<p className="content-environment">{environmentLabels[main.version.environment??'']??'Compatible con servidor'}</p>}
      {!bulk&&main&&versions.length>0&&<label className="content-version-label">Versión compatible<select value={main.version.id} disabled={locked} onChange={event=>void prepare(main.project.id,event.target.value)}>{versions.map(version=><option key={version.id} value={version.id}>{version.version_number}{version.version_type==='release'?'':` · ${version.version_type}`}</option>)}</select></label>}
      <div className="content-review-changes">{review.changes.map(change=><div key={change.filename}><div><strong>{change.title}</strong><span>{change.previousVersion&&change.action==='update'?`${change.previousVersion} → `:''}{change.version}</span></div><span className={`content-change-action ${change.action}`}>{actionLabels[change.action]}</span></div>)}</div>
      {review.plan.entries.filter(entry=>entry.required).length>0&&<p className="inline-note">Las dependencias obligatorias están incluidas en esta revisión.</p>}
      {review.plan.optional.length>0&&<details className="content-dependencies"><summary>Dependencias opcionales ({review.plan.optional.length})</summary>{review.plan.optional.map((dependency,index)=><p key={index}>{dependency.file_name??dependency.project_id??dependency.version_id??'Dependencia externa'}</p>)}</details>}
      {review.plan.incompatible.length>0&&<details className="content-dependencies"><summary>Incompatibilidades declaradas ({review.plan.incompatible.length})</summary>{review.plan.incompatible.map((dependency,index)=><p key={index}>{dependency.file_name??dependency.project_id??dependency.version_id??'Archivo incompatible'}</p>)}</details>}
      {review.warnings.map(warning=><p className="inline-note" key={warning}>{warning}</p>)}
      {review.changes.some(change=>change.action==='update')&&<p className="content-backup-note">Se guardará una copia del servidor antes de actualizar.</p>}
      {main?.version.changelog&&<details className="content-changelog"><summary>Cambios de esta versión</summary><div>{main.version.changelog}</div></details>}
      {reviewError&&<p className="form-error" role="alert">{reviewError}</p>}
      <div className="dialog-actions"><button data-modal-close disabled={locked}>Cancelar</button>{reviewError?<button disabled={locked||!stopped} onClick={()=>{if(bulk)void prepareUpdates();else if(main)void prepare(main.project.id,main.version.id)}}>Revisar de nuevo</button>:<button className="primary-button" disabled={locked||!stopped||changes.length===0} onClick={()=>void install()}>{installing?'Instalando…':preparing?'Revisando…':changes.length===0?'Ya instalado':`Instalar ${changes.length} ${changes.length===1?'archivo':'archivos'}`}</button>}</div>
    </Modal>}
  </section>
}
