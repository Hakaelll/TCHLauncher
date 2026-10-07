import {translatedMessage} from './locale.ts'
import {localizedStatic,t,getLanguage} from './locale.ts'
import {useEffect,useMemo,useRef,useState} from 'react'
import {api,type ContentAudit,type InstalledContent,type LogLine,type Server} from './api'
import {checkPage} from './contentBrowse'
import {contentSignals,contentStatus} from './contentHealth'
import {Modal} from './Modal'
import {contentWindow,type ContentFilter,type ContentSource} from './contentWindow'

const statusLabels=localizedStatic({error:'Error en el registro',warning:'Revisar',observed:'Presente en el último log',unknown:'Carga sin confirmar'})
const clock=(value:string|null)=>value?.includes('T')?new Date(value).toLocaleString(getLanguage()==='en'?'en-US':'es-ES'):value??t("Hora no disponible")

export function InstalledContentView({server,files,audit,logs,locked,stopped,onPrepare,onRemove,onBrowse,onConsole}:{server:Server;files:InstalledContent[];audit:ContentAudit|null;logs:LogLine[];locked:boolean;stopped:boolean;onPrepare:(id:string)=>void;onRemove:(file:InstalledContent)=>void;onBrowse:()=>void;onConsole:(query:string)=>void}) {
  const [query,setQuery]=useState(''),[filter,setFilter]=useState<ContentFilter>('all')
  const [source,setSource]=useState<ContentSource>('all')
  const [layout,setLayout]=useState<'grid'|'compact'>('grid')
  const [sort,setSort]=useState<'name'|'attention'>('name')
  const [icons,setIcons]=useState<Record<string,string|null>>({})
  const cache=useRef<Record<string,string|null>>({})
  const [selected,setSelected]=useState<string|null>(null)
  const rows=useMemo(()=>files.map(file=>{
    const check=audit?.checks.find(check=>check.filename===file.filename)
    const signals=contentSignals(file,check,logs)
    return {file,check,signals,status:contentStatus(check,signals),title:check?.descriptor.name??file.title}
  }),[files,audit,logs])
  const attention=rows.filter(row=>['error','warning'].includes(row.status)).length
  const errors=rows.filter(row=>row.status==='error').length
  const visible=contentWindow(rows,query,filter,source,sort)
  const resetFilters=()=>{setQuery('');setFilter('all');setSource('all')}
  const stateFilters: {value:ContentFilter;label:string;count:number}[]=[
    {value:'all',label:t("Todos"),count:files.length},
    {value:'attention',label:t("Para revisar"),count:attention},
    {value:'observed',label:t("En el registro"),count:rows.filter(row=>row.status==='observed').length},
    {value:'unknown',label:t("Sin confirmar"),count:rows.filter(row=>row.status==='unknown').length},
  ]
  const detail=rows.find(row=>row.file.filename===selected)
  const iconKey=(file:InstalledContent)=>`${server.id}:${file.filename}:${file.sha512}`
  const iconFiles=JSON.stringify(visible.map(({file})=>[file.filename,file.sha512]))
  useEffect(()=>{
    let active=true
    const currentKeys=new Set(files.map(iconKey))
    for(const key of Object.keys(cache.current))if(!currentKeys.has(key))delete cache.current[key]
    setIcons({...cache.current})
    // Four bounded disk reads; diagnostics changing never reload unchanged icons.
    void checkPage(visible.map(row=>row.file),async file=>{
      const key=iconKey(file)
      if(!(key in cache.current)) {
        try{const value=await api.installedContentIcon(server.id,file.filename);if(active)cache.current[key]=value}
        catch{if(active)cache.current[key]=null}
      }
      if(active)setIcons(previous=>({...previous,[key]:cache.current[key]??null}))
    },()=>active)
    return()=>{active=false}
  },[server.id,iconFiles])
  useEffect(()=>{if(selected&&!detail)setSelected(null)},[selected,detail])
  return <>
    <div className="installed-content-overview"><div><strong>{files.length}</strong><span>{server.platform==='paper'?t("plugins instalados"):t("mods instalados")}</span></div><div className={attention?'has-attention':''}><strong>{attention}</strong><span>{t("para revisar")}</span></div><p role="status" aria-live="polite">{errors?t("{0} {1} con errores del registro reciente.", errors, errors===1?'archivo relacionado':'archivos relacionados'):t("Los problemas del registro se mostrarán junto al archivo relacionado.")}</p>{attention>0&&<button className="installed-review-action" onClick={()=>{setQuery('');setSource('all');setFilter('attention');setSort('attention')}}>{t("Revisar avisos")} <span aria-hidden="true">→</span></button>}</div>
    {files.length>0&&<section className="installed-workbench" aria-label={t("Filtrar archivos instalados")}>
      <div className="installed-state-filters" role="group" aria-label={t("Estado en el registro")}>{stateFilters.map(item=><button key={item.value} aria-pressed={filter===item.value} className={item.value} onClick={()=>setFilter(item.value)}>{translatedMessage(item.label)}<span>{item.count}</span></button>)}</div>
      <div className="installed-content-tools">
        <label>{t("Buscar instalados")}<input id="installed-content-search" type="search" value={query} onChange={event=>setQuery(event.target.value)} placeholder={t("Nombre, ID o archivo…")}/></label>
        <label>{t("Origen")}<select value={source} onChange={event=>setSource(event.target.value as ContentSource)}><option value="all">{t("Todos los orígenes")}</option><option value="managed">Modrinth</option><option value="local">{t("Archivos locales")}</option></select></label>
        <label>{t("Ordenar")}<select value={sort} onChange={event=>setSort(event.target.value as typeof sort)}><option value="name">{t("Nombre")}</option><option value="attention">{t("Problemas primero")}</option></select></label>
        <div className="installed-layout-control" role="group" aria-label={t("Presentación de los archivos")}><button aria-pressed={layout==='grid'} onClick={()=>setLayout('grid')}>{t("Tarjetas")}</button><button aria-pressed={layout==='compact'} onClick={()=>setLayout('compact')}>{t("Compacta")}</button></div>
      </div>
      <div className="installed-results"><span role="status" aria-live="polite">{visible.length}  {t("de")} {files.length}  {t("archivos")}</span>{(query||filter!=='all'||source!=='all')&&<button className="text-button" onClick={resetFilters}>{t("Limpiar filtros")}</button>}</div>
    </section>}
    {files.length>0&&(!stopped||locked)&&<p className="installed-lock-note">{!stopped?t("Detén el servidor para cambiar versiones o retirar archivos. Puedes consultar sus detalles y avisos."):t("Hay una operación en curso. Podrás cambiar los archivos cuando termine.")}</p>}
    {visible.length?<div className={`installed-content-grid ${layout}`}>{visible.map(({file,check,signals,status,title})=>
      <article className={`installed-content-card ${status}`} key={file.filename}>
        <header><span className="installed-mod-icon" aria-hidden="true">{icons[iconKey(file)]?<img src={icons[iconKey(file)]!} alt=""/>:<svg viewBox="0 0 32 32" fill="currentColor"><path d="M6 5h20v4h4v18H2V9h4zm4 0v4h12V5zM6 13v10h20V13z"/><path className="mod-icon-cut" d="M13 13h6v10h-6z"/></svg>}</span><div><h3>{title}</h3><p className="installed-version"><span>{t("Versión")}</span><strong>{check?.descriptor.version??file.versionNumber??t("No declarada")}</strong></p><small>{file.projectId?'Modrinth':t("Añadido localmente")} · {(file.size/1024**2).toFixed(1)} MB</small></div></header>
        <div className="installed-mod-status"><span aria-hidden="true">{status==='error'?'!':status==='warning'?'!':status==='observed'?'✓':'·'}</span><strong>{statusLabels[status]}</strong></div>
        <p className="installed-mod-note">{signals[0]?signals[0].message:check?.issues[0]??(check?.listedInLog===false?t("Su ID no aparece en la lista Fabric del último log."):file.modified?t("El archivo ha cambiado fuera del launcher."):check?.descriptor.id?`ID: ${check.descriptor.id}`:file.filename)}</p>
        <footer><button onClick={()=>setSelected(file.filename)} aria-label={t("Ver detalles de {0}", title)}>{signals.length?t("Ver aviso"):t("Detalles")}</button>{signals.length>0&&<button className="installed-console-action" onClick={()=>onConsole((signals.find(signal=>signal.level==='error')??signals[0]).query)} aria-label={t("Ver mensajes de {0} en consola", title)}>{t("Consola")}</button>}{file.projectId&&<button disabled={!stopped||locked} onClick={()=>onPrepare(file.projectId!)} aria-label={t("Revisar versión de {0}", title)}>{t("Versión")}</button>}<button className="text-button content-remove" disabled={!stopped||locked} onClick={()=>onRemove(file)} aria-label={`Retirar ${title}`}>{t("Retirar")}</button></footer>
      </article>
    )}</div>:<div className="content-empty installed-empty"><svg className="installed-empty-glyph" aria-hidden="true" viewBox="0 0 32 32" fill="currentColor"><path d="M6 5h20v4h4v18H2V9h4zm4 0v4h12V5zM6 13v10h20V13z"/></svg><h3>{files.length?t("No hay coincidencias"):server.platform==='paper'?t("Tu servidor, con más posibilidades"):t("Haz tu mundo a tu medida")}</h3><p>{files.length?t("Prueba otro nombre o filtro."):t("Explora contenido compatible antes de instalarlo.")}</p><button className="primary-button" onClick={files.length?resetFilters:onBrowse}>{files.length?t("Mostrar todos"):t("Buscar {0}", server.platform==='paper'?'plugins':'mods')}</button></div>}
    {detail&&<Modal labelledBy="installed-mod-title" onClose={()=>setSelected(null)} className="installed-mod-dialog"><div className="installed-mod-detail-heading"><span className="installed-mod-icon" aria-hidden="true">{icons[iconKey(detail.file)]?<img src={icons[iconKey(detail.file)]!} alt=""/>:<span>JAR</span>}</span><div><h2 id="installed-mod-title">{detail.title}</h2><p>{detail.check?.descriptor.version??detail.file.versionNumber??t("Archivo local")}</p></div></div><p className={`installed-mod-detail-state ${detail.status}`}>{statusLabels[detail.status]}</p><dl className="installed-mod-facts"><div><dt>{t("Archivo")}</dt><dd>{detail.file.filename}</dd></div>{detail.check?.descriptor.id&&<div><dt>{(detail.check.descriptor.ids?.length??0)>1?t("IDs declarados"):'ID'}</dt><dd>{detail.check.descriptor.ids?.join(', ')||detail.check.descriptor.id}</dd></div>}{detail.check?.descriptor.minecraft&&<div><dt>{t("Minecraft declarado")}</dt><dd>{detail.check.descriptor.minecraft}</dd></div>}{detail.check?.descriptor.apiVersion&&<div><dt>{t("API declarada")}</dt><dd>{detail.check.descriptor.apiVersion}</dd></div>}{detail.check?.descriptor.environment&&<div><dt>{t("Entorno permitido")}</dt><dd>{detail.check.descriptor.environment==='client'?t("Solo cliente"):detail.check.descriptor.environment==='server'?t("Solo servidor"):t("Cliente y servidor")}</dd></div>}<div><dt>{t("Origen")}</dt><dd>{detail.file.projectId?t("Gestionado por Modrinth"):t("Archivo local")}</dd></div>{detail.file.modified&&<div><dt>{t("Archivo cambiado")}</dt><dd>{t("Modificado fuera del launcher")}</dd></div>}</dl>
      {!!detail.check?.descriptor.dependencies?.length&&<details className="installed-dependencies"><summary>{t("Dependencias declaradas")} <span>{detail.check.descriptor.dependencies.length}</span></summary><p>{t("Requisitos del archivo. Las alternativas separadas por «o» son rangos admitidos. En Fabric se comprueban rangos reconocidos entre mods principales cuando los metadatos están completos. Java, Minecraft, versión del loader y selección de archivos incluidos no se verifican aquí; no confirma la carga.")}</p><ul>{detail.check.descriptor.dependencies.map(dependency=><li key={`${dependency.kind}:${dependency.id}`}><div><strong>{dependency.id}</strong><span>{({depends:t("Obligatoria"),recommends:t("Recomendada"),suggests:t("Sugerida"),conflicts:t("Conflicto · aviso"),breaks:t("Incompatible · impide iniciar")})[dependency.kind]}</span></div><code>{dependency.alternatives.join(' o ')}</code></li>)}</ul></details>}
      {detail.check?.issues.map(issue=><p className="content-file-warning" key={issue}>{translatedMessage(issue)}</p>)}
      {detail.signals.length>0&&<section className="installed-mod-evidence"><h3>{t("Mensajes relacionados")}</h3><p>{t("El registro menciona este archivo o mod. Puede involucrar una dependencia; no confirma por sí solo la causa del problema.")}</p>{detail.signals.map((signal,index)=><div className={signal.level} key={index}><header><strong>{signal.level==='error'?'ERROR':t("AVISO")}</strong><time>{clock(signal.timestamp)}</time></header><pre>{signal.message}</pre><button onClick={()=>onConsole(signal.query)}>{t("Ver en consola")}</button></div>)}</section>}
      <p className="field-help">{detail.check?.listedInLog===true?t("Su ID aparece en el último log de Fabric; eso no confirma su funcionamiento dentro del mundo."):t("La presencia del JAR no confirma que se haya cargado.")}  {t("Los avisos de consola usan las últimas")} {logs.length}  {t("líneas disponibles.")}</p><div className="dialog-actions"><button data-modal-close>{t("Cerrar")}</button></div>
    </Modal>}
  </>
}
