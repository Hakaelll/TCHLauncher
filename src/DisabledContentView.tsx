import {useState} from 'react'
import {t} from './locale'
import type {InstalledContent} from './api'

export function DisabledContentView({files,locked,stopped,onEnable,onInstalled}:{files:InstalledContent[];locked:boolean;stopped:boolean;onEnable:(file:InstalledContent)=>void;onInstalled:()=>void}) {
  const [query,setQuery]=useState('')
  const search=query.trim().toLocaleLowerCase()
  const visible=files.filter(file=>[file.title,file.filename].some(value=>value.toLocaleLowerCase().includes(search)))
  return <section className="disabled-content">
    <div className="content-inventory-heading"><div><h3>{t("Contenido desactivado")}</h3><p>{t("Los archivos se conservan, pero no se cargarán en el próximo inicio.")}</p></div></div>
    <p className="field-help">{t("Cada cambio guarda una copia del servidor y revisa las dependencias conocidas.")}</p>
    {files.length>0&&<div className="disabled-content-search"><label>{t("Buscar desactivados")}<input type="search" value={query} onChange={event=>setQuery(event.target.value)} placeholder={t("Nombre o archivo…")}/></label><span role="status">{t("{0} de {1} archivos",visible.length,files.length)}</span></div>}
    {visible.length>0?<><ul className="disabled-content-list">{visible.map(file=><li key={file.filename}>
      <span className="disabled-file-mark" aria-hidden="true"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M5 3h9l5 5v13H5zM14 3v5h5M9 12v5M15 12v5"/></svg></span>
      <div><h4>{file.title}</h4><p>{file.versionNumber??t("Versión no declarada")} · {file.projectId?'Modrinth':t("Archivo local")} · {(file.size/1024**2).toFixed(1)} MB</p><code>{file.filename}.disabled</code>{file.modified&&<p className="content-file-warning">{t("Modificado fuera del launcher")}</p>}</div>
      <button disabled={locked||!stopped} onClick={()=>onEnable(file)} aria-label={t("Reactivar {0}",file.title)}>{t("Reactivar")}</button>
    </li>)}</ul>{!stopped&&<p className="installed-lock-note">{t("Detén el servidor para reactivar archivos.")}</p>}</>:<div className="content-empty installed-empty"><h3>{files.length?t("No hay coincidencias"):t("No hay archivos desactivados")}</h3><p>{files.length?t("Prueba otro nombre o filtro."):t("Puedes desactivar un archivo desde sus acciones en el inventario.")}</p><button onClick={files.length?()=>setQuery(''):onInstalled}>{files.length?t("Limpiar filtros"):t("Ver instalados")}</button></div>}
  </section>
}
