import {useEffect,useState} from 'react'
import {api,type Server} from './api'
import {Modal} from './Modal'

export function ForceStopDialog({server,onClose,onStopped}:{server:Server;onClose:()=>void;onStopped:()=>void}) {
  const [preview,setPreview]=useState<{sessionId:string;pid:number}|null>(null)
  const [error,setError]=useState<string|null>(null)
  const [busy,setBusy]=useState(false)
  useEffect(()=>{
    let active=true
    api.previewForceStop(server.id).then(value=>{if(active)setPreview(value)}).catch(error=>{if(active)setError(String(error))})
    return()=>{active=false}
  },[server.id])
  const confirm=async()=>{
    if(!preview||busy)return
    setBusy(true);setError(null)
    try {await api.forceStop(server.id,preview.sessionId);onStopped();onClose()}
    catch(error){setError(String(error));setPreview(null)}
    finally{setBusy(false)}
  }
  return <Modal labelledBy="force-stop-title" describedBy="force-stop-description" busy={busy} onClose={onClose}>
    <h2 id="force-stop-title">Forzar cierre de «{server.name}»</h2>
    <p id="force-stop-description">La parada normal ya se ha solicitado. Si sigue guardando el mundo, espera a que termine. Forzar el cierre puede perder cambios o dañar archivos.</p>
    <p className="inline-note">Este cierre no se contará como una parada correcta ni iniciará la copia automática de parada.</p>
    {preview?<p className="field-help">Proceso Java: {preview.pid}</p>:!error&&<p role="status">Comprobando el proceso…</p>}
    {error&&<p className="form-error" role="alert">{error}</p>}
    <div className="dialog-actions"><button data-modal-close autoFocus disabled={busy}>Seguir esperando</button><button className="danger-button" disabled={busy||!preview} onClick={()=>void confirm()}>{busy?'Cerrando…':'Sí, forzar cierre'}</button></div>
  </Modal>
}
