import {translatedMessage,t} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type Server,type LoaderReview,type Progress} from './api'
import {Modal} from './Modal'
import {InstallProgress} from './InstallProgress'

export function LoaderPanel({server,busy,onChanged}:{server:Server;busy:boolean;onChanged:()=>Promise<void>}) {
  const [review,setReview]=useState<LoaderReview|null>(null)
  const [target,setTarget]=useState('')
  const [working,setWorking]=useState(false)
  const [open,setOpen]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [warning,setWarning]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [syncError,setSyncError]=useState<string|null>(null)
  const scope=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};scope.current=current;setReview(null);setTarget('');setOpen(false);setError(null);setWarning(null);setNotice(null);setSyncError(null);setWorking(false);setProgress(null);return()=>{current.active=false}},[server.id])
  const [progress,setProgress]=useState<Progress|null>(null)
  const supported=['fabric','neoforge'].includes(server.platform)&&!!server.jarPath
  useEffect(()=>{
    let active=true
    const subscription=listen<Progress>('install-progress',event=>{if(active&&event.payload.serverId===server.id&&event.payload.kind==='loader_update')setProgress(event.payload)})
    return()=>{active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[server.id])
  if(!supported)return null
  const stopped=['stopped','crashed','error'].includes(server.state)
  const owns=(current:typeof scope.current)=>current.active&&scope.current===current
  const sync=async(current=scope.current)=>{
    try{await onChanged();if(owns(current))setSyncError(null)}
    catch(cause){if(owns(current))setSyncError(t("El loader se guardó, pero no se pudieron actualizar los datos: {0}", String(cause)))}
  }
  const retrySync=async()=>{
    if(working||busy||scope.current.working)return
    const current=scope.current;current.working=true;setWorking(true)
    try{await sync(current)}finally{current.working=false;if(owns(current))setWorking(false)}
  }
  const check=async()=>{
    if(working||busy||!stopped||scope.current.working)return
    const current=scope.current;current.working=true
    setWorking(true);setError(null);setReview(null);setProgress(null)
    try{
      const value=await api.loaderReview(server.id)
      if(owns(current)){setReview(value);setTarget(value.available[0]??'');setOpen(true)}
    }catch(cause){if(owns(current))setError(String(cause))}
    finally{current.working=false;if(owns(current))setWorking(false)}
  }
  const update=async()=>{
    if(!review||!review.available.includes(target)||working||busy||!stopped||scope.current.working)return
    const current=scope.current,version=target;current.working=true
    setWorking(true);setError(null);setProgress(null);setNotice(null);setWarning(null);setSyncError(null)
    try{
      const warning=await api.updateLoader(server.id,version,review.fingerprint)
      if(!owns(current))return
      setOpen(false);setReview(null);setWarning(warning);setNotice(t("Loader actualizado a {0}. Se aplicará en el próximo inicio; comprueba tus mods desde el juego.", version))
      await sync(current)
    }catch(cause){if(owns(current))setError(String(cause));await onChanged().catch(()=>{})}
    finally{current.working=false;if(owns(current))setWorking(false)}
  }
  return <section className="loader-panel"><div className="section-heading"><div><h3>{server.platform==='fabric'?'Fabric Loader':'NeoForge'}</h3><p>{t("Actualiza el loader sin cambiar la versión de Minecraft.")}</p></div><button disabled={working||busy||!stopped} onClick={()=>void check()}>{working&&!open?t("Buscando…"):t("Buscar actualizaciones")}</button></div>
    {error&&!open&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}{warning&&<p className="inline-note" role="status">{translatedMessage(warning)}</p>}{notice&&<p className="loader-acknowledgement" role="status">{translatedMessage(notice)}</p>}{syncError&&<div className="loader-sync-error"><p className="form-error" role="alert">{translatedMessage(syncError)}</p><button disabled={working||busy} onClick={()=>void retrySync()}>{t("Actualizar datos")}</button></div>}
    {progress&&!open&&<InstallProgress progress={progress} busy={working} onRetry={()=>void check()} onDismiss={()=>setProgress(null)}/>}
    {open&&review&&<Modal labelledBy="loader-title" busy={working} onClose={()=>{setOpen(false);setError(null)}}>
      <h2 id="loader-title">{t("Actualizar")} {server.platform==='fabric'?'Fabric Loader':'NeoForge'}</h2><p className="field-help">Minecraft {server.version}  {t("· instalado:")} {review.current}</p>
      {review.available.length?<><label className="loader-version">{t("Versión estable")}<select value={target} disabled={working} onChange={event=>setTarget(event.target.value)}>{review.available.map(version=><option key={version}>{version}</option>)}</select></label><p className="field-help">{t("Se guardará una copia previa. Se conservarán tus mundos, mods y configuración. Comprueba después que tus mods funcionan con el nuevo loader.")}</p></>:<p>{t("No hay una versión estable más reciente para este Minecraft.")}</p>}
      {progress&&<InstallProgress progress={progress} busy={working} onRetry={()=>void update()} onDismiss={()=>setProgress(null)}/>}
      {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>{t("Cerrar")}</button>{review.available.length>0&&<button className="primary-button" disabled={working||busy||!stopped||!review.available.includes(target)} onClick={()=>void update()}>{working?t("Actualizando…"):t("Crear copia y actualizar")}</button>}</div>
    </Modal>}
  </section>
}
