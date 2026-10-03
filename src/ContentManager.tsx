import { useEffect, useRef, useState, type FormEvent } from 'react'
import { api, type ContentReview, type ContentSearch, type ContentVersion, type InstalledContent, type Progress, type Server } from './api'
import { Modal } from './Modal'
import { InstallProgress } from './InstallProgress'

const bytes=(n:number)=>`${(n/1024**2).toFixed(1)} MB`
const errorText=(e:unknown)=>e instanceof Error?e.message:String(e)
const actionLabels={install:'Añadir',update:'Actualizar',present:'Ya instalado'}
const environmentLabels:Record<string,string>={client_and_server:'Cliente y servidor',client_only_server_optional:'Cliente necesario · servidor opcional',server_only:'Solo servidor',server_only_client_optional:'Servidor · cliente opcional',dedicated_server_only:'Servidor dedicado',client_or_server:'Cliente o servidor',client_or_server_prefers_both:'Cliente y servidor recomendados'}

function ProjectIcon({url,title}:{url:string|null;title:string}) {
  const [image,setImage]=useState<string|null>(null)
  useEffect(()=>{let active=true;setImage(null);if(url)api.contentIcon(url).then(value=>{if(active)setImage(value)}).catch(()=>{});return()=>{active=false}},[url])
  return <span className="content-project-icon" aria-hidden="true">{image?<img src={image} alt=""/>:<span>{title.slice(0,1).toUpperCase()}</span>}</span>
}

export function ContentManager({server,progress,onInstalled,onDismissProgress}:{server:Server;progress:Progress|null;onInstalled:()=>Promise<void>;onDismissProgress:()=>void}) {
  const plugins=server.platform==='paper'
  const [mode,setMode]=useState<'installed'|'discover'>('installed')
  const [files,setFiles]=useState<InstalledContent[]>([])
  const [loading,setLoading]=useState(true)
  const [query,setQuery]=useState('')
  const [results,setResults]=useState<ContentSearch|null>(null)
  const [searching,setSearching]=useState(false)
  const [review,setReview]=useState<ContentReview|null>(null)
  const [versions,setVersions]=useState<ContentVersion[]>([])
  const [preparing,setPreparing]=useState<string|null>(null)
  const [installing,setInstalling]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [reviewError,setReviewError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [bulk,setBulk]=useState(false)
  const [removeTarget,setRemoveTarget]=useState<InstalledContent|null>(null)
  const searchSequence=useRef(0)
  const reviewSequence=useRef(0)
  const stopped=['stopped','error','crashed'].includes(server.state)
  const locked=installing||preparing!==null
  useEffect(()=>{let active=true;api.listContent(server.id).then(value=>{if(active)setFiles(value)}).catch(e=>{if(active)setError(errorText(e))}).finally(()=>{if(active)setLoading(false)});return()=>{active=false;searchSequence.current++;reviewSequence.current++}},[server.id])
  const search=async(offset=0)=>{
    const sequence=++searchSequence.current;setSearching(true);setError(null)
    try {const value=await api.searchContent(server.id,query,offset);if(sequence===searchSequence.current)setResults(value)}
    catch(e){if(sequence===searchSequence.current)setError(errorText(e))}
    finally{if(sequence===searchSequence.current)setSearching(false)}
  }
  const browse=()=>{setMode('discover');if(!results&&!searching)void search()}
  const submit=(event:FormEvent)=>{event.preventDefault();void search()}
  const prepare=async(projectId:string,versionId:string|null=null)=>{
    setBulk(false);const sequence=++reviewSequence.current;setPreparing(projectId);setError(null);setReviewError(null)
    try {
      const value=await api.planContent(server.id,projectId,versionId)
      if(sequence!==reviewSequence.current)return
      setReview(value)
      if(versionId===null) {
        const selected=value.plan.entries.find(entry=>!entry.required)
        if(selected) {const available=await api.contentVersions(server.id,selected.project.id);if(sequence===reviewSequence.current)setVersions(available)}
      }
    }catch(e){if(sequence===reviewSequence.current){if(review)setReviewError(errorText(e));else setError(errorText(e))}}
    finally{if(sequence===reviewSequence.current)setPreparing(null)}
  }
  const prepareUpdates=async()=>{
    const sequence=++reviewSequence.current;setPreparing('updates');setError(null);setReviewError(null);setBulk(true);setVersions([])
    try {const value=await api.planContentUpdates(server.id);if(sequence===reviewSequence.current)setReview(value)}
    catch(e){if(sequence===reviewSequence.current){if(review)setReviewError(errorText(e));else setError(errorText(e))}}
    finally{if(sequence===reviewSequence.current)setPreparing(null)}
  }
  const remove=async()=>{
    if(!removeTarget)return;setInstalling(true);setReviewError(null)
    try {await api.removeContent(server.id,removeTarget.filename,removeTarget.sha512);setFiles(await api.listContent(server.id));await onInstalled();setRemoveTarget(null);setNotice('Archivo retirado. Se ha guardado una copia del servidor.')}
    catch(e){setReviewError(errorText(e))}
    finally{setInstalling(false)}
  }
  const install=async()=>{
    if(!review)return;setInstalling(true);setReviewError(null);setNotice(null)
    try {await api.installContent(server.id,review.token);setFiles(await api.listContent(server.id));await onInstalled();setReview(null);setMode('installed');setNotice(plugins?'Plugins instalados. Se cargarán en el próximo inicio.':'Mods instalados. Se cargarán en el próximo inicio.')}
    catch(e){setReviewError(errorText(e));await onInstalled()}
    finally{setInstalling(false)}
  }
  const main=review?.plan.entries.find(entry=>!entry.required)
  const changes=review?.changes.filter(change=>change.action!=='present')??[]
  return <section className="content-manager" aria-busy={loading||searching||locked}>
    <div className="section-heading"><div><h2>{plugins?'Plugins':'Mods'}</h2><p>{server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'} · Minecraft {server.version}</p></div><div className="content-toolbar"><button disabled={!stopped||locked||!files.some(file=>file.projectId)} onClick={()=>void prepareUpdates()}>{preparing==='updates'?'Comprobando…':'Revisar actualizaciones'}</button><button disabled={locked} onClick={browse}>Explorar Modrinth <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="M5 12h14m-6-6 6 6-6 6"/></svg></button></div></div>
    <div className="content-mode" role="group" aria-label="Vista de contenido"><button aria-pressed={mode==='installed'} disabled={locked} onClick={()=>setMode('installed')}>Instalados <span>{files.length}</span></button><button aria-pressed={mode==='discover'} disabled={locked} onClick={browse}>Explorar</button></div>
    {!stopped&&<p className="inline-note">Puedes explorar mientras juega tu servidor. Deténlo para instalar o actualizar {plugins?'plugins':'mods'}.</p>}
    {error&&<p className="form-error" role="alert">{error}</p>}
    {notice&&<p className="content-success" role="status">{notice}</p>}
    {!review&&progress?.kind==='content_install'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} onRetry={()=>{onDismissProgress();setMode('discover')}} onDismiss={onDismissProgress}/>}
    {mode==='installed'?loading?<p role="status">Leyendo archivos…</p>:files.length?<div className="content-installed-list">{files.map(file=><div className="content-installed-row" key={file.filename}><span className={`content-file-mark ${file.modified?'modified':file.projectId?'managed':''}`} aria-hidden="true"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m12 3 9 5v9l-9 5-9-5V8zM3 8l9 5 9-5M12 13v9"/></svg></span><div><strong>{file.title}</strong><span>{file.versionNumber??'Archivo local'} · {bytes(file.size)}{file.modified?' · Modificado fuera del launcher':''}</span><small>{file.filename}</small></div><div className="content-row-actions">{file.projectId&&<button disabled={!stopped||locked} onClick={()=>{setVersions([]);void prepare(file.projectId!)}}>Revisar actualización</button>}<button className="text-button content-remove" disabled={!stopped||locked} onClick={()=>{setRemoveTarget(file);setReviewError(null)}} aria-label={`Retirar ${file.title}`}>Retirar</button></div></div>)}</div>:<div className="content-empty"><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.3" aria-hidden="true"><path d="m12 3 9 5v9l-9 5-9-5V8zM3 8l9 5 9-5M12 13v9"/></svg><h3>{plugins?'Tu servidor, con más posibilidades':'Haz tu mundo a tu medida'}</h3><p>{plugins?'Busca herramientas y nuevas mecánicas para Paper.':'Encuentra mods para esta versión y revisa lo que necesitan antes de instalarlos.'}</p><button className="primary-button" onClick={browse}>Buscar {plugins?'plugins':'mods'}</button></div>:<>
      <form className="content-search" onSubmit={submit}><label className="sr-only" htmlFor="content-search">Buscar {plugins?'plugins':'mods'}</label><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 5 5"/></svg><input id="content-search" type="search" value={query} maxLength={200} onChange={event=>setQuery(event.target.value)} placeholder={plugins?'Buscar plugins para Paper…':'Buscar mods compatibles…'}/><button disabled={searching||locked} type="submit">{searching?'Buscando…':'Buscar'}</button></form>
      <div className="content-results-label" role="status">{searching?'Consultando Modrinth…':results?`${results.total_hits.toLocaleString()} resultados para ${server.version}`:'Busca en Modrinth'}</div>
      {results?.hits.length===0&&!searching&&<p className="empty-inline">No hay resultados. Prueba con otro nombre.</p>}
      <div className="content-results">{results?.hits.map(project=><article className="content-project" key={project.project_id}><ProjectIcon url={project.icon_url} title={project.title}/><div><h3>{project.title}</h3><span className="content-author">{project.author}</span><p>{project.description}</p><span className="content-compatibility">{server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'} · {server.version}</span></div><button disabled={!stopped||locked} onClick={()=>{setVersions([]);void prepare(project.project_id)}}>{preparing===project.project_id?'Revisando…':files.some(file=>file.projectId===project.project_id)?'Revisar versión':'Revisar instalación'}</button></article>)}</div>
      {results&&results.total_hits>results.limit&&<div className="content-pagination"><button disabled={searching||results.offset===0||locked} onClick={()=>void search(Math.max(0,results.offset-results.limit))}>Anterior</button><span>{Math.floor(results.offset/results.limit)+1} / {Math.ceil(results.total_hits/results.limit)}</span><button disabled={searching||results.offset+results.limit>=results.total_hits||locked} onClick={()=>void search(results.offset+results.limit)}>Siguiente</button></div>}
    </>}
    {preparing&&!review&&<p className="inline-note" role="status">Comprobando versión y dependencias…</p>}
    {removeTarget&&<Modal labelledBy="content-remove-title" busy={locked} onClose={()=>setRemoveTarget(null)}><h2 id="content-remove-title">Retirar {removeTarget.title}</h2><p>Se guardará una copia del servidor y se retirará este archivo. Sus carpetas de configuración se conservarán.</p><p className="inline-note">{removeTarget.filename}</p>{!removeTarget.projectId&&<p className="inline-note">Este archivo se añadió manualmente. No se conocen sus dependencias.</p>}{reviewError&&<p className="form-error" role="alert">{reviewError}</p>}<div className="dialog-actions"><button data-modal-close disabled={locked}>Cancelar</button><button className="danger-button" disabled={locked||!stopped} onClick={()=>void remove()}>{installing?'Retirando…':'Sí, retirar archivo'}</button></div></Modal>}
    {review&&<Modal labelledBy="content-review-title" describedBy="content-review-description" busy={locked} className="content-review-dialog" onClose={()=>{setReview(null);setReviewError(null);reviewSequence.current++}}><h2 id="content-review-title">{bulk?'Revisar actualizaciones':main?.project.title??'Revisar instalación'}</h2><p id="content-review-description">{review.plan.loader==='paper'?'Plugin para Paper':'Mod para '+review.plan.loader} · Minecraft {review.plan.minecraftVersion}</p>
      {installing&&progress?.kind==='content_install'&&<InstallProgress key={progress.taskId} progress={progress} busy={locked} onRetry={()=>{}} onDismiss={()=>{}}/>}
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
