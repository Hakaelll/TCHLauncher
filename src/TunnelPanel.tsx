import {useEffect,useRef,useState} from 'react'
import {api,type Server,type NetworkSnapshot,type TunnelStatus} from './api'
import {Modal} from './Modal'

const phases={creating:'Conectando…',active:'Túnel activo',closing:'Cerrando…',attention:'Conexión interrumpida'}
export function TunnelPanel({server,network}:{server:Server;network:NetworkSnapshot|null}){
  const [status,setStatus]=useState<TunnelStatus|null>(null)
  const [busy,setBusy]=useState(false)
  const [error,setError]=useState('')
  const [consent,setConsent]=useState<string|null>(null)
  const [copied,setCopied]=useState(false)
  const alive=useRef(false),mutating=useRef(false),revision=useRef(0),polling=useRef(false)
  useEffect(()=>{
    alive.current=true
    const poll=async()=>{
      if(polling.current||mutating.current)return
      polling.current=true;const version=revision.current
      try{const next=await api.tunnelStatus(server.id);if(alive.current&&version===revision.current)setStatus(next)}
      catch(reason){if(alive.current&&version===revision.current)setError(String(reason))}
      finally{polling.current=false}
    }
    void poll();const interval=setInterval(()=>void poll(),2000)
    return()=>{alive.current=false;clearInterval(interval)}
  },[server.id])
  async function act(work:()=>Promise<unknown>){
    if(mutating.current)return
    mutating.current=true;revision.current++;setBusy(true);setError('');setCopied(false)
    try{await work();if(alive.current)setConsent(null)}catch(reason){if(alive.current)setError(String(reason))}
    finally{
      try{const next=await api.tunnelStatus(server.id);if(alive.current)setStatus(next)}catch(reason){if(alive.current)setError(String(reason))}
      mutating.current=false;if(alive.current)setBusy(false)
    }
  }
  const ready=server.state==='running'&&network?.portState==='listening'
  const address=status?.phase==='active'&&status.endpoint?`${status.endpoint.host}:${status.endpoint.port}`:null
  return <section className="router-control" aria-labelledby="tunnel-title">
    <div className="router-heading"><div><h3 id="tunnel-title">Jugar con amigos · sin cuenta</h3><p>Un túnel de Bore conecta internet con tu servidor, incluso sin UPnP.</p></div><span className="status-pill">Bore</span></div>
    <p className="network-disclosure">El servidor sigue en tu ordenador. Las conexiones pasan por bore.pub; no necesitas registrarte ni cambiar el router. La dirección puede cambiar al volver a conectar y depende de la disponibilidad del servicio.</p>
    {error&&<p className="network-error" role="alert">{error}</p>}
    {status?<>
      <p className="router-mapping-status" role="status"><span className={`state-dot ${status.phase==='active'?'running':'starting'}`} aria-hidden="true"/>{phases[status.phase]}</p>
      {address&&<div className="network-address-row"><input aria-label="Dirección para compartir con amigos" readOnly value={address} onFocus={event=>event.currentTarget.select()}/><button onClick={()=>{void navigator.clipboard.writeText(address).then(()=>{if(alive.current)setCopied(true)},()=>{if(alive.current)setError('No se pudo copiar. Selecciona la dirección y cópiala.')})}}>{copied?'Copiada':'Copiar dirección'}</button></div>}
      {status.phase==='attention'&&<p>El túnel perdió la conexión. Ciérralo y vuelve a activarlo para obtener otra dirección.</p>}
      <button disabled={busy} onClick={()=>void act(()=>api.closeTunnel(server.id))}>{busy?'Cerrando…':'Cerrar túnel'}</button>
    </>:<button className="primary-button" disabled={busy||!ready} onClick={()=>{if(network)setConsent(network.localAddress)}}>{busy?'Conectando…':'Activar acceso sin cuenta'}</button>}
    {!ready&&!status&&<p className="network-disclosure">Inicia el servidor para activar el acceso.</p>}
    <p className="network-disclosure">Se cierra al detener el servidor o salir del launcher. Comparte la dirección solo mientras esté activo; tus amigos no necesitan instalar Bore.</p>
    {consent&&<Modal labelledBy="tunnel-confirm-title" busy={busy} onClose={()=>setConsent(null)}>
      <h2 id="tunnel-confirm-title">Activar acceso para amigos</h2><p>Bore conectará una dirección pública con tu servidor en {consent}. El tráfico pasará por bore.pub mientras el túnel esté activo. No cambia la autenticación ni los archivos del servidor.</p>
      {error&&<p className="form-error" role="alert">{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={busy}>Cancelar</button><button className="primary-button" disabled={busy||!ready} onClick={()=>void act(()=>api.openTunnel(server.id,consent))}>{busy?'Conectando…':'Activar túnel'}</button></div>
    </Modal>}
  </section>
}
