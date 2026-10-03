import {useEffect,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type OsAutostart,type StartupServer,type Server} from './api'

export function StartupPanel({servers,onBusy}:{servers:Server[];onBusy:(busy:boolean)=>void}) {
  const [os,setOs]=useState<OsAutostart|null>(null)
  const [options,setOptions]=useState<StartupServer[]>([])
  const [loaded,setLoaded]=useState(false)
  const [working,setWorking]=useState(false)
  const [error,setError]=useState<string|null>(null)
  useEffect(()=>{
    let active=true
    void Promise.all([api.osAutostart(),api.startupServers()]).then(([os,items])=>{if(active){setOs(os);setOptions(items);setLoaded(true)}}).catch(cause=>{if(active)setError(String(cause))})
    const subscription=listen('startup-result',()=>{void api.startupServers().then(items=>{if(active)setOptions(items)}).catch(cause=>{if(active)setError(String(cause))})})
    return()=>{active=false;void subscription.then(unlisten=>unlisten()).catch(()=>{})}
  },[])
  const mutate=async(operation:()=>Promise<unknown>)=>{setWorking(true);onBusy(true);setError(null);try{await operation();const [os,items]=await Promise.all([api.osAutostart(),api.startupServers()]);setOs(os);setOptions(items)}catch(cause){setError(String(cause))}finally{setWorking(false);onBusy(false)}}
  return <div className="startup-panel" aria-busy={working||undefined}>
    <section><h3>Aplicación</h3><label className="schedule-repeat"><input type="checkbox" checked={os?.enabled??false} disabled={!loaded||working||(!os?.supported&&!os?.enabled)} onChange={event=>void mutate(()=>api.setOsAutostart(event.target.checked))}/>Abrir TCHLauncher al iniciar sesión</label><p className="field-help">Se abrirá en segundo plano. Puedes mostrar la ventana desde el icono de la bandeja.</p>{os?.note&&<p className="inline-note">{os.note}</p>}{os?.supported&&os.enabled&&!os.currentPath&&<button disabled={working} onClick={()=>void mutate(()=>api.setOsAutostart(true))}>Actualizar ubicación</button>}</section>
    <section><h3>Servidores</h3><p className="field-help">Los seleccionados se iniciarán cada vez que abras TCHLauncher. Deben estar instalados, tener Java compatible y la EULA aceptada.</p>
      {!loaded?<p className="muted">Cargando preferencias…</p>:!options.length?<p className="empty-inline">Crea o importa un servidor para seleccionarlo aquí.</p>:<div className="startup-server-list">{options.map(option=>{const server=servers.find(server=>server.id===option.id);return <div className="startup-server-row" key={option.id}>
        {server?.imageData&&<img src={server.imageData} alt=""/>}<div><label><input type="checkbox" checked={option.enabled} disabled={working} onChange={event=>void mutate(()=>api.setStartupServer(option.id,event.target.checked))}/>{option.name}</label>{option.lastResult&&<p className={option.lastResult.startsWith('Error:')?'form-error':'field-help'}>{option.lastResult}</p>}</div>
      </div>})}</div>}
    </section>{working&&<p className="field-help" role="status">Aplicando preferencia…</p>}{error&&<p className="form-error" role="alert">{error}</p>}
  </div>
}
