import {useEffect,useRef,useState} from 'react'
import {api,type NatPmpDetection,type NatPmpRecord,type NetworkSnapshot,type Server} from './api'
import {Modal} from './Modal'

const text=(cause:unknown)=>cause instanceof Error?cause.message:String(cause)
const phases={pending:'Comprobando apertura',active:'Apertura confirmada por el router',closing:'Retirada pendiente',attention:'Requiere revisión'}
export function NatPmpPanel({server,network}:{server:Server;network:NetworkSnapshot|null}){
  const [record,setRecord]=useState<NatPmpRecord|null>(null)
  const [detection,setDetection]=useState<NatPmpDetection|null>(null)
  const [gateway,setGateway]=useState('')
  const [busy,setBusy]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [copied,setCopied]=useState(false)
  const [consent,setConsent]=useState<{gateway:string;ip:string;port:number}|null>(null)
  const alive=useRef(false),inFlight=useRef(false),revision=useRef(0)
  useEffect(()=>{
    alive.current=true
    const refresh=async()=>{
      if(inFlight.current)return
      const current=++revision.current;inFlight.current=true
      try{const value=await api.natpmpStatus(server.id);if(alive.current&&current===revision.current)setRecord(value)}catch(cause){if(alive.current&&current===revision.current)setError(text(cause))}finally{inFlight.current=false}
    }
    void refresh();const timer=setInterval(()=>void refresh(),5000)
    return()=>{alive.current=false;++revision.current;clearInterval(timer)}
  },[server.id])
  const act=async(operation:()=>Promise<void>)=>{
    if(inFlight.current)return
    inFlight.current=true;++revision.current;setBusy(true);setError(null)
    try{await operation()}catch(cause){if(alive.current)setError(text(cause))}
    finally{
      try{const value=await api.natpmpStatus(server.id);if(alive.current)setRecord(value)}catch{/* Keep the operation error. */}
      inFlight.current=false;if(alive.current)setBusy(false)
    }
  }
  const detect=()=>act(async()=>{if(alive.current)setDetection(null);const value=await api.detectNatpmp(server.id,gateway.trim()||null);if(alive.current)setDetection(value)})
  const address=record?.phase==='active'&&record.externalIp&&record.externalPort&&record.expiresAt>Date.now()?`${record.externalIp}:${record.externalPort}`:null
  const canOpen=server.state==='running'&&network?.portState==='listening'&&!!network.lanAddress&&!!detection&&!record
  const copy=async()=>{if(!address)return;try{await navigator.clipboard.writeText(address);setCopied(true)}catch{setError('No se pudo copiar la dirección. Selecciónala para copiarla.') }}
  return <section className="router-control" aria-labelledby="natpmp-title"><div className="router-heading"><div><h3 id="natpmp-title">Alternativa del router · NAT-PMP</h3><p>Comprueba esta opción si no se detectó UPnP.</p></div><button disabled={busy||!!record} onClick={()=>void detect()}>{busy?'Comprobando…':'Detectar NAT-PMP'}</button></div>
    <details className="natpmp-gateway"><summary>Indicar otro router</summary><label htmlFor="natpmp-gateway">IPv4 del router<input id="natpmp-gateway" inputMode="decimal" placeholder="Detección automática" disabled={busy||!!record} value={gateway} onChange={event=>{setGateway(event.target.value);setDetection(null)}}/></label><p className="field-help">Déjalo vacío para usar la puerta de enlace de la interfaz del servidor.</p></details>
    {error&&<p className="network-error" role="alert">{error}</p>}
    {detection&&!record&&<dl className="data-list network-facts"><div><dt>Router compatible</dt><dd>{detection.gateway}</dd></div><div><dt>IP externa del router</dt><dd>{detection.externalIp}</dd></div></dl>}
    {detection?.publicAddress===false&&!record&&<p className="muted-note">La IP del router no es pública. Puede haber otro router o CGNAT; esta apertura puede no bastar para jugar desde internet.</p>}
    {record?<><p className="router-mapping-status" role="status"><span className={`state-dot ${record.phase==='active'&&record.expiresAt>Date.now()?'running':'starting'}`} aria-hidden="true"/>{record.expiresAt<=Date.now()?'Concesión vencida; pendiente de revisión':phases[record.phase]}</p><dl className="data-list network-facts"><div><dt>Destino local</dt><dd>{record.localIp}:{record.internalPort}</dd></div><div><dt>Router</dt><dd>{record.gateway}</dd></div><div><dt>Fin de concesión</dt><dd>{new Date(record.expiresAt).toLocaleTimeString()}</dd></div></dl>{address&&<div className="network-address-row"><input readOnly aria-label="Dirección externa NAT-PMP" value={address} onFocus={event=>event.currentTarget.select()}/><button onClick={()=>void copy()}>{copied?'Copiada':'Copiar'}</button></div>}{record.lastError&&<p className="network-error">{record.lastError}</p>}<button disabled={busy} onClick={()=>void act(async()=>{await api.closeNatpmp(server.id);if(alive.current){setConsent(null);setCopied(false)}})}>Retirar apertura NAT-PMP</button></>:<button className="primary-button" disabled={busy||!canOpen} onClick={()=>{if(detection&&network?.localIp)setConsent({gateway:detection.gateway,ip:network.localIp,port:network.port})}}>Abrir con NAT-PMP</button>}
    <p className="network-disclosure">Se solicita una hora y se renueva mientras el servidor está activo. Al detenerlo o cerrar el launcher se intenta retirar este puerto TCP. La respuesta del router no comprueba el acceso desde internet ni modifica el firewall.</p>
    {consent&&<Modal labelledBy="natpmp-confirm-title" busy={busy} onClose={()=>setConsent(null)}><h2 id="natpmp-confirm-title">Abrir el puerto con NAT-PMP</h2><p>El router {consent.gateway} dirigirá conexiones entrantes hacia {consent.ip}:{consent.port}. Puede asignar un puerto externo diferente. Se gestionará la concesión TCP de este destino; no se solicita retirar todos los puertos del equipo.</p>{error&&<p className="form-error" role="alert">{error}</p>}<div className="modal-actions"><button data-modal-close disabled={busy}>Cancelar</button><button className="primary-button" disabled={busy||server.state!=='running'} onClick={()=>void act(async()=>{await api.openNatpmp(server.id,consent.gateway,consent.ip,consent.port);if(alive.current)setConsent(null)})}>{busy?'Abriendo…':'Confirmar apertura temporal'}</button></div></Modal>}
  </section>
}
