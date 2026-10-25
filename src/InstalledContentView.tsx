import {translatedMessage,localizedStatic,t,formatDate} from './locale.ts'
import {useEffect,useMemo,useRef,useState,useId} from 'react'
import {api,type ContentAudit,type InstalledContent,type LogLine,type Server} from './api'
import {checkPage} from './contentBrowse'
import {prepareContentDiagnostics,contentSignalsFromDiagnostics,contentStatus} from './contentHealth'
import {ActionMenu,type MenuTarget} from './ActionMenu'
import {Modal} from './Modal'
import {contentWindow,type ContentFilter,type ContentSource} from './contentWindow'
import {listWindow} from './listWindow'

const statusLabels=localizedStatic({error:'Error en el registro',warning:'Revisar',observed:'Presente en el último log',unknown:'Carga sin confirmar'})
const clock=(value:string|null)=>value?.includes('T')?formatDate(value):value??t("Hora no disponible")

export function InstalledContentView({server,files,audit,logs,locked,stopped,onPrepare,onRemove,onBrowse,onConsole,onProject,onDisable,active}:{server:Server;files:InstalledContent[];audit:ContentAudit|null;logs:LogLine[];locked:boolean;stopped:boolean;onPrepare:(id:string)=>void;onRemove:(file:InstalledContent)=>void;onBrowse:()=>void;onConsole:(query:string)=>void;onProject:(id:string)=>void;onDisable:(file:InstalledContent)=>void;active:boolean}) {
  const menuId=useId(),gridId=useId()
  const resultRef=useRef<HTMLSpanElement>(null)
  const [page,setPage]=useState(0)
  const [menu,setMenu]=useState<(MenuTarget&{filename:string;hash:string})|null>(null)
  const closeMenu=(restoreFocus=false)=>{if(restoreFocus&&menu?.trigger.isConnected)menu.trigger.focus({preventScroll:true});setMenu(null)}
  const [query,setQuery]=useState(''),[filter,setFilter]=useState<ContentFilter>('all')
  const [source,setSource]=useState<ContentSource>('all')
  const [layout,setLayout]=useState<'grid'|'compact'>('grid')
  const [sort,setSort]=useState<'name'|'attention'>('name')
  const [icons,setIcons]=useState<Record<string,string|null>>({})
  const cache=useRef<Record<string,string|null>>({})
  const [selected,setSelected]=useState<string|null>(null)
  const [detailView,setDetailView]=useState<'file'|'dependencies'|'messages'>('file')
  const diagnostics=useMemo(()=>prepareContentDiagnostics(logs),[logs])
  const checks=useMemo(()=>new Map(audit?.checks.map(check=>[check.filename,check])),[audit])
  const rows=useMemo(()=>files.map(file=>{
    const check=checks.get(file.filename)
    const signals=contentSignalsFromDiagnostics(file,check,diagnostics)
    return {file,check,signals,status:contentStatus(check,signals),title:check?.descriptor.name??file.title}
  }),[files,checks,diagnostics])
  const attention=rows.filter(row=>['error','warning'].includes(row.status)).length
  const errors=rows.filter(row=>row.status==='error').length
  const visible=useMemo(()=>contentWindow(rows,query,filter,source,sort),[rows,query,filter,source,sort])
  const window=listWindow(visible,page,24)
  useEffect(()=>{setPage(window.page)},[window.page])
  const changePage=(value:number)=>{setPage(value);resultRef.current?.focus({preventScroll:true});resultRef.current?.parentElement?.scrollIntoView({block:'start'})}
  const pagination=window.pages>1&&<nav className="installed-pagination" aria-label={t("Páginas de archivos instalados")}><span>{t("Página")} {window.page+1} {t("de")} {window.pages}</span><div><button aria-controls={gridId} disabled={window.page===0} onClick={()=>changePage(window.page-1)}>{t("Anterior")}</button><button aria-controls={gridId} disabled={window.page+1>=window.pages} onClick={()=>changePage(window.page+1)}>{t("Siguiente")}</button></div></nav>
  const menuRow=menu&&active?window.rows.find(row=>row.file.filename===menu?.filename&&row.file.sha512===menu.hash):undefined
  useEffect(()=>{if(menu&&!menuRow)setMenu(null)},[menu,menuRow])
  const resetFilters=()=>{setQuery('');setFilter('all');setSource('all');setPage(0)}
  const stateFilters: {value:ContentFilter;label:string;count:number}[]=[
    {value:'all',label:t("Todos"),count:files.length},
    {value:'attention',label:t("Para revisar"),count:attention},
    {value:'observed',label:t("En el registro"),count:rows.filter(row=>row.status==='observed').length},
    {value:'unknown',label:t("Sin confirmar"),count:rows.filter(row=>row.status==='unknown').length},
  ]
  const detail=rows.find(row=>row.file.filename===selected)
  const openDetails=(filename:string,view:typeof detailView='file')=>{setDetailView(view);setSelected(filename)}
  const iconKey=(file:InstalledContent)=>`${server.id}:${file.filename}:${file.sha512}`
  const iconFiles=JSON.stringify(window.rows.map(({file})=>[file.filename,file.sha512]))
  useEffect(()=>{
    if(!active)return
    let alive=true
    const currentKeys=new Set(files.map(iconKey))
    for(const key of Object.keys(cache.current))if(!currentKeys.has(key))delete cache.current[key]
    setIcons({...cache.current})
    // Four bounded disk reads; diagnostics changing never reload unchanged icons.
    void checkPage(window.rows.map(row=>row.file).filter(file=>!(iconKey(file) in cache.current)),async file=>{
      const key=iconKey(file)
      try{const value=await api.installedContentIcon(server.id,file.filename);if(alive)cache.current[key]=value}
      catch{if(alive)cache.current[key]=null}
      if(alive)setIcons(previous=>previous[key]===cache.current[key]?previous:{...previous,[key]:cache.current[key]??null})
    },()=>alive)
    return()=>{alive=false}
  },[active,server.id,iconFiles])
  useEffect(()=>{if(selected&&!detail)setSelected(null)},[selected,detail])
  useEffect(()=>{
    if(detailView==='messages'&&!detail?.signals.length||detailView==='dependencies'&&!detail?.check?.descriptor.dependencies?.length)setDetailView('file')
  },[detailView,detail])
  return <>
    <div className="installed-content-overview"><div><strong>{files.length}</strong><span>{server.platform==='paper'?t("plugins instalados"):t("mods instalados")}</span></div><div className={attention?'has-attention':''}><strong>{attention}</strong><span>{t("para revisar")}</span></div><p role="status" aria-live="polite">{errors?t("{0} {1} con errores del registro reciente.", errors, errors===1?t("archivo relacionado"):t("archivos relacionados")):t("Los problemas del registro se mostrarán junto al archivo relacionado.")}</p>{attention>0&&<button className="installed-review-action" onClick={()=>{setQuery('');setSource('all');setFilter('attention');setSort('attention');setPage(0)}}>{t("Revisar avisos")} <span aria-hidden="true">→</span></button>}</div>
    {files.length>0&&<section className="installed-workbench" aria-label={t("Filtrar archivos instalados")}>
      <div className="installed-state-filters" role="group" aria-label={t("Estado en el registro")}>{stateFilters.map(item=><button key={item.value} aria-pressed={filter===item.value} className={item.value} onClick={()=>{setFilter(item.value);setPage(0)}}>{translatedMessage(item.label)}<span>{item.count}</span></button>)}</div>
      <div className="installed-content-tools">
        <label>{t("Buscar instalados")}<input id="installed-content-search" type="search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} placeholder={t("Nombre, ID o archivo…")}/></label>
        <label>{t("Origen")}<select value={source} onChange={event=>{const value=event.target.value;if(value==='all'||value==='managed'||value==='local'){setSource(value);setPage(0)}}}><option value="all">{t("Todos los orígenes")}</option><option value="managed">Modrinth</option><option value="local">{t("Archivos locales")}</option></select></label>
        <label>{t("Ordenar")}<select value={sort} onChange={event=>{const value=event.target.value;if(value==='name'||value==='attention'){setSort(value);setPage(0)}}}><option value="name">{t("Nombre")}</option><option value="attention">{t("Problemas primero")}</option></select></label>
        <div className="installed-layout-control" role="group" aria-label={t("Presentación de los archivos")}><button aria-pressed={layout==='grid'} onClick={()=>setLayout('grid')}><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z"/></svg>{t("Tarjetas")}</button><button aria-pressed={layout==='compact'} onClick={()=>setLayout('compact')}><svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M3 4h4v4H3zM3 10h4v4H3zM3 16h4v4H3zM11 6h10M11 12h10M11 18h10"/></svg>{t("Compacta")}</button></div>
      </div>
      <div className="installed-results"><span ref={resultRef} tabIndex={-1} role="status" aria-live="polite">{visible.length?t("Archivos {0}–{1} de {2}",window.page*24+1,window.page*24+window.rows.length,visible.length):t("No hay coincidencias")}{visible.length!==files.length?` · ${t("{0} archivos en total",files.length)}`:""}</span>{(query||filter!=='all'||source!=='all')&&<button className="text-button" onClick={resetFilters}>{t("Limpiar filtros")}</button>}</div>
    </section>}
    {pagination}
    {files.length>0&&(!stopped||locked)&&<p className="installed-lock-note">{!stopped?t("Detén el servidor para cambiar versiones o retirar archivos. Puedes consultar sus detalles y avisos."):t("Hay una operación en curso. Podrás cambiar los archivos cuando termine.")}</p>}
    {visible.length?<div id={gridId} className={`installed-content-grid ${layout}`}>{window.rows.map(({file,check,signals,status,title})=>
      <article className={`installed-content-card ${status}`} key={file.filename} onContextMenu={event=>{const trigger=event.currentTarget.querySelector<HTMLButtonElement>('[data-content-menu]');if(trigger){event.preventDefault();setMenu({filename:file.filename,hash:file.sha512,x:event.clientX,y:event.clientY,trigger})}}}>
        <header><span className="installed-mod-icon" aria-hidden="true">{icons[iconKey(file)]?<img src={icons[iconKey(file)]!} alt=""/>:<svg viewBox="0 0 32 32" fill="currentColor"><path d="M6 5h20v4h4v18H2V9h4zm4 0v4h12V5zM6 13v10h20V13z"/><path className="mod-icon-cut" d="M13 13h6v10h-6z"/></svg>}</span><div><h3>{title}</h3><p className="installed-version"><span>{t("Versión")}</span><strong>{check?.descriptor.version??file.versionNumber??t("No declarada")}</strong></p><small>{file.projectId?'Modrinth':t("Añadido localmente")} · {(file.size/1024**2).toFixed(1)} MB</small></div></header>
        <div className="installed-mod-status"><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">{status==='observed'?<path d="M4 12l5 5L20 6"/>:status==='unknown'?<><path d="M4 4h16v16H4zM12 10v7"/><path d="M12 7h.01"/></>:<><path d="M9 3h6l6 6v6l-6 6H9l-6-6V9zM12 7v6"/><path d="M12 16h.01"/></>}</svg><strong>{statusLabels[status]}</strong></div>
        <p className="installed-mod-note">{signals[0]?signals[0].message:(check?.issues[0]?translatedMessage(check.issues[0]):null)??(check?.listedInLog===false?t("Su ID no aparece en la lista Fabric del último log."):file.modified?t("El archivo ha cambiado fuera del launcher."):check?.descriptor.id?`ID: ${check.descriptor.id}`:file.filename)}</p>
        <footer><button onClick={()=>openDetails(file.filename,signals.length?'messages':'file')} aria-label={t("Ver detalles de {0}", title)}>{signals.length?t("Ver aviso"):t("Detalles")}</button>{signals.length>0&&<button className="installed-console-action" onClick={()=>onConsole((signals.find(signal=>signal.level==='error')??signals[0]).query)} aria-label={t("Ver mensajes de {0} en consola", title)}>{t("Consola")}</button>}<button className="content-menu-trigger" data-content-menu aria-label={t("Acciones de {0}",title)} aria-haspopup="menu" aria-expanded={menu?.filename===file.filename} aria-controls={menu?.filename===file.filename?menuId:undefined} onClick={event=>{if(menu?.filename===file.filename){closeMenu(true);return}const trigger=event.currentTarget,rect=trigger.getBoundingClientRect();setMenu({filename:file.filename,hash:file.sha512,x:rect.right-218,y:rect.bottom+6,trigger})}}><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/></svg></button></footer>
      </article>
    )}</div>:<div className="content-empty installed-empty"><svg className="installed-empty-glyph" aria-hidden="true" viewBox="0 0 32 32" fill="currentColor"><path d="M6 5h20v4h4v18H2V9h4zm4 0v4h12V5zM6 13v10h20V13z"/></svg><h3>{files.length?t("No hay coincidencias"):server.platform==='paper'?t("Tu servidor, con más posibilidades"):t("Haz tu mundo a tu medida")}</h3><p>{files.length?t("Prueba otro nombre o filtro."):t("Explora contenido compatible antes de instalarlo.")}</p><button className="primary-button" onClick={files.length?resetFilters:onBrowse}>{files.length?t("Mostrar todos"):t("Buscar {0}", server.platform==='paper'?'plugins':'mods')}</button></div>}
    {pagination}
    {menu&&menuRow&&<ActionMenu id={menuId} target={menu} name={menuRow.title} label={t("Acciones de {0}",menuRow.title)} onClose={closeMenu} actions={[
      ...(menuRow.file.projectId?[
        {key:'version',label:t("Versión compatible"),disabled:!stopped||locked,run:()=>onPrepare(menuRow.file.projectId!)},
        {key:'project',label:t("Ver en Modrinth"),run:()=>onProject(menuRow.file.projectId!)},
      ]:[]),
      {key:'disable',label:t("Desactivar"),disabled:!stopped||locked,run:()=>onDisable(menuRow.file)},
      {key:'remove',label:t("Retirar"),separator:!!menuRow.file.projectId,destructive:true,disabled:!stopped||locked,run:()=>onRemove(menuRow.file)},
    ]}/>}
    {detail&&<Modal labelledBy="installed-mod-title" onClose={()=>setSelected(null)} className="installed-mod-dialog"><div className="installed-mod-detail-heading"><span className="installed-mod-icon" aria-hidden="true">{icons[iconKey(detail.file)]?<img src={icons[iconKey(detail.file)]!} alt=""/>:<span>JAR</span>}</span><div><h2 id="installed-mod-title">{detail.title}</h2><p>{detail.check?.descriptor.version??detail.file.versionNumber??t("Archivo local")}</p></div></div><p className={`installed-mod-detail-state ${detail.status}`}>{statusLabels[detail.status]}</p><div className="installed-detail-navigation" role="group" aria-label={t("Información del archivo")}><button aria-pressed={detailView==='file'} onClick={()=>setDetailView('file')}>{t("Detalles")}</button>{!!detail.check?.descriptor.dependencies?.length&&<button aria-pressed={detailView==='dependencies'} onClick={()=>setDetailView('dependencies')}>{t("Dependencias")} <span>{detail.check.descriptor.dependencies.length}</span></button>}{detail.signals.length>0&&<button className={detail.status} aria-pressed={detailView==='messages'} onClick={()=>setDetailView('messages')}>{t("Mensajes relacionados")} <span>{detail.signals.length}</span></button>}</div><div className="installed-detail-body"><div hidden={detailView!=='file'}><dl className="installed-mod-facts"><div><dt>{t("Archivo")}</dt><dd>{detail.file.filename}</dd></div>{detail.check?.descriptor.id&&<div><dt>{(detail.check.descriptor.ids?.length??0)>1?t("IDs declarados"):'ID'}</dt><dd>{detail.check.descriptor.ids?.join(', ')||detail.check.descriptor.id}</dd></div>}{detail.check?.descriptor.minecraft&&<div><dt>{t("Minecraft declarado")}</dt><dd>{detail.check.descriptor.minecraft}</dd></div>}{detail.check?.descriptor.apiVersion&&<div><dt>{t("API declarada")}</dt><dd>{detail.check.descriptor.apiVersion}</dd></div>}{detail.check?.descriptor.environment&&<div><dt>{t("Entorno permitido")}</dt><dd>{detail.check.descriptor.environment==='client'?t("Solo cliente"):detail.check.descriptor.environment==='server'?t("Solo servidor"):t("Cliente y servidor")}</dd></div>}<div><dt>{t("Origen")}</dt><dd>{detail.file.projectId?t("Gestionado por Modrinth"):t("Archivo local")}</dd></div>{detail.file.modified&&<div><dt>{t("Archivo cambiado")}</dt><dd>{t("Modificado fuera del launcher")}</dd></div>}</dl>{detail.check?.issues.map(issue=><p className="content-file-warning" key={issue}>{translatedMessage(issue)}</p>)}</div>
      {!!detail.check?.descriptor.dependencies?.length&&<section hidden={detailView!=='dependencies'} className="installed-dependencies"><h3>{t("Dependencias declaradas")}</h3><p>{t("Requisitos del archivo. Las alternativas separadas por «o» son rangos admitidos. En Fabric se revisan rangos entre mods y el requisito de Minecraft de los JAR principales para versiones estables configuradas, sin overrides. Java, el loader y la selección de archivos incluidos siguen a cargo de Fabric. Los avisos no confirman la carga.")}</p><ul>{detail.check.descriptor.dependencies.map(dependency=><li key={`${dependency.kind}:${dependency.id}`}><div><strong>{dependency.id}</strong><span>{({depends:t("Obligatoria"),recommends:t("Recomendada"),suggests:t("Sugerida"),conflicts:t("Conflicto · aviso"),breaks:t("Incompatible · impide iniciar")})[dependency.kind]}</span></div><code>{dependency.alternatives.join(t(" o "))}</code></li>)}</ul></section>}
      {detail.signals.length>0&&<section hidden={detailView!=='messages'} className="installed-mod-evidence"><h3>{t("Mensajes relacionados")}</h3><p>{t("El registro menciona este archivo o mod. Puede involucrar una dependencia; no confirma por sí solo la causa del problema.")}</p>{detail.signals.map((signal,index)=><div className={signal.level} key={index}><header><strong>{signal.level==='error'?'ERROR':t("AVISO")}</strong><time>{clock(signal.timestamp)}</time></header><pre>{signal.message}</pre><button onClick={()=>onConsole(signal.query)}>{t("Ver en consola")}</button></div>)}</section>}
      </div><p className="field-help">{detail.check?.listedInLog===true?t("Su ID aparece en el último log de Fabric; eso no confirma su funcionamiento dentro del mundo."):t("La presencia del JAR no confirma que se haya cargado.")}  {t("Los avisos de consola usan las últimas")} {logs.length}  {t("líneas disponibles.")}</p><div className="dialog-actions"><button data-modal-close>{t("Cerrar")}</button></div>
    </Modal>}
  </>
}
