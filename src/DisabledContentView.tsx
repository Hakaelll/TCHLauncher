import {normalizeSearchText} from './searchText.ts'
import {useEffect,useId,useMemo,useRef,useState} from 'react'
import {listWindow} from './listWindow'
import {t} from './locale'
import type {InstalledContent} from './api'

export function DisabledContentView({files,locked,stopped,onEnable,onInstalled}:{files:InstalledContent[];locked:boolean;stopped:boolean;onEnable:(file:InstalledContent)=>void;onInstalled:()=>void}) {
  const [query,setQuery]=useState('')
  const [page,setPage]=useState(0)
  const listId=useId(),resultRef=useRef<HTMLSpanElement>(null)
  const search=normalizeSearchText(query)
  const visible=useMemo(()=>search?files.filter(file=>[file.title,file.filename].some(value=>normalizeSearchText(value).includes(search))):files,[files,search])
  const window=listWindow(visible,page,24)
  useEffect(()=>{setPage(window.page)},[window.page])
  const changePage=(value:number)=>{setPage(value);resultRef.current?.focus({preventScroll:true});resultRef.current?.parentElement?.scrollIntoView({block:'start'})}
  return <section className="disabled-content">
    <div className="content-inventory-heading"><div><h3>{t("Contenido desactivado")}</h3><p>{t("Los archivos se conservan, pero no se cargarán en el próximo inicio.")}</p></div></div>
    <p className="field-help">{t("Cada cambio guarda una copia del servidor y revisa las dependencias conocidas.")}</p>
    {files.length>0&&<div className="disabled-content-search"><label>{t("Buscar desactivados")}<input type="search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} placeholder={t("Nombre o archivo…")}/></label><span ref={resultRef} tabIndex={-1} role="status">{t("{0} de {1} archivos",visible.length,files.length)}{window.pages>1&&<span className="sr-only"> · {t("Página")} {window.page+1} {t("de")} {window.pages}</span>}</span></div>}
    {visible.length>0?<><ul id={listId} className="disabled-content-list">{window.rows.map(file=><li key={file.filename}>
      <span className="disabled-file-mark" aria-hidden="true"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M5 3h9l5 5v13H5zM14 3v5h5M9 12v5M15 12v5"/></svg></span>
      <div><h4>{file.title}</h4><p>{file.versionNumber??t("Versión no declarada")} · {file.projectId?'Modrinth':t("Archivo local")} · {(file.size/1024**2).toFixed(1)} MB</p><code>{file.filename}.disabled</code>{file.modified&&<p className="content-file-warning">{t("Modificado fuera del launcher")}</p>}</div>
      <button disabled={locked||!stopped} onClick={()=>onEnable(file)} aria-label={t("Reactivar {0}",file.title)}>{t("Reactivar")}</button>
    </li>)}</ul>{window.pages>1&&<nav className="installed-pagination" aria-label={t("Páginas de archivos desactivados")}><span>{t("Página")} {window.page+1} {t("de")} {window.pages}</span><div><button aria-controls={listId} disabled={window.page===0} onClick={()=>changePage(window.page-1)}>{t("Anterior")}</button><button aria-controls={listId} disabled={window.page+1>=window.pages} onClick={()=>changePage(window.page+1)}>{t("Siguiente")}</button></div></nav>}{!stopped&&<p className="installed-lock-note">{t("Detén el servidor para reactivar archivos.")}</p>}</>:<div className="content-empty installed-empty"><h3>{files.length?t("No hay coincidencias"):t("No hay archivos desactivados")}</h3><p>{files.length?t("Prueba otro nombre o filtro."):t("Puedes desactivar un archivo desde sus acciones en el inventario.")}</p><button onClick={files.length?()=>{setQuery('');setPage(0)}:onInstalled}>{files.length?t("Limpiar filtros"):t("Ver instalados")}</button></div>}
  </section>
}
