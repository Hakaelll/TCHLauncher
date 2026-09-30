import { useEffect, useState } from 'react'
import { api, type RuntimeInfo, type Server } from './api'

export function RuntimePanel({server,busy,onPrepare}: {server:Server;busy:boolean;onPrepare:(reinstall:boolean)=>void}) {
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
  return <section className="runtime-panel" aria-labelledby="runtime-heading">
    <div className="runtime-heading"><div className="runtime-mark" aria-hidden="true"><svg viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M9 16h13v7a5 5 0 0 1-5 5h-3a5 5 0 0 1-5-5zM22 17h2a3 3 0 0 1 0 6h-2M6 29h20M13 4c-4 4 5 5 1 9M19 2c-4 4 5 5 1 9"/></svg></div><div><h3 id="runtime-heading">Java · {info?.distribution ?? 'Eclipse Temurin'}</h3><p>{loading ? 'Comprobando el runtime…' : info?.status === 'ready' ? `Java ${info.version} · ${info.managed ? 'Gestionado por TCHLauncher' : 'Runtime externo'}` : 'Instala la versión compatible con este servidor.'}</p></div><span className={`runtime-state ${info?.status === 'ready' ? 'ready' : ''}`} role="status">{loading ? 'Comprobando' : info?.status === 'ready' ? 'Listo' : 'No disponible'}</span></div>
    {info?.path && <div className="runtime-path"><span>Ejecutable</span><code>{info.path}</code></div>}
    {(error || info?.error) && <p className="form-error" role="alert">{error || info?.error}</p>}
    <div className="runtime-actions"><button className="primary-button" disabled={busy || !stopped || loading} onClick={() => onPrepare(info?.status === 'ready')}>{info?.status === 'ready' ? 'Reinstalar Temurin' : 'Instalar Temurin'}</button><button disabled={busy || loading} onClick={() => setRevision(value=>value+1)}>Comprobar de nuevo</button></div>
    <p className="field-help">Java se descarga y verifica dentro de la biblioteca. Tu instalación global no cambia.</p>
  </section>
}
