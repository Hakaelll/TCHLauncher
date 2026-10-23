import {errorText} from './format.ts'
import {useEffect,useRef,useState} from 'react'
import {api,type PlayerIdentity,type WhitelistImport,type WhitelistSelection,type WhitelistOutcome} from './api'
import {t,translatedMessage} from './locale'
import {playerWindow} from './playerWindow'
import {PlayerHead} from './PlayerHead'
import {identityKey} from './playerIdentity'

export function WhitelistImportReview({whitelisted,running,disabled,onImport,onRefresh}:{whitelisted:PlayerIdentity[];running:boolean;disabled:boolean;onImport:(selection:WhitelistSelection)=>Promise<WhitelistOutcome|null>;onRefresh:()=>Promise<void>}){
  const [file,setFile]=useState<WhitelistImport|null>(null)
  const [query,setQuery]=useState(''),[page,setPage]=useState(0)
  const [phase,setPhase]=useState<'idle'|'picking'|'sending'>('idle')
  const [error,setError]=useState<string|null>(null),[notice,setNotice]=useState<string|null>(null)
  const scope=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};scope.current=current;return()=>{current.active=false}},[])
  const existing=new Set(whitelisted.flatMap(player=>player.uuid?[identityKey(player.uuid)]:[]))
  const entries:PlayerIdentity[]=file?.kind==='identities'?file.players:file?.names.map(name=>({name,uuid:null}))??[]
  const pending=file?.kind==='identities'?file.players.filter(player=>!existing.has(identityKey(player.uuid))):[]
  const count=entries.length
  const window=playerWindow(entries,query,player=>`${player.name} ${player.uuid??''}`,page)
  const blocked=disabled||phase!=='idle'
  const modeReady=file?.kind==='identities'?!running:running
  const choose=async()=>{
    const current=scope.current;if(disabled||current.working)return
    current.working=true;setPhase('picking');setError(null)
    try{
      const selected=await api.pickWhitelistFile()
      if(current.active&&scope.current===current&&selected){setFile(selected);setQuery('');setPage(0);setNotice(null)}
    }catch(cause){if(current.active&&scope.current===current)setError(errorText(cause))}
    finally{current.working=false;if(current.active&&scope.current===current)setPhase('idle')}
  }
  const submit=async()=>{
    const current=scope.current;if(!file||!modeReady||blocked||current.working||!count)return
    const selection:WhitelistSelection=file.kind==='identities'?{kind:'identities',players:[...file.players]}:{kind:'names',names:[...file.names]}
    current.working=true;setPhase('sending');setError(null);setNotice(null)
    try{
      const outcome=await onImport(selection)
      if(!outcome||!current.active||scope.current!==current)return
      setFile(null);setQuery('');setPage(0)
      if(outcome.mode==='commands')setNotice(t('Se enviaron {0} solicitudes de lista blanca. Minecraft actualizará la lista al procesarlas.',outcome.submitted))
      else if(outcome.added===0)setNotice(t('La lista ya contiene esos UUID. No se modificaron archivos.'))
      else setNotice(outcome.previousFile?t('Se añadieron {0} identidades para el próximo inicio. Copia anterior: {1}',outcome.added,outcome.previousFile):t('Se añadieron {0} identidades para el próximo inicio.',outcome.added))
      try{await onRefresh()}catch{if(current.active&&scope.current===current)setError(t('La importación se aplicó, pero no se pudo actualizar la vista. Vuelve a consultar la lista.'))}
    }catch(cause){if(current.active&&scope.current===current)setError(errorText(cause))}
    finally{current.working=false;if(current.active&&scope.current===current)setPhase('idle')}
  }
  return <div className="whitelist-import" aria-busy={phase!=='idle'}>
    <div><strong>{t('Importar jugadores')}</strong><p>{t('Elige un archivo .txt con un nombre por línea o un whitelist.json de Minecraft.')}</p></div>
    <button disabled={blocked} onClick={()=>void choose()}>{phase==='picking'?t('Eligiendo archivo…'):t('Elegir archivo')}</button>
    {file&&<section className="whitelist-review" aria-label={t('Revisar lista blanca')}>
      <header><strong title={file.fileName}>{file.fileName}</strong><span>{file.kind==='identities'?t('{0} nuevos · {1} ya autorizados',pending.length,entries.length-pending.length):t('{0} nombres por resolver en Minecraft',entries.length)}</span></header>
      <p className="field-help">{file.kind==='identities'?t('Se conservan los UUID del archivo. Las cuentas con el mismo nombre permanecen separadas.'):t('Minecraft resolverá los nombres. Un nombre coincidente no confirma que sea la misma cuenta.')}</p>
      <label className="form-row" htmlFor="whitelist-import-search"><span>{t('Buscar en el archivo')}</span><input id="whitelist-import-search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} maxLength={128} disabled={blocked} placeholder={t('Nombre o UUID')}/></label>
      <ul className="whitelist-review-list" tabIndex={window.total?0:undefined} aria-label={t('Jugadores del archivo')}>{window.rows.map(player=>{
        const authorized=!!player.uuid&&existing.has(identityKey(player.uuid))
        return <li key={player.uuid??player.name.toLowerCase()}><PlayerHead name={player.name} uuid={player.uuid}/><div className="whitelist-review-identity"><strong>{player.name}</strong><small>{player.uuid??t('UUID pendiente de resolver')}</small></div><span className={authorized?'existing':'new'}>{authorized?t('Ya autorizado'):player.uuid?t('Nuevo'):t('Resolver en Minecraft')}</span></li>
      })}</ul>
      {!window.total&&<p role="status">{t('No hay coincidencias. Prueba otro nombre, UUID o IP.')}</p>}
      {window.pages>1&&<nav className="player-pagination" aria-label={t('Páginas del archivo')}><span>{t('Página')} {window.page+1} {t('de')} {window.pages}</span><div><button disabled={blocked||window.page===0} onClick={()=>setPage(window.page-1)}>{t('Anterior')}</button><button disabled={blocked||window.page+1>=window.pages} onClick={()=>setPage(window.page+1)}>{t('Siguiente')}</button></div></nav>}
      <footer>{!modeReady&&<span>{file.kind==='identities'?t('Detén el servidor para importar los UUID sin sobrescribir cambios de Minecraft.'):t('Inicia el servidor para que Minecraft resuelva los nombres del archivo TXT.')}</span>}<div><button disabled={blocked} className="text-button" onClick={()=>{setFile(null);setError(null)}}>{t('Cancelar')}</button><button disabled={blocked||!modeReady||!count} className="primary-button" onClick={()=>void submit()}>{phase==='sending'?t('Aplicando…'):file.kind==='identities'?t('Importar {0} identidades',count):t('Enviar {0} nombres',count)}</button></div></footer>
    </section>}
    {notice&&<p className="player-notice" role="status">{translatedMessage(notice)}</p>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
  </div>
}
