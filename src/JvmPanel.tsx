import { useEffect, useRef, useState } from 'react'
import { api, type JvmPreview, type JvmSettings, type Server } from './api'

const defaults:JvmSettings={mode:'recommended',arguments:''}

export function JvmPanel({server,busy}:{server:Server;busy:boolean}) {
  const [draft,setDraft]=useState<JvmSettings>(defaults)
  const [saved,setSaved]=useState<JvmSettings>(defaults)
  const [loading,setLoading]=useState(true)
  const [loaded,setLoaded]=useState(false)
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [previewError,setPreviewError]=useState<string|null>(null)
  const [preview,setPreview]=useState<JvmPreview|null>(null)
  const [checking,setChecking]=useState(false)
  const [notice,setNotice]=useState('')
  const [revision,setRevision]=useState(0)
  const mounted=useRef(true)
  useEffect(()=>{mounted.current=true;return()=>{mounted.current=false}},[])
  useEffect(()=>{
    let active=true;setLoading(true);setLoaded(false);setError(null)
    api.jvmSettings(server.id).then(value=>{if(active){setDraft(value);setSaved(value);setLoaded(true)}}).catch(cause=>{if(active)setError(String(cause))}).finally(()=>{if(active)setLoading(false)})
    return()=>{active=false}
  },[server.id,revision])
  useEffect(()=>{
    if(loading || !loaded)return
    let active=true;setChecking(true);setPreview(null);setPreviewError(null)
    const timer=setTimeout(()=>{
      api.previewJvm(server.id,draft).then(value=>{if(active)setPreview(value)}).catch(cause=>{if(active)setPreviewError(String(cause))}).finally(()=>{if(active)setChecking(false)})
    },250)
    return()=>{active=false;clearTimeout(timer)}
  },[server.id,server.memoryMinMb,server.memoryMaxMb,server.jarPath,loading,loaded,draft])
  const stopped=['stopped','crashed','error'].includes(server.state)
  const disabled=loading || !loaded || saving || busy || !stopped
  const dirty=draft.mode!==saved.mode || (draft.mode==='custom' && draft.arguments!==saved.arguments)
  const update=(value:JvmSettings)=>{setDraft(value);setError(null);setNotice('')}
  const save=async()=>{
    setSaving(true);setError(null);setNotice('')
    try{await api.saveJvm(server.id,draft);if(mounted.current){const value=draft.mode==='recommended'?defaults:draft;setDraft(value);setSaved(value);setNotice('Perfil JVM guardado. Se aplicará en el próximo inicio.')}}
    catch(cause){if(mounted.current)setError(String(cause))}
    finally{if(mounted.current)setSaving(false)}
  }
  return <details className="jvm-panel">
    <summary><span>Argumentos JVM</span><span className="jvm-profile-label">{saved.mode==='recommended'?'Recomendado':'Personalizado'}</span></summary>
    <div className="jvm-panel-body">
      {loading?<p role="status">Cargando perfil…</p>:loaded && <>
        <fieldset className="jvm-profiles" disabled={disabled}>
          <legend>Perfil de arranque</legend>
          <label><input type="radio" name={`jvm-mode-${server.id}`} checked={draft.mode==='recommended'} onChange={()=>update({...draft,mode:'recommended'})}/><span><strong>Recomendado</strong><small>G1 con procesamiento paralelo de referencias.</small></span></label>
          <label><input type="radio" name={`jvm-mode-${server.id}`} checked={draft.mode==='custom'} onChange={()=>update({...draft,mode:'custom'})}/><span><strong>Personalizado</strong><small>Usa tus propias opciones de Java.</small></span></label>
        </fieldset>
        {draft.mode==='custom' && <div className="form-row"><label htmlFor="jvm-arguments">Opciones de Java</label><textarea id="jvm-arguments" rows={5} value={draft.arguments} disabled={disabled} maxLength={8192} spellCheck={false} autoComplete="off" placeholder={'-XX:+UseG1GC\n-XX:+ParallelRefProcEnabled'} aria-describedby={`jvm-help${previewError?' jvm-validation':''}`} aria-invalid={previewError?true:undefined} onChange={event=>update({...draft,arguments:event.target.value})}/><p id="jvm-help" className="field-help">Una opción por línea, sin comillas exteriores. Configura la RAM en Capacidad. Deja el campo vacío para usar los valores del runtime.</p></div>}
        <div className="jvm-preview"><span>{checking?'Preparando vista previa…':'Argumentos del próximo inicio'}</span>{preview && <code>{preview.arguments.map(argument=>/\s/.test(argument)?JSON.stringify(argument):argument).join(' ')}</code>}<p>Usa los límites de RAM guardados.{preview && !preview.installed?' El archivo de arranque se añadirá al instalar el servidor.':''}</p></div>
        {previewError && <p id="jvm-validation" className="form-error" role="alert">{previewError}</p>}
        <div className="jvm-actions"><button className="primary-button" disabled={disabled || !dirty || checking || !!previewError} onClick={save}>{saving?'Comprobando y guardando…':draft.mode==='custom'?'Comprobar y guardar perfil':'Guardar perfil'}</button><button disabled={disabled || (draft.mode==='recommended' && !draft.arguments)} onClick={()=>update(defaults)}>Restaurar valores predeterminados</button></div>
        {draft.mode==='custom' && <p className="field-help">Antes de guardar se comprobará el perfil con Java instalado, sin iniciar Minecraft. Los agentes Java ejecutan código: usa archivos de confianza.</p>}
        {notice && <p className="jvm-notice" role="status">{notice}</p>}
      </>}
      {error && <p className="form-error" role="alert">{error}{!loading && !loaded && <button onClick={()=>setRevision(value=>value+1)}>Reintentar</button>}</p>}
    </div>
  </details>
}
