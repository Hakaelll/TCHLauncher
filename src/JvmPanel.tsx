import {translatedMessage,t} from './locale.ts'
import { useEffect, useRef, useState } from 'react'
import { api, type JvmPreview, type JvmSettings, type Server } from './api'
import {useServerRead} from './useServerRead'
import {jvmPreviewKey} from './jvmPreview'

const defaults:JvmSettings={mode:'recommended',arguments:''}

export function JvmPanel({server,busy}:{server:Server;busy:boolean}) {
  const [draft,setDraft]=useState<JvmSettings>(defaults)
  const [saved,setSaved]=useState<JvmSettings>(defaults)
  const {data:profile,loading,error:readError,refresh}=useServerRead(server.id,api.jvmSettings)
  const loaded=profile!==null
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [previewError,setPreviewError]=useState<string|null>(null)
  const [previewRecord,setPreviewRecord]=useState<{key:string;value:JvmPreview}|null>(null)
  const previewKey=jvmPreviewKey(server,draft)
  const preview=previewRecord?.key===previewKey?previewRecord.value:null
  const [checking,setChecking]=useState(false)
  const [notice,setNotice]=useState('')
  const scope=useRef({active:true,saving:false})
  useEffect(()=>{
    const current={active:true,saving:false};scope.current=current
    return()=>{current.active=false}
  },[server.id])
  useEffect(()=>{if(profile){setDraft(profile);setSaved(profile);setNotice('');setError(null)}},[profile])
  useEffect(()=>{
    if(loading || !loaded || readError)return
    let active=true;setChecking(true);setPreviewRecord(null);setPreviewError(null)
    const timer=setTimeout(()=>{
      api.previewJvm(server.id,draft).then(value=>{if(active)setPreviewRecord({key:previewKey,value})}).catch(cause=>{if(active)setPreviewError(String(cause))}).finally(()=>{if(active)setChecking(false)})
    },250)
    return()=>{active=false;clearTimeout(timer)}
  },[server.id,previewKey,loading,loaded,readError])
  const stopped=['stopped','crashed','error'].includes(server.state)
  const disabled=loading || !loaded || !!readError || saving || busy || !stopped
  const dirty=draft.mode!==saved.mode || (draft.mode==='custom' && draft.arguments!==saved.arguments)
  const update=(value:JvmSettings)=>{if(jvmPreviewKey(server,value)!==previewKey){setPreviewRecord(null);setChecking(true)}setDraft(value);setError(null);setNotice('')}
  const save=async()=>{
    if(disabled||!dirty||checking||previewError||!preview||scope.current.saving)return
    const current=scope.current,value=draft.mode==='recommended'?{...defaults}:{...draft}
    current.saving=true;setSaving(true);setError(null);setNotice('')
    try{
      await api.saveJvm(server.id,value)
      if(current.active&&scope.current===current){setDraft(value);setSaved(value);setNotice(t("Perfil JVM guardado. Se aplicará en el próximo inicio."))}
    }catch(cause){if(current.active&&scope.current===current)setError(String(cause))}
    finally{current.saving=false;if(current.active&&scope.current===current)setSaving(false)}
  }
  return <details className="jvm-panel">
    <summary><span>{t("Argumentos JVM")}</span><span className="jvm-profile-label">{loading?t("Consultando"):readError?t("Sin actualizar"):!loaded?t("No disponible"):saved.mode==='recommended'?t("Recomendado"):t("Personalizado")}</span></summary>
    <div className="jvm-panel-body">
      <div className="jvm-read-heading"><p className="field-help" role="status">{loading?t("Consultando el perfil guardado…"):readError?t("No se pudo actualizar el perfil."):!loaded?t("Perfil pendiente de consulta."):dirty?t("Cambios sin guardar"):t("Perfil guardado para este servidor.")}</p><button disabled={loading||saving||busy||dirty} onClick={()=>void refresh(server.id)}>{loading?t("Consultando…"):t("Actualizar perfil")}</button></div>
      {readError&&<p className="form-error" role="alert">{translatedMessage(readError)}</p>}
      {loaded && <>
        <fieldset className="jvm-profiles" disabled={disabled}>
          <legend>{t("Perfil de arranque")}</legend>
          <label><input type="radio" name={`jvm-mode-${server.id}`} checked={draft.mode==='recommended'} onChange={()=>update({...draft,mode:'recommended'})}/><span><strong>{t("Recomendado")}</strong><small>{t("G1 con procesamiento paralelo de referencias.")}</small></span></label>
          <label><input type="radio" name={`jvm-mode-${server.id}`} checked={draft.mode==='custom'} onChange={()=>update({...draft,mode:'custom'})}/><span><strong>{t("Personalizado")}</strong><small>{t("Usa tus propias opciones de Java.")}</small></span></label>
        </fieldset>
        {draft.mode==='custom' && <div className="form-row"><label htmlFor="jvm-arguments">{t("Opciones de Java")}</label><textarea id="jvm-arguments" rows={5} value={draft.arguments} disabled={disabled} maxLength={8192} spellCheck={false} autoComplete="off" placeholder={'-XX:+UseG1GC\n-XX:+ParallelRefProcEnabled'} aria-describedby={`jvm-help${previewError?' jvm-validation':''}`} aria-invalid={previewError?true:undefined} onChange={event=>update({...draft,arguments:event.target.value})}/><p id="jvm-help" className="field-help">{t("Una opción por línea, sin comillas exteriores. Configura la RAM en Capacidad. Deja el campo vacío para usar los valores del runtime.")}</p></div>}
        <div className="jvm-preview"><span>{loading||readError?t("Vista previa pendiente de consultar"):checking||!preview?t("Preparando vista previa…"):t("Argumentos del próximo inicio")}</span>{!loading&&!readError&&preview && <code>{preview.arguments.map(argument=>/\s/.test(argument)?JSON.stringify(argument):argument).join(' ')}</code>}<p>{t("Usa los límites de RAM guardados.")}{preview && !preview.installed?' '+t("El archivo de arranque se añadirá al instalar el servidor."):''}</p></div>
        {previewError && <p id="jvm-validation" className="form-error" role="alert">{translatedMessage(previewError)}</p>}
        <div className="jvm-actions"><button className="primary-button" disabled={disabled || !dirty || checking || !!previewError || !preview} onClick={save}>{saving?t("Comprobando y guardando…"):draft.mode==='custom'?t("Comprobar y guardar perfil"):t("Guardar perfil")}</button><button disabled={disabled || (draft.mode==='recommended' && !draft.arguments)} onClick={()=>update(defaults)}>{t("Restaurar valores predeterminados")}</button></div>
        {draft.mode==='custom' && <p className="field-help">{t("Antes de guardar se comprobará el perfil con Java instalado, sin iniciar Minecraft. Los agentes Java ejecutan código: usa archivos de confianza.")}</p>}
        {notice && <p className="jvm-notice" role="status">{translatedMessage(notice)}</p>}
      </>}
      {error && <p className="form-error" role="alert">{translatedMessage(error)}</p>}
    </div>
  </details>
}
