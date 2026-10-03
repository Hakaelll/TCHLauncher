import {useEffect,useState} from 'react'
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
  const [progress,setProgress]=useState<Progress|null>(null)
  const supported=['fabric','neoforge'].includes(server.platform)&&!!server.jarPath
  useEffect(()=>{const subscription=listen<Progress>('install-progress',event=>{if(event.payload.serverId===server.id&&event.payload.kind==='loader_update')setProgress(event.payload)});return()=>{void subscription.then(unlisten=>unlisten()).catch(()=>{})}},[server.id])
  if(!supported)return null
  const check=async()=>{setWorking(true);setError(null);setWarning(null);try{const value=await api.loaderReview(server.id);setReview(value);setTarget(value.available[0]??'');setProgress(null);setOpen(true)}catch(cause){setError(String(cause))}finally{setWorking(false)}}
  const update=async()=>{if(!review)return;setWorking(true);setError(null);setProgress(null);try{const warning=await api.updateLoader(server.id,target,review.fingerprint);setWarning(warning);await onChanged();setOpen(false);setReview(null)}catch(cause){setError(String(cause));await onChanged().catch(()=>{})}finally{setWorking(false)}}
  return <section className="loader-panel"><div className="section-heading"><div><h3>{server.platform==='fabric'?'Fabric Loader':'NeoForge'}</h3><p>Actualiza el loader sin cambiar la versión de Minecraft.</p></div><button disabled={working||busy||!['stopped','crashed','error'].includes(server.state)} onClick={()=>void check()}>{working&&!open?'Buscando…':'Buscar actualizaciones'}</button></div>
    {error&&!open&&<p className="form-error" role="alert">{error}</p>}{warning&&<p className="inline-note" role="status">{warning}</p>}
    {progress&&!open&&<InstallProgress progress={progress} busy={working} onRetry={()=>void check()} onDismiss={()=>setProgress(null)}/>}
    {open&&review&&<Modal labelledBy="loader-title" busy={working} onClose={()=>{setOpen(false);setError(null)}}>
      <h2 id="loader-title">Actualizar {server.platform==='fabric'?'Fabric Loader':'NeoForge'}</h2><p className="field-help">Minecraft {server.version} · instalado: {review.current}</p>
      {review.available.length?<><label className="loader-version">Versión estable<select value={target} disabled={working} onChange={event=>setTarget(event.target.value)}>{review.available.map(version=><option key={version}>{version}</option>)}</select></label><p className="field-help">Se guardará una copia previa. Se conservarán tus mundos, mods y configuración. Comprueba después que tus mods funcionan con el nuevo loader.</p></>:<p>No hay una versión estable más reciente para este Minecraft.</p>}
      {progress&&<InstallProgress progress={progress} busy={working} onRetry={()=>void update()} onDismiss={()=>setProgress(null)}/>}
      {error&&<p className="form-error" role="alert">{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={working}>Cerrar</button>{review.available.length>0&&<button className="primary-button" disabled={working||!target} onClick={()=>void update()}>{working?'Actualizando…':'Crear copia y actualizar'}</button>}</div>
    </Modal>}
  </section>
}
