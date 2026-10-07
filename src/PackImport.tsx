import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {listen} from '@tauri-apps/api/event'
import {api,type PackInspection,type Progress,type Server} from './api'
import {Modal} from './Modal'
import {SliderField} from './SliderField'
import {InstallProgress} from './InstallProgress'
import {useNavigationMotion} from './motion'

type Props={systemMemory:number;resumeId?:string|null;onClose:()=>void;onInstalled:(id:string)=>Promise<void>}
const size=(bytes:number)=>bytes>=1024*1024?`${(bytes/1024/1024).toFixed(1)} MB`:`${Math.ceil(bytes/1024)} KB`

export function PackImport({systemMemory,resumeId,onClose,onInstalled}:Props) {
  const [step,setStep]=useState(0)
  const [pack,setPack]=useState<PackInspection|null>(null)
  const [chosen,setChosen]=useState<Set<string>>(new Set())
  const [name,setName]=useState('')
  const [ram,setRam]=useState(4096)
  const [players,setPlayers]=useState(20)
  const [busy,setBusy]=useState(!!resumeId)
  const [error,setError]=useState<string|null>(null)
  const [consent,setConsent]=useState<boolean|null>(null)
  const [accepted,setAccepted]=useState(false)
  const [progress,setProgress]=useState<Progress|null>(null)
  const server=useRef<Server|null>(null)
  const stepBody=useRef<HTMLDivElement>(null)
  useNavigationMotion(stepBody,String(step))
  const locked=busy||!!server.current
  useEffect(()=>{let active=true;if(resumeId)void api.listServers().then(values=>{if(!active)return;const existing=values.find(value=>value.id===resumeId);if(existing){server.current=existing;setName(existing.name);setRam(existing.memoryMaxMb)}else{setError(t("No se encontró el servidor que quieres reintentar."))}}).catch(cause=>{if(active)setError(String(cause))}).finally(()=>{if(active)setBusy(false)});return()=>{active=false}},[resumeId])
  useEffect(()=>{let active=true;void api.eulaConsentRecorded().then(value=>{if(active)setConsent(value)}).catch(cause=>{if(active)setError(String(cause))});return()=>{active=false}},[])
  useEffect(()=>{let dispose:(()=>void)|undefined;let active=true
    void listen<Progress>('install-progress',event=>{if(active&&event.payload.kind==='pack_install'&&event.payload.serverId===server.current?.id)setProgress(event.payload)}).then(fn=>{if(active)dispose=fn;else fn()}).catch(cause=>{if(active)setError(String(cause))})
    return()=>{active=false;dispose?.()}
  },[])
  const pick=async()=>{setBusy(true);setError(null);try {const value=await api.pickModpack();if(value){setPack(value);setName(server.current?.name??value.name);setChosen(new Set(value.entries.filter(e=>e.selected).map(e=>e.path)));setStep(1)}}catch(cause){setError(String(cause))}finally{setBusy(false)}}
  const install=async()=>{
    if(!pack||!name.trim()||consent===null||(!consent&&!accepted))return
    setBusy(true);setError(null);setProgress(null)
    try {
      if(!server.current)server.current=await api.create({name:name.trim(),version:pack.minecraft,platform:pack.loader as 'vanilla'|'fabric'|'neoforge',memoryMinMb:1024,memoryMaxMb:ram,maxPlayers:players,port:25565,storageTargetGb:20,imagePath:null})
      if(!consent){await api.acceptEula(server.current.id);setConsent(true)}
      await api.installModpack(server.current.id,pack,[...chosen]);await onInstalled(server.current.id)
    }catch(cause){setError(String(cause))}finally{setBusy(false)}
  }
  const total=pack?.entries.filter(entry=>chosen.has(entry.path)).reduce((sum,entry)=>sum+entry.size,0)??0
  return <Modal labelledBy="pack-title" className="pack-dialog" busy={busy} onClose={onClose}>
    <header className="pack-heading"><div><span className="eyebrow">MODPACK</span><h2 id="pack-title">{pack?.name??t("Importar un modpack")}</h2></div><button data-modal-close disabled={busy} aria-label={t("Cerrar importación")}>×</button></header>
    <ol className="pack-steps" aria-label={t("Pasos de importación")}>{[t("Paquete"),t("Archivos"),t("Capacidad")].map((label,index)=><li key={label} aria-current={step===index?'step':undefined}><span>{index+1}</span>{translatedMessage(label)}</li>)}</ol>
    <div ref={stepBody}>{step===0&&<div className="pack-pick"><p>{t("Elige el archivo .mrpack descargado de Modrinth.")}</p><button className="primary-button" disabled={busy||!!resumeId&&!server.current} onClick={pick}>{busy?t("Revisando…"):t("Elegir modpack")}</button><p className="muted">{t("Se revisarán el motor, las descargas y los archivos compatibles con el servidor.")}</p></div>}
    {pack&&step>0&&<div className="pack-meta"><span>Minecraft <strong>{pack.minecraft}</strong></span><span>{pack.loader} <strong>{pack.loaderVersion??''}</strong></span><span>{size(total)}  {t("en archivos")}</span></div>}
    {pack&&step===1&&<>
      {!!pack.blockers.length&&<div className="form-error" role="alert">{pack.blockers.map(message=><p key={message}>{translatedMessage(message)}</p>)}</div>}
      <div className="pack-file-list" role="group" aria-label={t("Archivos del servidor")}>{pack.entries.map(entry=><label className="pack-file" key={entry.path}>
        <input type="checkbox" checked={chosen.has(entry.path)} disabled={busy||entry.protected||entry.server==='required'} onChange={event=>setChosen(current=>{const next=new Set(current);if(event.target.checked)next.add(entry.path);else next.delete(entry.path);return next})}/>
        <span><strong>{entry.path}</strong><small>{entry.protected?t("Se conserva el archivo del servidor"):entry.server==='optional'?t("Opcional"):entry.source==='server-overrides'?t("Configuración del servidor"):t("Obligatorio")}</small></span><span>{size(entry.size)}</span>
      </label>)}</div>
      {!!pack.excludedClientFiles&&<p className="muted">{pack.excludedClientFiles}  {t("archivos exclusivos del cliente excluidos.")}</p>}
      {pack.warnings.map(message=><p className="muted" key={message}>{translatedMessage(message)}</p>)}
    </>}
    {pack&&step===2&&<div className="pack-capacity"><label>{t("Nombre del servidor")}<input maxLength={80} value={name} disabled={locked} onChange={event=>setName(event.target.value)}/></label>
      <fieldset disabled={locked}><SliderField id="pack-ram" label={t("RAM máxima")} value={ram} min={1024} max={Math.min(65536,Math.max(4096,Math.floor(systemMemory/1024/1024)))} maxInput={65536} step={512} unit="MB" onChange={setRam}/>
      <SliderField id="pack-players" label={t("Jugadores máximos")} value={players} min={1} max={100} maxInput={1000} step={1} onChange={setPlayers}/></fieldset>
      {consent===false&&<label className="pack-consent"><input type="checkbox" checked={accepted} disabled={busy} onChange={event=>setAccepted(event.target.checked)}/><span>{t("He leído y acepto la")} <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">{t("EULA de Minecraft")}</a>{t(". Se guardará en este equipo.")}</span></label>}
      {consent===null&&<p role="status">{t("Comprobando la aceptación de la EULA…")}</p>}
    </div>}
    </div>
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    {progress&&<InstallProgress progress={progress} busy={busy} onRetry={()=>void install()} onDismiss={()=>setProgress(null)}/>}
    {step>0&&<footer className="modal-actions"><button disabled={busy||(step===1&&!!server.current)} onClick={()=>setStep(step-1)}>{t("Atrás")}</button>{step===1?<button className="primary-button" disabled={busy||!!pack?.blockers.length} onClick={()=>setStep(2)}>{t("Continuar →")}</button>:<button className="primary-button" disabled={busy||!name.trim()||ram<1024||ram>65536||players<1||players>1000||consent===null||(!consent&&!accepted)} onClick={install}>{busy?t("Preparando servidor…"):server.current?t("Reintentar importación"):t("Crear servidor")}</button>}</footer>}
  </Modal>
}
