import {errorText} from './format'
import {startVisibilityPolling} from './visibilityPolling'
import {translatedMessage,localizedStatic,t,formatTime} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {api,type NatPmpDetection,type NatPmpRecord,type NetworkSnapshot,type Server} from './api'
import {Modal} from './Modal'

const phases=localizedStatic({pending:'Comprobando apertura',active:'Apertura confirmada por el router',closing:'Retirada pendiente',attention:'Requiere revisión'})
export function NatPmpPanel({server,network}:{server:Server;network:NetworkSnapshot|null}){
  const [loaded,setLoaded]=useState(0)
  const attempted=useRef('')
  const [record,setRecord]=useState<NatPmpRecord|null>(null)
  const [detection,setDetection]=useState<NatPmpDetection|null>(null)
  const [gateway,setGateway]=useState('')
  const [busy,setBusy]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [statusError,setStatusError]=useState<string|null>(null)
  const [copied,setCopied]=useState(false)
  const [consent,setConsent]=useState<{gateway:string;ip:string;port:number}|null>(null)
  const alive=useRef(false),inFlight=useRef(false),revision=useRef(0)
  useEffect(()=>{
    alive.current=true
    const refresh=async()=>{
      if(inFlight.current)return
      const current=++revision.current;inFlight.current=true
      try{const value=await api.natpmpStatus(server.id);if(alive.current&&current===revision.current){setRecord(value);setStatusError(null)}}catch(cause){if(alive.current&&current===revision.current)setStatusError(errorText(cause))}finally{inFlight.current=false;if(alive.current)setLoaded(value=>value+1)}
    }
    const stop=startVisibilityPolling(refresh,5000,{immediate:true})
    return()=>{alive.current=false;++revision.current;stop()}
  },[server.id])
  const act=async(operation:()=>Promise<void>)=>{
    if(inFlight.current)return
    inFlight.current=true;++revision.current;setBusy(true);setError(null)
    try{await operation()}catch(cause){if(alive.current)setError(errorText(cause))}
    finally{
      try{const value=await api.natpmpStatus(server.id);if(alive.current){setRecord(value);setStatusError(null)}}catch(cause){if(alive.current)setStatusError(errorText(cause))}
      inFlight.current=false;if(alive.current)setBusy(false)
    }
  }
  const detect=()=>act(async()=>{if(alive.current)setDetection(null);const value=await api.detectNatpmp(server.id,gateway.trim()||null);if(alive.current)setDetection(value)})
  useEffect(()=>{
    const key=network?.lanAddress?`${server.id}:${network.localIp}`:''
    if(!loaded||!key||record||busy||inFlight.current||attempted.current===key)return
    attempted.current=key
    void act(async()=>{const value=await api.detectNatpmp(server.id,null);if(alive.current)setDetection(value)})
  },[loaded,server.id,network?.localIp,network?.lanAddress,record,busy])
  const address=record?.phase==='active'&&record.externalIp&&record.externalPort&&record.expiresAt>Date.now()?`${record.externalIp}:${record.externalPort}`:null
  const canOpen=!statusError&&server.state==='running'&&network?.portState==='listening'&&!!network.lanAddress&&!!detection&&!record
  const copy=async()=>{if(!address)return;try{await navigator.clipboard.writeText(address);setCopied(true)}catch{setError(t("No se pudo copiar la dirección. Selecciónala para copiarla.")) }}
  return <section className="router-control" aria-labelledby="natpmp-title"><div className="router-heading"><div><h3 id="natpmp-title">{t("Alternativa · NAT-PMP")}</h3></div><span className="router-discovery-status" role="status">{!network?.lanAddress?t("Esperando red local"):busy?t("Comprobando…"):statusError?t("Sin comprobar"):detection||record?t("Router compatible"):t("Sin NAT-PMP detectado")}</span></div>
    {statusError&&<p className="network-error" role="alert">{translatedMessage(statusError)}</p>}
    <details className="natpmp-gateway"><summary>{t("Indicar otro router")}</summary><label htmlFor="natpmp-gateway">{t("IPv4 del router")}<input id="natpmp-gateway" inputMode="decimal" placeholder={t("Detección automática")} disabled={busy||!!record} value={gateway} onChange={event=>{setGateway(event.target.value);setDetection(null)}}/></label><button disabled={busy||!!record} onClick={()=>void detect()}>{t("Comprobar este router")}</button><p className="field-help">{t("Déjalo vacío para usar la puerta de enlace de la interfaz del servidor.")}</p></details>
    {error&&<p className="muted-note" role="status">{translatedMessage(error)} <button className="text-button" disabled={busy||!!record} onClick={()=>void detect()}>{t("Volver a comprobar")}</button></p>}
    {detection&&!record&&<dl className="data-list network-facts"><div><dt>{t("Router compatible")}</dt><dd>{detection.gateway}</dd></div><div><dt>{t("IP externa del router")}</dt><dd>{detection.externalIp}</dd></div></dl>}
    {detection?.publicAddress===false&&!record&&<p className="muted-note">{t("IP externa no pública: puede haber otro router o CGNAT.")}</p>}
    {record?<><p className="router-mapping-status" role="status"><span className={`state-dot ${record.phase==='active'&&record.expiresAt>Date.now()?'running':'starting'}`} aria-hidden="true"/>{record.expiresAt<=Date.now()?t("Concesión vencida; pendiente de revisión"):phases[record.phase]}</p><dl className="data-list network-facts"><div><dt>{t("Destino local")}</dt><dd>{record.localIp}:{record.internalPort}</dd></div><div><dt>Router</dt><dd>{record.gateway}</dd></div><div><dt>{t("Fin de concesión")}</dt><dd>{formatTime(record.expiresAt)}</dd></div></dl>{address&&<div className="network-address-row"><input readOnly aria-label={t("Dirección externa NAT-PMP")} value={address} onFocus={event=>event.currentTarget.select()}/><button onClick={()=>void copy()}>{copied?t("Copiada"):t("Copiar")}</button></div>}{record.lastError&&<p className="network-error">{translatedMessage(record.lastError)}</p>}<button disabled={busy} onClick={()=>void act(async()=>{await api.closeNatpmp(server.id);if(alive.current){setConsent(null);setCopied(false)}})}>{t("Retirar apertura NAT-PMP")}</button></>:<button className="primary-button" disabled={busy||!canOpen} onClick={()=>{if(detection&&network?.localIp)setConsent({gateway:detection.gateway,ip:network.localIp,port:network.port})}}>{t("Abrir con NAT-PMP")}</button>}

    {consent&&<Modal labelledBy="natpmp-confirm-title" busy={busy} onClose={()=>setConsent(null)}><h2 id="natpmp-confirm-title">{t("Abrir el puerto con NAT-PMP")}</h2><p>{t("El router")} {consent.gateway}  {t("dirigirá conexiones entrantes hacia")} {consent.ip}:{consent.port}{t(". Puede asignar un puerto externo diferente. Se gestionará la concesión TCP de este destino; se renueva mientras el servidor está activo y se intenta retirar al detenerlo. No cambia el firewall ni comprueba el acceso desde internet.")}</p>{statusError&&<p className="form-error" role="alert">{translatedMessage(statusError)}</p>}{error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}<div className="modal-actions"><button data-modal-close disabled={busy}>{t("Cancelar")}</button><button className="primary-button" disabled={busy||!!statusError||server.state!=='running'} onClick={()=>void act(async()=>{await api.openNatpmp(server.id,consent.gateway,consent.ip,consent.port);if(alive.current)setConsent(null)})}>{busy?t("Abriendo…"):t("Confirmar apertura temporal")}</button></div></Modal>}
  </section>
}
