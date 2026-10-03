import {useEffect,useRef,useState} from 'react'
import {api,type NetworkSnapshot,type RouterStatus,type Server} from './api'
import {Modal} from './Modal'

const errorText=(error:unknown)=>error instanceof Error?error.message:String(error)
const phaseLabels={pending:'Comprobando apertura',active:'Apertura confirmada por el router',closing:'Retirada pendiente',attention:'Requiere revisión'}
const clock=(time:number)=>new Date(time).toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit',second:'2-digit'})

export function RouterPanel({server,network}:{server:Server;network:NetworkSnapshot|null}) {
  const [status,setStatus]=useState<RouterStatus|null>(null)
  const [error,setError]=useState<string|null>(null)
  const [busy,setBusy]=useState<'detect'|'open'|'close'|null>(null)
  const [consent,setConsent]=useState<{ip:string;port:number}|null>(null)
  const alive=useRef(false),inFlight=useRef(false),sequence=useRef(0)
  useEffect(()=>{
    alive.current=true
    const refresh=async()=>{
      if(inFlight.current)return
      const request=++sequence.current
      try {const value=await api.routerStatus(server.id);if(alive.current&&request===sequence.current){setStatus(value)}}
      catch(cause){if(alive.current&&request===sequence.current)setError(errorText(cause))}
    }
    void refresh()
    const timer=window.setInterval(()=>void refresh(),5000)
    return()=>{alive.current=false;++sequence.current;window.clearInterval(timer)}
  },[server.id])
  const act=async(kind:'detect'|'open'|'close',operation:()=>Promise<RouterStatus>)=>{
    if(inFlight.current)return
    inFlight.current=true;++sequence.current;setBusy(kind);setError(null)
    try {
      const value=await operation()
      if(alive.current){setStatus(value);if(kind==='open')setConsent(null)}
    } catch(cause) {
      if(alive.current)setError(errorText(cause))
      try{const value=await api.routerStatus(server.id);if(alive.current)setStatus(value)}catch{/* Preserve the actionable operation error. */}
    } finally {inFlight.current=false;if(alive.current)setBusy(null)}
  }
  const mapping=status?.mapping
  const expired=!!mapping?.expiresAt&&mapping.expiresAt<=Date.now()
  const canOpen=server.state==='running'&&network?.portState==='listening'&&!!network.lanAddress&&!!status?.router&&!mapping
  return <section className="router-control" aria-labelledby="router-title">
    <div className="router-heading"><div><h3 id="router-title">Apertura automática · UPnP</h3><p>Una concesión temporal para el puerto TCP de este servidor.</p></div><button disabled={!!busy} onClick={()=>void act('detect',()=>api.detectRouter(server.id))}>{busy==='detect'?'Detectando…':'Detectar router'}</button></div>
    {error&&<p className="network-error" role="alert">{error}</p>}
    {status?.discoveryError&&<p className="muted-note">{status.discoveryError}</p>}
    {status?.router&&<dl className="data-list network-facts"><div><dt>Router</dt><dd>{status.router.name} · {status.router.ip}</dd></div><div><dt>IP externa del router</dt><dd>{status.router.externalIp||'No disponible'}</dd></div></dl>}
    {status?.router?.publicAddress===false&&<p className="muted-note">El router no tiene una IPv4 pública utilizable. Esta apertura por sí sola puede ser insuficiente si hay otro router o CGNAT.</p>}
    {mapping?<>
      <p className="router-mapping-status" role="status"><span className={`state-dot ${mapping.phase==='active'&&!expired?'running':'starting'}`} aria-hidden="true"/>{expired&&mapping.phase==='active'?'Concesión vencida; pendiente de revisión':phaseLabels[mapping.phase]}</p>
      <dl className="data-list network-facts"><div><dt>Destino local</dt><dd>{mapping.localAddress}</dd></div>{mapping.externalAddress&&<div><dt>Dirección externa</dt><dd><input readOnly aria-label="Dirección externa del servidor" value={mapping.externalAddress} onFocus={event=>event.currentTarget.select()}/></dd></div>}{mapping.verifiedAt&&<div><dt>Última comprobación del router</dt><dd>{clock(mapping.verifiedAt)}</dd></div>}{mapping.expiresAt&&<div><dt>Fin de concesión confirmado</dt><dd>{clock(mapping.expiresAt)}</dd></div>}</dl>
      {mapping.lastError&&<p className="network-error">{mapping.lastError}</p>}
      <button disabled={!!busy} onClick={()=>void act('close',()=>api.closeRouterPort(server.id))}>{busy==='close'?'Retirando…':'Retirar apertura'}</button>
    </>:<button className="primary-button" disabled={!!busy||!canOpen} onClick={()=>{if(network?.localIp)setConsent({ip:network.localIp,port:network.port})}}>Abrir puerto temporalmente</button>}
    {!mapping&&server.state!=='running'&&<p className="muted-note">Inicia el servidor para abrir el puerto.</p>}
    <p className="network-disclosure">La confirmación del router no verifica el acceso desde internet ni modifica el firewall. La concesión se renueva mientras el servidor está activo; al detenerlo o cerrar el launcher se intenta retirarla. Si el router no responde, queda pendiente y la concesión temporal puede caducar.</p>
    {consent&&<Modal labelledBy="router-consent-title" describedBy="router-consent-description" busy={!!busy} onClose={()=>setConsent(null)}>
      <h2 id="router-consent-title">Abrir el puerto {consent.port} TCP</h2>
      <p id="router-consent-description">Solicitar una apertura temporal hacia {consent.ip}:{consent.port} permitirá que el router dirija conexiones entrantes a este servidor. Duración solicitada: {Math.round((status?.leaseSeconds??3600)/60)} minutos, renovable mientras esté activo.</p>
      {error&&<p className="form-error" role="alert">{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={!!busy}>Cancelar</button><button className="primary-button" disabled={!!busy||server.state!=='running'} onClick={()=>void act('open',()=>api.openRouterPort(server.id,consent.ip,consent.port))}>{busy==='open'?'Abriendo…':'Confirmar apertura'}</button></div>
    </Modal>}
  </section>
}
