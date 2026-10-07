import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import {useEffect,useId,useRef,useState,type FormEvent} from 'react'
import {api,type Operator,type Server} from './api'
import {operatorDraft,reconcileOperator,operatorDirty,operatorConflict} from './operatorDraft'

export function OperatorSettings({server,operator,disabled,onRefresh}:{server:Server;operator:Operator;disabled:boolean;onRefresh:()=>Promise<void>}){
  const id=useId()
  const identity=`${server.id}:${operator.uuid??operator.name}`
  const [state,setState]=useState(()=>operatorDraft(identity,operator))
  const {level,bypassesPlayerLimit:bypass}=state.draft
  const scope=useRef({active:true,saving:false})
  useEffect(()=>{const current={active:true,saving:false};scope.current=current;return()=>{current.active=false}},[identity])
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [saved,setSaved]=useState<string|null>(null)
  useEffect(()=>{if(!scope.current.saving)setState(current=>reconcileOperator(current,identity,operator))},[identity,operator])
  const conflict=operatorConflict(state,identity,operator)
  const stopped=['stopped','crashed','error'].includes(server.state)
  const blocked=disabled||saving||!stopped||!operator.uuid||conflict
  const changed=operatorDirty(state)
  const submit=async(event:FormEvent)=>{
    event.preventDefault()
    if(blocked||!changed||!operator.uuid||scope.current.saving)return
    const current=scope.current;current.saving=true
    const acknowledged={...state.draft}
    setError(null);setSaved(null);setSaving(true)
    try{
      const copy=await api.saveOperator(server.id,{uuid:operator.uuid,level,bypassesPlayerLimit:bypass,expectedLevel:state.baseline.level,expectedBypassesPlayerLimit:state.baseline.bypassesPlayerLimit})
      if(!current.active||scope.current!==current)return
      setState(operatorDraft(identity,acknowledged))
      setSaved(t("Permisos guardados. Copia anterior: {0}", copy))
      try{await onRefresh()}catch{if(current.active&&scope.current===current)setError(t("Los permisos se guardaron, pero no se pudo actualizar la lista."))}
    }catch(cause){if(current.active&&scope.current===current)setError(cause instanceof Error?cause.message:String(cause))}
    finally{current.saving=false;if(current.active&&scope.current===current)setSaving(false)}
  }
  return <details className="operator-settings"><summary>{t("Permisos de")} {operator.name}</summary>
    <form onSubmit={event=>void submit(event)} aria-busy={saving}>
      <label htmlFor={`${id}-level`}>{t("Nivel de operador")}</label>
      <select id={`${id}-level`} value={level} disabled={blocked} onChange={event=>{setState(current=>({...current,draft:{...current.draft,level:Number(event.target.value)}}));setSaved(null);setError(null)}}>
        {[0,1,2,3,4].map(value=><option key={value} value={value}>{t("Nivel")} {value}</option>)}
      </select>
      <label className="check-row"><input type="checkbox" checked={bypass} disabled={blocked} onChange={event=>{setState(current=>({...current,draft:{...current.draft,bypassesPlayerLimit:event.target.checked}}));setSaved(null);setError(null)}}/>{t("Permitir entrar cuando el servidor esté lleno")}</label>
      <p className="field-help">{t("Se aplican en el próximo inicio. Los plugins pueden gestionar permisos adicionales.")}</p>
      {!stopped&&<p className="field-help">{t("Detén el servidor para cambiar estos permisos.")}</p>}
      {!operator.uuid&&<p className="form-error">{t("Este operador no tiene un UUID válido guardado.")}</p>}
      {conflict&&<div className="operator-conflict"><p className="form-error" role="alert">{t("Los permisos guardados cambiaron mientras editabas. Tus cambios siguen aquí; revisa los actuales antes de guardar.")}</p><p className="field-help">{t("Actuales: nivel")} {operator.level}{operator.bypassesPlayerLimit?' · puede entrar con el servidor lleno':''}.</p><button type="button" disabled={saving||disabled} onClick={()=>{setState(operatorDraft(identity,operator));setSaved(null);setError(null)}}>{t("Cargar permisos actuales")}</button></div>}
      {changed&&!conflict&&<p className="field-help">{t("Cambios sin guardar.")}</p>}
      <button disabled={!!blocked||!changed}>{saving?t("Guardando…"):t("Guardar permisos")}</button>
      {saved&&<p className="field-help" role="status">{saved}</p>}
      {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
    </form>
  </details>
}
