import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import { useEffect, useRef, useState } from 'react'
import { Modal } from './Modal'
import { api, type RuntimeInfo, type Server } from './api'
import {useServerRead} from './useServerRead'
import {runtimeStatus} from './runtimeStatus'

export function RuntimePanel({server,busy,onPrepare,onChanged}: {server:Server;busy:boolean;onPrepare:(reinstall:boolean)=>void;onChanged:()=>Promise<void>}) {
  const {data:info,loading,error:readError,refresh}=useServerRead(server.id,api.runtimeInfo)
  const [candidate,setCandidate]=useState<RuntimeInfo|null>(null)
  const [choosing,setChoosing]=useState(false)
  const [saving,setSaving]=useState(false)
  const [actionError,setActionError]=useState<string|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const cancelRef=useRef<HTMLButtonElement>(null)
  const scope=useRef({id:server.id,active:true,working:false})
  const observed=useRef({id:server.id,path:server.javaPath,state:server.state,version:server.version})
  useEffect(()=>{
    const current={id:server.id,active:true,working:false};scope.current=current
    setCandidate(null);setActionError(null);setNotice(null);setChoosing(false);setSaving(false)
    return()=>{current.active=false}
  },[server.id])
  useEffect(()=>{
    const previous=observed.current
    observed.current={id:server.id,path:server.javaPath,state:server.state,version:server.version}
    if(previous.id===server.id&&(previous.path!==server.javaPath||previous.state!==server.state||previous.version!==server.version))void refresh(server.id)
  },[server.id,server.javaPath,server.state,server.version,refresh])
  const stopped=['stopped','crashed','error'].includes(server.state)
  const blocked=busy||!stopped||loading||choosing||saving
  const currentScope=(value:typeof scope.current)=>value.active&&scope.current===value
  const choose=async()=>{
    if(blocked||scope.current.working)return
    const current=scope.current;current.working=true
    setChoosing(true);setActionError(null);setNotice(null);setCandidate(null)
    try {const value=await api.pickCustomRuntime(current.id);if(currentScope(current))setCandidate(value)}
    catch(cause){if(currentScope(current))setActionError(String(cause))}
    finally{current.working=false;if(currentScope(current))setChoosing(false)}
  }
  const apply=async()=>{
    if(!candidate?.path||blocked||scope.current.working)return
    const current=scope.current;current.working=true
    setSaving(true);setActionError(null);setNotice(null)
    try {
      await api.setCustomRuntime(current.id,candidate.path)
      if(!currentScope(current))return
      setCandidate(null);setNotice(t("Java seleccionado. Se usará en el próximo inicio."))
      const results=await Promise.allSettled([refresh(current.id),onChanged()])
      if(currentScope(current)&&results[1].status==='rejected')setActionError(t("Java se guardó, pero no se pudieron actualizar los datos del servidor: {0}", String(results[1].reason)))
    }catch(cause){if(currentScope(current))setActionError(String(cause))}
    finally{current.working=false;if(currentScope(current))setSaving(false)}
  }
  const status=runtimeStatus(info,loading,readError)
  return <section className="runtime-panel" aria-labelledby="runtime-heading" aria-busy={loading||choosing||saving}>
    <div className="runtime-heading">
      <div className="runtime-mark" aria-hidden="true"><svg viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M7 16h16v9H7zM23 17h4v6h-4M5 28h22M12 4v4h3v4M19 2v4h3v5"/></svg></div>
      <div className="runtime-identity"><h3 id="runtime-heading">Java · {info?.distribution??'Eclipse Temurin'}</h3><p>{translatedMessage(status.description)}</p></div>
      <span className={`runtime-state ${status.kind}`} role="status">{translatedMessage(status.label)}</span>
    </div>
    {info?.requiredMajor&&<p className="runtime-requirement">Minecraft {server.version}<span aria-hidden="true">·</span>{t("Requiere Java")} {info.requiredMajor}{readError&&<span>{t("Última lectura")}</span>}</p>}
    {(readError||info?.error)&&<p className="form-error" role="alert">{readError||info?.error}</p>}
    {actionError&&!candidate&&<p className="form-error" role="alert">{translatedMessage(actionError)}</p>}
    {notice&&<p className="runtime-notice" role="status">{translatedMessage(notice)}</p>}
    <div className="runtime-actions"><button className="primary-button" disabled={blocked} onClick={()=>onPrepare(!readError&&info?.status==='ready'&&info.managed===true)}>{!readError&&info?.status==='ready'?info.managed?t("Reinstalar Temurin"):t("Usar Temurin gestionado"):t("Instalar Temurin")}</button><button disabled={busy||loading||choosing||saving} onClick={()=>void refresh(server.id)}>{loading?t("Comprobando…"):t("Comprobar de nuevo")}</button></div>
    <div className="runtime-details">
      {info?.path&&<details className="runtime-location"><summary>{t("Ubicación del runtime")}{readError?' · última lectura':''}</summary><div className="runtime-path"><code>{info.path}</code></div></details>}
      <details className="runtime-advanced"><summary>{t("Java personalizado")}</summary><p className="field-help">{t("Elige un runtime instalado. Comprobaremos su versión antes de cambiarlo.")}</p><button disabled={blocked} onClick={choose}>{choosing?t("Comprobando Java…"):t("Elegir ejecutable Java")}</button></details>
    </div>
    {candidate&&<Modal labelledBy="custom-java-title" describedBy="custom-java-description" busy={saving} initialFocusRef={cancelRef} onClose={()=>setCandidate(null)}><h2 id="custom-java-title">{t("Usar Java")} {candidate.version}</h2><p id="custom-java-description">{t("Compatible con este servidor. El cambio solo afecta a su configuración en TCHLauncher.")}</p><div className="runtime-path"><span>{t("Ejecutable")}</span><code>{candidate.path}</code></div>{actionError&&<p className="form-error" role="alert">{translatedMessage(actionError)}</p>}<div className="world-confirm-actions"><button data-modal-close ref={cancelRef} disabled={saving} onClick={()=>setCandidate(null)}>{t("Cancelar")}</button><button className="primary-button" disabled={blocked} onClick={apply}>{saving?t("Guardando…"):t("Usar este Java")}</button></div></Modal>}
    <p className="runtime-footnote">{t("El runtime gestionado se guarda y verifica en la biblioteca del launcher.")}</p>
  </section>
}
