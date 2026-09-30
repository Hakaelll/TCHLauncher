import { useRef, useState } from 'react'
import type { Server } from './api'
import { useListMotion } from './motion'
import { ServerCover } from './ServerCover'

type Props={servers:Server[];onCreate:()=>void;onImport:()=>void;onPack:()=>void;onSelect:(server:Server)=>void;onMenu:(server:Server,trigger:HTMLButtonElement,x:number,y:number)=>void}
const states:Record<string,string>={stopped:'Detenido',running:'En línea',starting:'Iniciando',stopping:'Deteniendo',installing:'Instalando',crashed:'Error',error:'Error'}

export function ServerLibrary({servers,onCreate,onImport,onPack,onSelect,onMenu}:Props) {
  const [query,setQuery]=useState('')
  const [layout,setLayout]=useState<'grid'|'list'>(()=>{try{return window.localStorage.getItem('tch-library-layout')==='list'?'list':'grid'}catch{return 'grid'}})
  const list=useRef<HTMLDivElement>(null)
  const filtered=servers.filter(server=>`${server.name} ${server.platform} ${server.version}`.toLocaleLowerCase().includes(query.toLocaleLowerCase()))
  useListMotion(list,`${layout}:${filtered.map(server=>server.id).join(',')}`)
  const switchLayout=(value:'grid'|'list')=>{setLayout(value);try{window.localStorage.setItem('tch-library-layout',value)}catch{/* Preference is optional. */}}
  return <section className="server-library" aria-labelledby="library-title">
    <header className="library-heading"><div><h1 id="library-title">Mis servidores</h1>{!servers.length && <p>Crea tu primer mundo o importa el que ya tienes.</p>}</div><div className="library-tools"><label className="library-search"><svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" strokeWidth="1.6" aria-hidden="true"><circle cx="10" cy="10" r="6"/><path d="m15 15 5 5"/></svg><input aria-label="Buscar servidores" type="search" placeholder="Buscar…" value={query} onChange={event=>setQuery(event.target.value)}/></label><div className="library-layout" role="group" aria-label="Vista de la biblioteca"><button className={layout==='grid'?'active':''} aria-label="Vista de tarjetas" aria-pressed={layout==='grid'} onClick={()=>switchLayout('grid')}><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z"/></svg></button><button className={layout==='list'?'active':''} aria-label="Vista de lista" aria-pressed={layout==='list'} onClick={()=>switchLayout('list')}><svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6"><path d="M9 6h12M9 12h12M9 18h12M3 6h2M3 12h2M3 18h2"/></svg></button></div></div></header>
    <div className={`library-cards ${layout}`} ref={list}>
      <button className="library-create" onClick={onCreate}><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4" aria-hidden="true"><path d="M12 3v18M3 12h18"/></svg><strong>Nuevo servidor</strong><span>Crea un nuevo mundo<br/>de Minecraft.</span></button>
      {filtered.map(server=><article className="library-card" key={server.id} data-layout-id={server.id}>
        <button className={`library-cover ${server.imageData?'has-image':''}`} onClick={()=>onSelect(server)} aria-label={`Abrir ${server.name}`}><ServerCover server={server}/></button>
        <div className="library-card-body"><span className={`library-status ${server.state}`}><span className={`state-dot ${server.state}`} aria-hidden="true"/>{states[server.state]??server.state}</span><button className="library-name" onClick={()=>onSelect(server)}>{server.name}</button><p>{({vanilla:'Vanilla',paper:'Paper',fabric:'Fabric',neoforge:'NeoForge'})[server.platform]} · {server.version}</p><div className="library-card-footer"><span title="RAM máxima">{(server.memoryMaxMb/1024).toFixed(1)} GB RAM</span><span title="Objetivo de espacio">{server.storageTargetGb} GB</span><button className="library-open" onClick={()=>onSelect(server)} aria-label={`Administrar ${server.name}`}><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.7"><path d="m9 5 7 7-7 7"/></svg></button><button className="library-menu" aria-label={`Opciones de ${server.name}`} aria-haspopup="menu" onClick={event=>{const rect=event.currentTarget.getBoundingClientRect();onMenu(server,event.currentTarget,rect.left,rect.bottom+5)}}>···</button></div></div>
      </article>)}
      {!servers.length && <button className="library-import" onClick={onImport}><svg viewBox="0 0 24 24" width="36" height="36" fill="none" stroke="currentColor" strokeWidth="1.4" aria-hidden="true"><path d="M3 7h7l3 3h8v10H3zM3 7V4h8l3 3h7v3"/></svg><strong>Importar servidor</strong><span>Continúa con tus archivos<br/>y conserva tus mundos.</span></button>}
    </div>
    {!!query && !filtered.length && <p className="empty-inline">No hay servidores que coincidan con «{query}».</p>}
    <button className="library-import-link" onClick={onPack}>Importar un modpack .mrpack</button>
    {!!servers.length && <button className="library-import-link" onClick={onImport}>Importar un servidor existente</button>}
  </section>
}
