import {translatedMessage,localizedStatic,t} from './locale.ts'
import { useRef, useState } from 'react'
import type { Server } from './api'
import { useListMotion } from './motion'
import { ServerCover } from './ServerCover'
import { libraryWindow, matchesLibraryFilter, type LibraryFilter, type LibraryOrder } from './libraryWindow'
import launcherEmblem from './assets/launcher-logo.png'
import worldArt from './assets/choice-world.png'
import importArt from './assets/choice-import.png'
import packArt from './assets/choice-pack.png'

type Props={servers:Server[];onCreate:()=>void;onImport:()=>void;onPack:()=>void;onSelect:(server:Server)=>void;onMenu:(server:Server,trigger:HTMLButtonElement,x:number,y:number)=>void}
const states:Record<string,string>=localizedStatic({stopped:'Detenido',running:'En línea',starting:'Iniciando',stopping:'Deteniendo',installing:'Instalando',crashed:'Error',error:'Error'})

export function ServerLibrary({servers,onCreate,onImport,onPack,onSelect,onMenu}:Props) {
  const [query,setQuery]=useState('')
  const [layout,setLayout]=useState<'grid'|'list'>(()=>{try{return window.localStorage.getItem('tch-library-layout')==='list'?'list':'grid'}catch{return 'grid'}})
  const [filter,setFilter]=useState<LibraryFilter>('all')
  const [order,setOrder]=useState<LibraryOrder>('existing')
  const list=useRef<HTMLDivElement>(null)
  const filtered=libraryWindow(servers,query,filter,order)
  const resetFilters=()=>{setQuery('');setFilter('all')}
  const filters: {value:LibraryFilter;label:string}[]=[{value:'all',label:t("Todos")},{value:'running',label:t("En línea")},{value:'attention',label:t("Con errores")},{value:'pinned',label:t("Fijados")}]
  useListMotion(list,`${layout}:${filtered.map(server=>server.id).join(',')}`)
  const switchLayout=(value:'grid'|'list')=>{setLayout(value);try{window.localStorage.setItem('tch-library-layout',value)}catch{/* Preference is optional. */}}
  return <section className={`server-library ${servers.length ? "" : "library-empty"}`} aria-labelledby="library-title">
    <header className="library-heading"><div className="library-heading-identity">{!servers.length && <img className="library-welcome-emblem" src={launcherEmblem} alt=""/>}<div><h1 id="library-title">{servers.length ? t("Mis servidores") : t("Tu primer servidor")}</h1><p>{servers.length ? t("{0} {1} · {2} en línea", servers.length, t(servers.length===1?'servidor':'servidores'), servers.filter(server=>server.state==='running').length) : t("Un mundo nuevo. O el que ya tienes.")}</p></div></div>{!!servers.length && <div className="library-tools"><label className="library-search"><svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 5 5"/></svg><input aria-label={t("Buscar servidores")} type="search" placeholder={t("Buscar…")} value={query} onChange={event=>setQuery(event.target.value)}/></label><div className="library-layout" role="group" aria-label={t("Vista de la biblioteca")}><button className={layout==='grid'?'active':''} aria-label={t("Vista de tarjetas")} aria-pressed={layout==='grid'} onClick={()=>switchLayout('grid')}><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"/></svg></button><button className={layout==='list'?'active':''} aria-label={t("Vista de lista")} aria-pressed={layout==='list'} onClick={()=>switchLayout('list')}><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M9 6h12M9 12h12M9 18h12M3 6h2M3 12h2M3 18h2"/></svg></button></div></div>}</header>
    {!!servers.length && <div className="library-workbench">
      <div className="library-filters" role="group" aria-label={t("Filtrar servidores")}>{filters.map(item=><button key={item.value} className={filter===item.value?'selected':''} aria-pressed={filter===item.value} onClick={()=>setFilter(item.value)}>{translatedMessage(item.label)}<span>{servers.filter(server=>matchesLibraryFilter(server,item.value)).length}</span></button>)}</div>
      <label className="library-order">{t("Ordenar")}<select value={order} onChange={event=>setOrder(event.target.value as LibraryOrder)}><option value="existing">{t("Orden de la biblioteca")}</option><option value="name">{t("Nombre")}</option><option value="recent">{t("Más recientes")}</option></select></label>
    </div>}
    <div className={`library-cards ${servers.length ? layout : "first-server-choices"}`} ref={list}>
      <button className="library-create" onClick={onCreate}><span className="choice-illustration"><img src={worldArt} alt=""/></span><strong>{t("Nuevo servidor")}</strong><span>{t("Elige versión, tipo y capacidad.")}<br/>{t("Tu mundo empieza aquí.")}</span>{!servers.length && <span className="choice-action">{t("Crear servidor")} <ChoiceArrow/></span>}</button>
      {filtered.map(server=><article className="library-card" data-state={server.state} key={server.id} data-layout-id={server.id} onContextMenu={event=>{const trigger=event.currentTarget.querySelector<HTMLButtonElement>('.library-menu');if(trigger){event.preventDefault();const rect=trigger.getBoundingClientRect();onMenu(server,trigger,event.clientX||rect.left,event.clientY||rect.bottom)}}}>
        <button className={`library-cover ${server.imageData?'has-image':''}`} onClick={()=>onSelect(server)} aria-label={t("Abrir {0}", server.name)}><ServerCover server={server}/></button>
        <div className="library-card-body"><div className="library-card-state"><span className={`library-status ${server.state}`}><span className={`state-dot ${server.state}`} aria-hidden="true"/>{states[server.state]??server.state}</span>{server.pinned&&<span className="library-pinned" title={t("Servidor fijado")}><PinIcon/>{t("Fijado")}</span>}</div><button className="library-name" onClick={()=>onSelect(server)}>{server.name}</button><p>{({vanilla:'Vanilla',paper:'Paper',fabric:'Fabric',neoforge:'NeoForge'})[server.platform]} · {server.version}</p><div className="library-card-footer"><span title={t("RAM máxima")}>{(server.memoryMaxMb/1024).toFixed(1)} GB RAM</span><span title={t("Objetivo de espacio")}>{server.storageTargetGb} GB</span><button className="library-open" onClick={()=>onSelect(server)} aria-label={t("Administrar {0}",server.name)}><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.7"><path d="m9 5 7 7-7 7"/></svg></button><button className="library-menu" aria-label={t("Opciones de {0}", server.name)} aria-haspopup="menu" onClick={event=>{const rect=event.currentTarget.getBoundingClientRect();onMenu(server,event.currentTarget,rect.left,rect.bottom+5)}}><svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/></svg></button></div></div>
      </article>)}
      {!servers.length && <button className="library-import" onClick={onImport}><span className="choice-illustration"><img src={importArt} alt=""/></span><strong>{t("Importar servidor")}</strong><span>{t("Continúa con tus archivos")}<br/>{t("y conserva tus mundos.")}</span><span className="choice-action">{t("Elegir carpeta")} <ChoiceArrow/></span></button>}
      {!servers.length && <button className="library-pack" onClick={onPack}><span className="choice-illustration"><img src={packArt} alt=""/></span><strong>{t("Importar modpack")}</strong><span>{t("Prepara un paquete .mrpack")}<br/>{t("con sus mods y dependencias.")}</span><span className="choice-action">{t("Elegir archivo")} <ChoiceArrow/></span></button>}
    </div>
    {!!servers.length && !filtered.length && <div className="library-no-results" role="status"><strong>{t("No se encontraron servidores")}</strong><p>{query.trim()?t("No hay coincidencias para «{0}» con este filtro.", query.trim()):t("No hay servidores en esta categoría.")}</p><button onClick={resetFilters}>{t("Mostrar todos")}</button></div>}
    {!!servers.length && <p className="library-result-count" role="status">{filtered.length===servers.length?`${servers.length} ${t(servers.length===1?'servidor':'servidores')}`:t("{0} de {1} servidores", filtered.length, servers.length)}</p>}
    {!!servers.length && <div className="library-import-actions"><button onClick={onImport}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><path d="M3 7V5h6l3 3h9v12H3V7Z"/></svg>{t("Importar servidor")}</button><button onClick={onPack}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><path d="m12 3 9 5v10l-9 5-9-5V8l9-5Zm0 10v10M3 8l9 5 9-5"/></svg>{t("Importar modpack")}</button></div>}
  </section>
}

function ChoiceArrow() {
  return <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><path d="M4 12h16m-6-6 6 6-6 6"/></svg>
}

function PinIcon() {
  return <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><path d="m8 3 8 0-1 7 4 4H5l4-4-1-7ZM12 14v7"/></svg>
}
