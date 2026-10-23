import {useEffect,useId,useRef,useState,type FormEvent} from 'react'
import {api,type GrantOutcome,type PlayerIdentity,type Server} from './api'
import {errorText} from './format'
import {t,translatedMessage} from './locale'

export function OperatorGrant({server,player,alreadyOperator,disabled,onRefresh}:{server:Server;player:PlayerIdentity;alreadyOperator:boolean;disabled:boolean;onRefresh:()=>Promise<void>}){
  const id=useId()
  const [level,setLevel]=useState(2)
  const [bypass,setBypass]=useState(false)
  const [saving,setSaving]=useState(false)
  const [saved,setSaved]=useState<GrantOutcome|null>(null)
  const [error,setError]=useState<string|null>(null)
  const scope=useRef({active:true,saving:false})
  useEffect(()=>{const current={active:true,saving:false};scope.current=current;return()=>{current.active=false}},[server.id,player.uuid])
  const stopped=['stopped','crashed','error'].includes(server.state)
  const blocked=disabled||saving||!stopped||!player.uuid||alreadyOperator||!!saved
  const submit=async(event:FormEvent)=>{
    event.preventDefault()
    if(blocked||!player.uuid||scope.current.saving)return
    const current=scope.current;current.saving=true;setSaving(true);setError(null)
    try{
      const outcome=await api.grantOperator(server.id,{player:{name:player.name,uuid:player.uuid},level,bypassesPlayerLimit:bypass})
      if(!current.active)return
      setSaved(outcome)
      try{await onRefresh()}catch{if(current.active)setError(t('Los permisos se guardaron, pero no se pudo actualizar la lista.'))}
    }catch(cause){if(current.active)setError(errorText(cause))}
    finally{current.saving=false;if(current.active)setSaving(false)}
  }
  if(alreadyOperator&&!saved)return <p className="known-operator-status">{t('Operador')}</p>
  return <details className="operator-grant">
    <summary>{saved?t('Permisos guardados'):t('Dar OP')}<span>{saved?t('Para el próximo inicio'):t('Revisar permisos')}</span></summary>
    {saved?<div className="operator-grant-result"><p role="status">{saved.added?t('Se concedió OP al UUID {0}.',player.uuid):t('Ese UUID ya es operador. No se cambiaron sus permisos.')}</p>{saved.previousFile&&<p>{t('Copia anterior: {0}',saved.previousFile)}</p>}</div>:<form onSubmit={event=>void submit(event)} aria-busy={saving}>
      <div className="operator-grant-identity"><strong>{player.name}</strong><code>{player.uuid??t('UUID no disponible')}</code></div>
      <fieldset disabled={!!blocked}>
        <label className="form-row" htmlFor={id}><span>{t('Nivel de operador')}</span><select id={id} value={level} onChange={event=>setLevel(Number(event.target.value))}>{[0,1,2,3,4].map(value=><option key={value} value={value}>{t('Nivel')} {value}</option>)}</select></label>
        <label className="check-row"><input type="checkbox" checked={bypass} onChange={event=>setBypass(event.target.checked)}/>{t('Permitir entrar cuando el servidor esté lleno')}</label>
      </fieldset>
      <p className="field-help">{t('OP concede permisos de administración a esta cuenta. Se aplican en el próximo inicio.')}</p>
      {!stopped&&<p className="field-help">{t('Detén el servidor para conceder OP por UUID.')}</p>}
      <button className="primary-button" disabled={!!blocked}>{saving?t('Guardando…'):t('Confirmar permisos de operador')}</button>
    </form>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
  </details>
}
