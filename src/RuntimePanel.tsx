import { useEffect, useRef, useState } from 'react'
import { Modal } from './Modal'
import { api, type RuntimeInfo, type Server } from './api'

export function RuntimePanel({server,busy,onPrepare,onChanged}: {server:Server;busy:boolean;onPrepare:(reinstall:boolean)=>void;onChanged:()=>Promise<void>}) {
  const [candidate,setCandidate]=useState<RuntimeInfo|null>(null)
  const [choosing,setChoosing]=useState(false)
  const [saving,setSaving]=useState(false)
  const cancelRef=useRef<HTMLButtonElement>(null)
  const mounted=useRef(true)
  useEffect(()=>{mounted.current=true;return ()=>{mounted.current=false}},[])
  const choose=async()=>{
    setChoosing(true);setError(null)
    try {const value=await api.pickCustomRuntime(server.id);if(mounted.current)setCandidate(value)}
    catch(cause){if(mounted.current)setError(String(cause))}
    finally{if(mounted.current)setChoosing(false)}
  }
  const apply=async()=>{
    if(!candidate?.path)return
    setSaving(true);setError(null)
    try {await api.setCustomRuntime(server.id,candidate.path);if(mounted.current){setCandidate(null);setRevision(value=>value+1)};await onChanged()}
    catch(cause){if(mounted.current)setError(String(cause))}
    finally{if(mounted.current)setSaving(false)}
  }
  const [info,setInfo] = useState<RuntimeInfo|null>(null)
  const [error,setError] = useState<string|null>(null)
  const [loading,setLoading] = useState(true)
  const [revision,setRevision] = useState(0)
  useEffect(() => {
    let active = true
    setLoading(true);setError(null)
    api.runtimeInfo(server.id).then(value => {if (active) setInfo(value)}).catch(cause => {if (active) setError(String(cause))}).finally(() => {if (active) setLoading(false)})
    return () => {active=false}
  },[server.id,server.javaPath,server.state,revision])
  const stopped = ['stopped','crashed','error'].includes(server.state)
  const compatible = info?.status === 'ready' && info.compatibility !== 'incompatible'
  const status = loading ? 'checking' : compatible ? 'ready' : info?.compatibility === 'incompatible' ? 'incompatible' : 'missing'
  return <section className="runtime-panel" aria-labelledby="runtime-heading" aria-busy={loading || choosing || saving}>
    <div className="runtime-heading">
      <div className="runtime-mark" aria-hidden="true"><svg viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M7 16h16v9H7zM23 17h4v6h-4M5 28h22M12 4v4h3v4M19 2v4h3v5"/></svg></div>
      <div className="runtime-identity"><h3 id="runtime-heading">Java · {info?.distribution ?? 'Eclipse Temurin'}</h3><p>{loading ? 'Comprobando el runtime…' : info?.status === 'ready' ? `Java ${info.version} · ${info.managed ? 'Gestionado por TCHLauncher' : 'Runtime externo'}` : 'Prepara Java para este servidor.'}</p></div>
      <span className={`runtime-state ${status}`} role="status">{loading ? 'Comprobando' : compatible ? info?.compatibility === 'compatible' ? 'Listo' : 'Versión comprobada' : status === 'incompatible' ? 'Incompatible' : 'No disponible'}</span>
    </div>
    {info?.requiredMajor && <p className="runtime-requirement">Minecraft {server.version}<span aria-hidden="true">·</span>Requiere Java {info.requiredMajor}</p>}
    {(error || info?.error) && <p className="form-error" role="alert">{error || info?.error}</p>}
    <div className="runtime-actions"><button className="primary-button" disabled={busy || !stopped || loading || choosing || saving} onClick={() => onPrepare(info?.status === 'ready' && info.managed)}>{info?.status === 'ready' ? info.managed?'Reinstalar Temurin':'Usar Temurin gestionado' : 'Instalar Temurin'}</button><button disabled={busy || loading || choosing || saving} onClick={() => setRevision(value=>value+1)}>Comprobar de nuevo</button></div>
    <div className="runtime-details">
      {info?.path && <details className="runtime-location"><summary>Ubicación del runtime</summary><div className="runtime-path"><code>{info.path}</code></div></details>}
      <details className="runtime-advanced"><summary>Java personalizado</summary><p className="field-help">Elige un runtime instalado. Comprobaremos su versión antes de cambiarlo.</p><button disabled={busy || !stopped || choosing || saving} onClick={choose}>{choosing ? 'Comprobando Java…' : 'Elegir ejecutable Java'}</button></details>
    </div>
    {candidate && <Modal labelledBy="custom-java-title" describedBy="custom-java-description" busy={saving} initialFocusRef={cancelRef} onClose={()=>setCandidate(null)}><h2 id="custom-java-title">Usar Java {candidate.version}</h2><p id="custom-java-description">Compatible con este servidor. El cambio solo afecta a su configuración en TCHLauncher.</p><div className="runtime-path"><span>Ejecutable</span><code>{candidate.path}</code></div>{error && <p className="form-error" role="alert">{error}</p>}<div className="world-confirm-actions"><button data-modal-close ref={cancelRef} disabled={saving} onClick={()=>setCandidate(null)}>Cancelar</button><button className="primary-button" disabled={saving || busy || !stopped} onClick={apply}>{saving ? 'Guardando…' : 'Usar este Java'}</button></div></Modal>}
    <p className="runtime-footnote">El runtime gestionado se guarda y verifica en la biblioteca del launcher.</p>
  </section>
}
