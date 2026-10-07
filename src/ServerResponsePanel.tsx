import {translatedMessage} from './locale.ts'
import {t,getLanguage} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {api,type Server,type ServerResponse} from './api'

export function ServerResponsePanel({server}:{server:Server}){
  const [response,setResponse]=useState<ServerResponse|null>(null)
  const [error,setError]=useState<string|null>(null)
  const [checking,setChecking]=useState(false)
  const sequence=useRef(0),inFlight=useRef(false)
  const available=['running','starting'].includes(server.state)
  useEffect(()=>{
    ++sequence.current;inFlight.current=false;setChecking(false);setResponse(null);setError(null)
    return()=>{++sequence.current;inFlight.current=false}
  },[server.id,server.state])
  const check=async()=>{
    if(inFlight.current||!available)return
    inFlight.current=true;const request=++sequence.current;setChecking(true);setError(null);setResponse(null)
    try{const value=await api.serverResponse(server.id);if(request===sequence.current)setResponse(value)}
    catch(cause){if(request===sequence.current)setError(cause instanceof Error?cause.message:String(cause))}
    finally{if(request===sequence.current){inFlight.current=false;setChecking(false)}}
  }
  return <div className="server-response" aria-busy={checking}>
    <div className="section-heading"><div><h3>{t("Respuesta de Minecraft")}</h3><p>{t("Latencia y respuesta del servidor en este equipo.")}</p></div><button disabled={!available||checking} onClick={()=>void check()}>{checking?t("Comprobando…"):t("Comprobar respuesta")}</button></div>
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    {response&&<><dl className="data-list"><div><dt>{t("Latencia local")}</dt><dd>{response.latencyMs.toLocaleString(getLanguage()==='en'?'en-US':'es-ES',{maximumFractionDigits:1})} ms</dd></div><div><dt>{t("Versión anunciada")}</dt><dd>{response.version??t("No publicada")}</dd></div><div><dt>{t("Jugadores publicados")}</dt><dd>{response.online??'—'} / {response.maximum??'—'}</dd></div></dl><p className="field-help" role="status">{t("Respuesta recibida de")} {response.address}  {t("a las")} {new Date(response.sampledAt).toLocaleTimeString(getLanguage()==='en'?'en-US':'es-ES')}.</p></>}
    <p className="field-help">{!available?t("Inicia el servidor para comprobarlo. "):''}{t("Es una consulta local, no el ping de tus amigos ni una comprobación de acceso desde internet.")}</p>
  </div>
}
