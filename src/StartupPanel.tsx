import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type Server} from './api'
import {useServerRead} from './useServerRead'

const OS_SCOPE='startup-os'
const SERVERS_SCOPE='startup-servers'
export function StartupPanel({servers,onBusy}:{servers:Server[];onBusy:(busy:boolean)=>void}) {
  const osRead=useServerRead(OS_SCOPE,api.osAutostart)
  const serverRead=useServerRead(SERVERS_SCOPE,api.startupServers)
  const os=osRead.data
  const options=serverRead.data??[]
  const latestOptions=useRef(serverRead.data);latestOptions.current=serverRead.data
  const [working,setWorking]=useState(false)
  const [actionError,setActionError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const [eventError,setEventError]=useState<string|null>(null)
  const owner=useRef({active:true,working:false})
  const busyCallback=useRef(onBusy);busyCallback.current=onBusy
  const catalogKey=JSON.stringify(servers.map(server=>[server.id,server.name]))
  const observedCatalog=useRef(catalogKey)
  useEffect(()=>{
    const current={active:true,working:false};owner.current=current
    const subscription=listen('startup-result',()=>{if(current.active)void serverRead.refresh(SERVERS_SCOPE)})
    void subscription.catch(()=>{if(current.active)setEventError('No se pudieron recibir resultados de inicio. Usa Actualizar para consultarlos.')})
    return()=>{current.active=false;if(current.working)busyCallback.current(false);void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[serverRead.refresh])
  useEffect(()=>{
    if(observedCatalog.current===catalogKey)return
    observedCatalog.current=catalogKey;void serverRead.refresh(SERVERS_SCOPE)
  },[catalogKey,serverRead.refresh])
  const currentOwner=(current:typeof owner.current)=>current.active&&owner.current===current
  const mutate=async(operation:()=>Promise<void>,message:string)=>{
    const current=owner.current;if(!current.active||current.working)return
    current.working=true;setWorking(true);busyCallback.current(true);setActionError(null);setNotice(null)
    try{await operation();if(currentOwner(current))setNotice(message)}
    catch(cause){if(currentOwner(current))setActionError(String(cause))}
    finally{current.working=false;if(currentOwner(current)){setWorking(false);busyCallback.current(false)}}
  }
  const changeOs=(enabled:boolean)=>void mutate(async()=>{
    const current=owner.current
    const value=await api.setOsAutostart(enabled)
    if(currentOwner(current))osRead.acknowledge(OS_SCOPE,value)
  },enabled?'Inicio con el sistema guardado.':'Inicio con el sistema desactivado.')
  const changeServer=(id:string,enabled:boolean)=>{
    const previous=serverRead.data
    if(!previous||serverRead.error||serverRead.loading||!previous.some(option=>option.id===id))return
    void mutate(async()=>{
      const current=owner.current
      await api.setStartupServer(id,enabled)
      if(!currentOwner(current))return
      serverRead.acknowledge(SERVERS_SCOPE,(latestOptions.current??previous).map(option=>option.id===id?{...option,enabled}:option))
      setNotice('Preferencia guardada. Se aplicará al abrir TCHLauncher.')
      await serverRead.refresh(SERVERS_SCOPE)
    },'Preferencia guardada. Se aplicará al abrir TCHLauncher.')
  }
  const osBlocked=working||osRead.loading||!!osRead.error||!os||(!os.supported&&!os.enabled)
  return <div className="startup-panel" aria-busy={working||undefined}>
    <section><div className="startup-section-heading"><h3>Aplicación</h3><button disabled={working||osRead.loading} onClick={()=>void osRead.refresh(OS_SCOPE)}>{osRead.loading?'Consultando…':'Comprobar de nuevo'}</button></div>
      <label className="schedule-repeat"><input type="checkbox" checked={os?.enabled??false} disabled={osBlocked} onChange={event=>changeOs(event.target.checked)}/>Abrir TCHLauncher al iniciar sesión</label>
      {!os&&osRead.loading&&<p className="field-help" role="status">Consultando el inicio con el sistema…</p>}
      {osRead.error&&<p className="form-error" role="alert">No se pudo consultar el inicio con el sistema: {osRead.error}{os?' Se muestra la última preferencia conocida.':''}</p>}
      <p className="field-help">Se abrirá en segundo plano. Puedes mostrar la ventana desde el icono de la bandeja.</p>{os?.note&&<p className="inline-note">{os.note}</p>}{os?.supported&&os.enabled&&!os.currentPath&&<button disabled={osBlocked} onClick={()=>changeOs(true)}>Actualizar ubicación</button>}</section>
    <section><div className="startup-section-heading"><h3>Servidores</h3><button disabled={working||serverRead.loading} onClick={()=>void serverRead.refresh(SERVERS_SCOPE)}>{serverRead.loading?'Consultando…':'Actualizar'}</button></div>
      <p className="field-help">Los seleccionados se iniciarán cada vez que abras TCHLauncher. Deben estar instalados, tener Java compatible y la EULA aceptada.</p>
      {serverRead.error&&<p className="form-error" role="alert">No se pudieron consultar las preferencias de servidores: {serverRead.error}{serverRead.data?' Se muestran los últimos datos conocidos.':''}</p>}
      {!serverRead.data?<p className="muted" role="status">{serverRead.error?'Preferencias no disponibles. Usa Actualizar para reintentarlo.':'Cargando preferencias…'}</p>:!options.length?<p className="empty-inline">Crea o importa un servidor para seleccionarlo aquí.</p>:<div className="startup-server-list">{options.map(option=>{const server=servers.find(server=>server.id===option.id);return <div className="startup-server-row" key={option.id}>
        <span className="startup-server-icon" aria-hidden="true">{server?.imageData?<img src={server.imageData} alt=""/>:<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m12 3 9 5v10l-9 5-9-5V8l9-5Zm0 10v10M3 8l9 5 9-5"/></svg>}</span><div><label><input type="checkbox" checked={option.enabled} disabled={working||serverRead.loading||!!serverRead.error||!server} onChange={event=>changeServer(option.id,event.target.checked)}/>{option.name}</label>{option.lastResult&&<p className={option.lastResult.startsWith('Error:')?'form-error':'field-help'}>{option.lastResult}</p>}</div><span className="startup-option-state">{option.enabled?'Seleccionado':'Manual'}</span>
      </div>})}</div>}
      {eventError&&<p className="inline-note">{eventError}</p>}
    </section>{working&&<p className="field-help" role="status">Aplicando preferencia…</p>}{notice&&<p className="runtime-notice" role="status">{notice}</p>}{actionError&&<p className="form-error" role="alert">{actionError}</p>}
  </div>
}
