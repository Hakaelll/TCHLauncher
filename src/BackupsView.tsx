import {fmtBytes} from './format'
import {useEffect,useId,useMemo,useRef,useState} from 'react'
import {listWindow} from './listWindow'
import type {Backup,Server} from './api'
import {getLanguage,t,translatedMessage,formatDate} from './locale'
import {BackupPolicyPanel} from './BackupPolicyPanel'
import {Icon} from './ServerPresentation'
import {useListMotion} from './motion'
import './backups.css'

type Props={server:Server;copies:Backup[];loaded:boolean;loading:boolean;error:string|null;busy:boolean;reasons:Record<string,string>;onCreate:()=>void;onRefresh:()=>Promise<void>;onRestore:(copy:Backup)=>void}

export function BackupsView({server,copies,loaded,loading,error,busy,reasons,onCreate,onRefresh,onRestore}:Props){
  const [query,setQuery]=useState(''),[reason,setReason]=useState('')
  const [page,setPage]=useState(0)
  const listId=useId(),resultRef=useRef<HTMLParagraphElement>(null)
  const list=useRef<HTMLDivElement>(null)
  const locale=getLanguage()==='en'?'en-US':'es-ES'
  const date=(copy:Backup)=>formatDate(copy.createdAt)
  const reasonText=(copy:Backup)=>copy.reason?reasons[copy.reason]??t('Motivo no disponible'):t('Motivo no registrado')
  const visible=useMemo(()=>copies.filter(copy=>(!reason||copy.reason===reason)&&(!query.trim()||`${date(copy)} ${reasonText(copy)} ${copy.platform??''} ${copy.minecraftVersion??''}`.toLocaleLowerCase(locale).includes(query.trim().toLocaleLowerCase(locale)))),[copies,query,reason,locale,reasons])
  const window=listWindow(visible,page)
  useEffect(()=>{setPage(window.page)},[window.page])
  const changePage=(value:number)=>{setPage(value);resultRef.current?.focus({preventScroll:true});resultRef.current?.parentElement?.scrollIntoView({block:'start'})}
  const latest=copies.reduce<Backup|null>((found,copy)=>!found||copy.createdAt>found.createdAt?copy:found,null)
  const availableReasons=[...new Set(copies.map(copy=>copy.reason).filter((value):value is string=>!!value))]
  const stopped=['stopped','crashed','error'].includes(server.state)
  useListMotion(list,window.rows.map(copy=>copy.id).join())
  return <section className="backup-view">
    <header className="backup-heading"><div><h2>{t('Copias de seguridad')}</h2><p>{t('Se guardan en el almacenamiento local de la aplicación.')}</p></div><div className="backup-heading-actions"><button disabled={loading} onClick={()=>void onRefresh().catch(()=>{})}>{loading?t('Consultando…'):t('Actualizar lista')}</button><button className="primary-button" disabled={busy||!stopped} onClick={onCreate}><Icon name="plus" size={16}/>{t('Crear copia')}</button></div></header>
    {!stopped&&<p className="inline-note">{t('Detén el servidor para crear una copia consistente.')}</p>}
    {error&&<p className="form-error" role="alert">{t('No se pudo consultar la lista:')} {translatedMessage(error)} {loaded?t('Se muestran las últimas copias conocidas.'):''}</p>}
    {!loaded&&!error&&<p className="empty-inline" role="status">{t('Cargando copias…')}</p>}
    {copies.length>0&&<>
      <div className="backup-tools"><label className="backup-search"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 6 6"/></svg><input type="search" aria-label={t('Buscar copias')} placeholder={t('Buscar por fecha, motivo o versión…')} value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}}/></label><label className="backup-reason-filter"><span>{t('Motivo')}</span><select value={reason} onChange={event=>{setReason(event.target.value);setPage(0)}}><option value="">{t('Todos los motivos')}</option>{availableReasons.map(value=><option value={value} key={value}>{reasons[value]??t('Motivo no disponible')}</option>)}</select></label><p ref={resultRef} tabIndex={-1} className="backup-count" role="status">{t('{0} de {1} copias',visible.length,copies.length)}{window.pages>1&&<span className="sr-only"> · {t('Página')} {window.page+1} {t('de')} {window.pages}</span>}</p></div>
      <div id={listId} className="backup-history" ref={list} aria-busy={loading||undefined}>
        <div className="backup-history-head" aria-hidden="true"><span>{t('Fecha y motivo')}</span><span>{t('Tamaño')}</span><span>{t('Acción')}</span></div>
        {window.rows.map(copy=><article className={`backup-history-row${copy.id===latest?.id?' newest':''}`} data-layout-id={copy.id} key={copy.id}>
          <div className="backup-history-description"><svg className="backup-file-mark" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden="true"><path d="M4 3h16v18H4zM7 3v6h10V3M7 21v-7h10v7M14 5h1v2h-1"/></svg><div><time dateTime={copy.createdAt}>{date(copy)}</time><div className="backup-history-meta"><strong>{reasonText(copy)}</strong>{copy.id===latest?.id&&<span className="backup-latest">{t('Más reciente')}</span>}</div><small>{copy.minecraftVersion?`${copy.platform??t('Motor no registrado')} · Minecraft ${copy.minecraftVersion}`:t('Versión no registrada')}</small>{copy.metadataError&&<p className="form-error">{translatedMessage(copy.metadataError)}</p>}</div></div>
          <span className="backup-history-size">{fmtBytes(copy.sizeBytes)}</span><button className="backup-restore" disabled={busy||loading||!!error||!stopped} aria-label={t('Restaurar copia del {0}',date(copy))} onClick={()=>onRestore(copy)}>{t('Restaurar')}<Icon name="arrow" size={15}/></button>
        </article>)}
        {!visible.length&&<div className="backup-no-results"><p>{t('No hay copias que coincidan con los filtros.')}</p><button onClick={()=>{setQuery('');setReason('');setPage(0)}}>{t('Limpiar filtros')}</button></div>}
      </div>
      {window.pages>1&&<nav className="installed-pagination" aria-label={t('Páginas de copias de seguridad')}><span>{t('Página')} {window.page+1} {t('de')} {window.pages}</span><div><button aria-controls={listId} disabled={window.page===0} onClick={()=>changePage(window.page-1)}>{t('Anterior')}</button><button aria-controls={listId} disabled={window.page+1>=window.pages} onClick={()=>changePage(window.page+1)}>{t('Siguiente')}</button></div></nav>}
    </>}
    {!copies.length&&loaded&&!error&&<div className="backup-empty"><svg viewBox="0 0 48 48" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true"><path d="M8 8h32v32H8zM14 8v12h20V8M14 40V28h20v12M28 12h2v4h-2"/></svg><div><h3>{t('Aún no hay copias de este servidor.')}</h3><p>{t('Se guardan en el almacenamiento local de la aplicación.')}</p></div></div>}
    <div className="backup-preferences"><BackupPolicyPanel key={server.id} id={server.id} disabled={busy||!stopped} onChanged={onRefresh}/></div>
  </section>
}
