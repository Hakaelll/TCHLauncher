import {useEffect,useRef,useState,type FormEvent} from 'react'
import {api,type ContentAvailability,type ContentSearch,type InstalledContent,type Server} from './api'
import {checkPage} from './contentBrowse'

const categories=[['adventure','Aventura'],['storage','Almacenamiento'],['worldgen','Mundo y estructuras'],['mobs','Criaturas'],['equipment','Equipamiento'],['technology','Tecnología'],['optimization','Rendimiento'],['utility','Utilidades'],['library','Bibliotecas']]
const pluginCategories=[['management','Administración'],['utility','Utilidades'],['social','Social'],['game-mechanics','Mecánicas'],['optimization','Rendimiento'],['worldgen','Mundo']]
const errorText=(e:unknown)=>e instanceof Error?e.message:String(e)
type AvailabilityState={value?:ContentAvailability;error?:string}

function ProjectIcon({url,title}:{url:string|null;title:string}) {
  const [image,setImage]=useState<string|null>(null)
  useEffect(()=>{let active=true;setImage(null);if(url)api.contentIcon(url).then(value=>{if(active)setImage(value)}).catch(()=>{});return()=>{active=false}},[url])
  return <span className="content-project-icon" aria-hidden="true">{image?<img src={image} alt="" loading="lazy"/>:<span>{title.slice(0,1).toUpperCase()}</span>}</span>
}

export function ContentDiscovery({server,active,files,locked,stopped,preparing,onPrepare}:{server:Server;active:boolean;files:InstalledContent[];locked:boolean;stopped:boolean;preparing:string|null;onPrepare:(id:string,version:string)=>void}) {
  const [query,setQuery]=useState(''),[term,setTerm]=useState(''),[category,setCategory]=useState(''),[sort,setSort]=useState('relevance'),[allVersions,setAllVersions]=useState(false),[offset,setOffset]=useState(0),[reload,setReload]=useState(0)
  const [results,setResults]=useState<ContentSearch|null>(null),[searching,setSearching]=useState(false),[error,setError]=useState<string|null>(null),[availability,setAvailability]=useState<Record<string,AvailabilityState>>({})
  const generation=useRef(0),loadedKey=useRef('')
  const key=JSON.stringify([server.id,server.version,server.platform,term,category,sort,allVersions,offset,reload])
  const loader=server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'
  useEffect(()=>{
    if(!active)return
    // Reopening the inventory preserves the current page and checked releases.
    if(loadedKey.current===key&&results)return
    const sequence=++generation.current;let alive=true
    const current=()=>alive&&sequence===generation.current
    setSearching(true);setError(null);setResults(null);setAvailability({})
    void api.searchContent(server.id,term,offset,{category,sort,allVersions}).then(async value=>{
      if(!current())return
      loadedKey.current=key;setResults(value);setSearching(false)
      await checkPage(value.hits,async project=>{
        try {
          const result=project.versions.includes(server.version)?await api.contentAvailability(server.id,project.project_id):{versionId:null,versionNumber:null,channel:null,environment:null}
          if(current())setAvailability(previous=>({...previous,[project.project_id]:{value:result}}))
        }catch(cause){if(current())setAvailability(previous=>({...previous,[project.project_id]:{error:errorText(cause)}}))}
      },current)
    }).catch(cause=>{if(current()){setError(errorText(cause));setSearching(false)}})
    return()=>{alive=false;generation.current++;loadedKey.current=''}
  // Results are intentionally not a dependency: setting them must not restart a search.
  },[active,key])
  const filter=(update:()=>void)=>{update();setOffset(0)}
  const submit=(event:FormEvent)=>{event.preventDefault();filter(()=>setTerm(query.trim()));setReload(value=>value+1)}
  const retry=async(id:string)=>{
    const sequence=generation.current;setAvailability(previous=>({...previous,[id]:{}}))
    try{const value=await api.contentAvailability(server.id,id);if(sequence===generation.current)setAvailability(previous=>({...previous,[id]:{value}}))}
    catch(cause){if(sequence===generation.current)setAvailability(previous=>({...previous,[id]:{error:errorText(cause)}}))}
  }
  return <div className="content-discovery" aria-busy={searching}>
    <div className="content-discovery-heading"><div><h3>{server.platform==='paper'?'Descubre tu próximo plugin':'Descubre tu próximo mod'}</h3><p>Modrinth · {loader} · Minecraft {server.version}</p></div><span className="content-source-label">MODRINTH</span></div>
    <form className="content-search" onSubmit={submit}><label className="sr-only" htmlFor="content-search">Buscar en Modrinth</label><input id="content-search" type="search" value={query} maxLength={200} onChange={event=>setQuery(event.target.value)} placeholder="Nombre, mecánica o autor…"/><button disabled={searching||locked} type="submit">{searching?'Buscando…':'Buscar'}</button></form>
    <div className="content-filters"><label>Categoría<select disabled={locked} value={category} onChange={event=>filter(()=>setCategory(event.target.value))}><option value="">Todas las categorías</option>{(server.platform==='paper'?pluginCategories:categories).map(([id,label])=><option key={id} value={id}>{label}</option>)}</select></label><label>Ordenar por<select disabled={locked} value={sort} onChange={event=>filter(()=>setSort(event.target.value))}><option value="relevance">Relevancia</option><option value="downloads">Más descargados</option><option value="follows">Más seguidos</option><option value="updated">Actualizados recientemente</option><option value="newest">Nuevos proyectos</option></select></label><label className="content-version-filter"><input type="checkbox" checked={allVersions} disabled={locked} onChange={event=>filter(()=>setAllVersions(event.target.checked))}/>Incluir otras versiones</label></div>
    <div className="content-results-label" role="status">{searching?'Buscando proyectos…':results?`${results.total_hits.toLocaleString('es-ES')} proyectos${allVersions?'':` para ${server.version}`}`:'Busca en Modrinth'}{results&&<span>Compatibilidad comprobada por publicación</span>}</div>
    {error&&<div className="form-error" role="alert">{error} <button disabled={searching} onClick={()=>setReload(value=>value+1)}>Reintentar</button></div>}
    {results?.hits.length===0&&!searching&&<div className="content-empty"><h3>No hay coincidencias</h3><p>Prueba otra categoría, nombre o incluye otras versiones para ver qué está disponible.</p><button onClick={()=>filter(()=>{setCategory('');setTerm('');setQuery('');setAllVersions(true)})}>Ampliar búsqueda</button></div>}
    <div className="content-results">{results?.hits.map(project=>{
      const status=availability[project.project_id],release=status?.value,installed=files.some(file=>file.projectId===project.project_id),beta=release?.channel&&release.channel!=='release'
      return <article className="content-project" key={project.project_id}>
        <ProjectIcon url={project.icon_url} title={project.title}/><div className="content-project-body"><div className="content-project-title"><h3>{project.title}</h3>{installed&&<span className="content-installed-badge">Instalado</span>}</div><span className="content-author">{project.author}</span><p>{project.description}</p><div className="content-tags">{project.categories.filter(value=>value!==server.platform).slice(0,3).map(value=><span key={value}>{[...categories,...pluginCategories].find(([id])=>id===value)?.[1]??value}</span>)}{project.downloads!=null&&<span>{new Intl.NumberFormat('es-ES',{notation:'compact'}).format(project.downloads)} descargas</span>}</div></div>
        <div className="content-project-footer"><div className={`content-availability ${status?.error?'unknown':release?.versionId?beta?'experimental':'compatible':release?'unavailable':'checking'}`}><strong>{status?.error?'Sin comprobar':!release?'Comprobando versión…':release.versionId?`${server.version} · ${loader}${beta?` · ${release.channel}`:''}`:`Sin versión de servidor para ${server.version}`}</strong><span>{status?.error?status.error:release?.versionId?`Publicación ${release.versionNumber}`:release?'No se instalará un archivo de otra versión.':'Comprobando versión, cargador y entorno.'}</span></div>{status?.error?<button disabled={locked} onClick={()=>void retry(project.project_id)}>Recomprobar</button>:<button disabled={!stopped||locked||!release?.versionId} onClick={()=>{if(release?.versionId)onPrepare(project.project_id,release.versionId)}}>{preparing===project.project_id?'Revisando…':installed?'Revisar versión':beta?'Revisar beta':'Revisar instalación'}</button>}</div>
      </article>
    })}</div>
    {results&&results.total_hits>results.limit&&<div className="content-pagination"><button disabled={searching||offset===0||locked} onClick={()=>setOffset(Math.max(0,offset-results.limit))}>Anterior</button><span>Página {Math.floor(results.offset/results.limit)+1} de {Math.ceil(results.total_hits/results.limit)}</span><button disabled={searching||results.offset+results.limit>=results.total_hits||locked} onClick={()=>setOffset(offset+results.limit)}>Siguiente</button></div>}
  </div>
}
