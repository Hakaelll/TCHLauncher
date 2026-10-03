import {useEffect,useId,useState,type FormEvent} from 'react'
import {api,type Operator,type Server} from './api'

export function OperatorSettings({server,operator,disabled,onRefresh}:{server:Server;operator:Operator;disabled:boolean;onRefresh:()=>Promise<void>}){
  const id=useId()
  const [level,setLevel]=useState(operator.level)
  const [bypass,setBypass]=useState(operator.bypassesPlayerLimit)
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [saved,setSaved]=useState<string|null>(null)
  useEffect(()=>{setLevel(operator.level);setBypass(operator.bypassesPlayerLimit)},[operator.level,operator.bypassesPlayerLimit])
  const stopped=['stopped','crashed','error'].includes(server.state)
  const blocked=disabled||saving||!stopped||!operator.uuid
  const changed=level!==operator.level||bypass!==operator.bypassesPlayerLimit
  const submit=async(event:FormEvent)=>{
    event.preventDefault()
    if(blocked||!changed||!operator.uuid)return
    setError(null);setSaved(null);setSaving(true)
    try{
      const copy=await api.saveOperator(server.id,{uuid:operator.uuid,level,bypassesPlayerLimit:bypass,expectedLevel:operator.level,expectedBypassesPlayerLimit:operator.bypassesPlayerLimit})
      setSaved(`Permisos guardados. Copia anterior: ${copy}`)
      try{await onRefresh()}catch{setError('Los permisos se guardaron, pero no se pudo actualizar la lista.')}
    }catch(cause){setError(cause instanceof Error?cause.message:String(cause))}
    finally{setSaving(false)}
  }
  return <details className="operator-settings"><summary>Permisos de {operator.name}</summary>
    <form onSubmit={event=>void submit(event)} aria-busy={saving}>
      <label htmlFor={`${id}-level`}>Nivel de operador</label>
      <select id={`${id}-level`} value={level} disabled={blocked} onChange={event=>setLevel(Number(event.target.value))}>
        {[0,1,2,3,4].map(value=><option key={value} value={value}>Nivel {value}</option>)}
      </select>
      <label className="check-row"><input type="checkbox" checked={bypass} disabled={blocked} onChange={event=>setBypass(event.target.checked)}/>Permitir entrar cuando el servidor esté lleno</label>
      <p className="field-help">Se aplican en el próximo inicio. Los plugins pueden gestionar permisos adicionales.</p>
      {!stopped&&<p className="field-help">Detén el servidor para cambiar estos permisos.</p>}
      {!operator.uuid&&<p className="form-error">Este operador no tiene un UUID válido guardado.</p>}
      <button disabled={!!blocked||!changed}>{saving?'Guardando…':'Guardar permisos'}</button>
      {saved&&<p className="field-help" role="status">{saved}</p>}
      {error&&<p className="form-error" role="alert">{error}</p>}
    </form>
  </details>
}
