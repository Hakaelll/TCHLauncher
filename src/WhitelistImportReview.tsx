import {errorText} from './format.ts'
import {useEffect,useRef,useState} from 'react'
import {api,type PlayerIdentity,type WhitelistImport} from './api'
import {t,translatedMessage} from './locale'
import {playerWindow} from './playerWindow'

export function WhitelistImportReview({whitelisted,running,disabled,onImport,onRefresh}:{whitelisted:PlayerIdentity[];running:boolean;disabled:boolean;onImport:(names:string[])=>Promise<boolean>;onRefresh:()=>Promise<void>}){
  const [file,setFile]=useState<WhitelistImport|null>(null)
  const [query,setQuery]=useState(''),[page,setPage]=useState(0)
  const [phase,setPhase]=useState<'idle'|'picking'|'sending'>('idle')
  const [error,setError]=useState<string|null>(null),[notice,setNotice]=useState<string|null>(null)
  const scope=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};scope.current=current;return()=>{current.active=false}},[])
  const existing=new Set(whitelisted.map(player=>player.name.toLowerCase()))
  const pending=file?.names.filter(name=>!existing.has(name.toLowerCase()))??[]
  const window=playerWindow(file?.names??[],query,name=>name,page)
  const blocked=disabled||phase!=='idle'
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
    const current=scope.current;if(!running||blocked||current.working||!pending.length)return
    current.working=true;setPhase('sending');setError(null);setNotice(null)
    const names=[...pending]
    try{
      if(!await onImport(names)||!current.active||scope.current!==current)return
      setFile(null);setQuery('');setPage(0)
      setNotice(t('Se enviaron {0} solicitudes de lista blanca. Minecraft actualizará la lista al procesarlas.',names.length))
      try{await onRefresh()}catch{if(current.active&&scope.current===current)setError(t('Las solicitudes se enviaron, pero no se pudo actualizar la lista.'))}
    }catch(cause){if(current.active&&scope.current===current)setError(errorText(cause))}
    finally{current.working=false;if(current.active&&scope.current===current)setPhase('idle')}
  }
  return <div className="whitelist-import" aria-busy={phase!=='idle'}>
    <div><strong>{t('Importar jugadores')}</strong><p>{t('Elige un archivo .txt con un nombre por línea o un whitelist.json de Minecraft.')}</p></div>
    <button disabled={blocked} onClick={()=>void choose()}>{phase==='picking'?t('Eligiendo archivo…'):t('Elegir archivo')}</button>
    {file&&<section className="whitelist-review" aria-label={t('Revisar lista blanca')}>
      <header><strong title={file.fileName}>{file.fileName}</strong><span>{t('{0} nuevos · {1} ya autorizados',pending.length,file.names.length-pending.length)}</span></header>
      <label className="form-row" htmlFor="whitelist-import-search"><span>{t('Buscar en el archivo')}</span><input id="whitelist-import-search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} maxLength={128} disabled={blocked} placeholder={t('Nombre de Minecraft')}/></label>
      <ul className="whitelist-review-list" tabIndex={window.total?0:undefined} aria-label={t('Jugadores del archivo')}>{window.rows.map(name=><li key={name.toLowerCase()}><strong>{name}</strong><span className={existing.has(name.toLowerCase())?'existing':'new'}>{existing.has(name.toLowerCase())?t('Ya autorizado'):t('Nuevo')}</span></li>)}</ul>
      {!window.total&&<p role="status">{t('No hay coincidencias. Prueba otro nombre, UUID o IP.')}</p>}
      {window.pages>1&&<nav className="player-pagination" aria-label={t('Páginas del archivo')}><span>{t('Página')} {window.page+1} {t('de')} {window.pages}</span><div><button disabled={blocked||window.page===0} onClick={()=>setPage(window.page-1)}>{t('Anterior')}</button><button disabled={blocked||window.page+1>=window.pages} onClick={()=>setPage(window.page+1)}>{t('Siguiente')}</button></div></nav>}
      <footer>{!running&&<span>{t('Inicia el servidor para aplicar la importación.')}</span>}<div><button disabled={blocked} className="text-button" onClick={()=>{setFile(null);setError(null)}}>{t('Cancelar')}</button><button disabled={blocked||!running||!pending.length} className="primary-button" onClick={()=>void submit()}>{phase==='sending'?t('Enviando…'):t('Añadir {0} jugadores',pending.length)}</button></div></footer>
    </section>}
    {notice&&<p className="player-notice" role="status">{translatedMessage(notice)}</p>}
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
  </div>
}
