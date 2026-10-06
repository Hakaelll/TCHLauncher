import {useEffect,useRef,useState,type FormEvent} from 'react'
import {api,type ContentAvailability,type ContentKind,type ContentSearch,type InstalledContent,type Server} from './api'
import {checkPage} from './contentBrowse'

const categories=[['adventure','Aventura'],['storage','Almacenamiento'],['worldgen','Mundo y estructuras'],['mobs','Criaturas'],['equipment','Equipamiento'],['technology','Tecnología'],['optimization','Rendimiento'],['utility','Utilidades'],['library','Bibliotecas']]
const pluginCategories=[['management','Administración'],['utility','Utilidades'],['social','Social'],['game-mechanics','Mecánicas'],['optimization','Rendimiento'],['worldgen','Mundo']]
const kindLabels:Record<ContentKind,string>={mod:'Mods',plugin:'Plugins',modpack:'Modpacks',resourcepack:'Texturas',shader:'Shaders'}
const resourceCategories=[['16x','16×'],['32x','32×'],['64x','64×'],['128x','128×'],['decoration','Decoración'],['realistic','Realista'],['simplistic','Sencillo'],['themed','Temático'],['vanilla-like','Estilo Vanilla']]
const shaderCategories=[['realistic','Realista'],['fantasy','Fantasía'],['vanilla-like','Estilo Vanilla'],['low','Requisitos bajos'],['medium','Requisitos medios'],['high','Requisitos altos'],['shadows','Sombras'],['reflections','Reflejos'],['colored-lighting','Luz de colores']]
const packCategories=[['adventure','Aventura'],['challenging','Desafiante'],['quests','Misiones'],['technology','Tecnología'],['magic','Magia'],['lightweight','Ligero'],['multiplayer','Multijugador'],['optimization','Rendimiento']]
const errorText=(e:unknown)=>e instanceof Error?e.message:String(e)
type AvailabilityState={value?:ContentAvailability;error?:string}

function ProjectIcon({url,title}:{url:string|null;title:string}) {
  const [image,setImage]=useState<string|null>(null)
  useEffect(()=>{let active=true;setImage(null);if(url)api.contentIcon(url).then(value=>{if(active)setImage(value)}).catch(()=>{});return()=>{active=false}},[url])
  return <span className="content-project-icon" aria-hidden="true">{image?<img src={image} alt="" loading="lazy"/>:<span>{title.slice(0,1).toUpperCase()}</span>}</span>
}

export function ContentDiscovery({server,active,files,locked,stopped,preparing,onPrepare,onImportPack}:{server:Server;active:boolean;files:InstalledContent[];locked:boolean;stopped:boolean;preparing:string|null;onPrepare:(id:string,version:string)=>void;onImportPack:()=>void}) {
  const [kind,setKind]=useState<ContentKind>(server.platform==='paper'?'plugin':'mod')
  const installable=kind==='mod'||kind==='plugin'
  const kindOptions:ContentKind[]=[server.platform==='paper'?'plugin':'mod','modpack','resourcepack','shader']
  const availableCategories=installable?(server.platform==='paper'?pluginCategories:categories):kind==='modpack'?packCategories:kind==='shader'?shaderCategories:resourceCategories
  const [opening,setOpening]=useState<string|null>(null)
  const [query,setQuery]=useState(''),[term,setTerm]=useState(''),[category,setCategory]=useState(''),[sort,setSort]=useState('relevance'),[allVersions,setAllVersions]=useState(false),[offset,setOffset]=useState(0),[reload,setReload]=useState(0)
  const [results,setResults]=useState<ContentSearch|null>(null),[searching,setSearching]=useState(false),[error,setError]=useState<string|null>(null),[availability,setAvailability]=useState<Record<string,AvailabilityState>>({})
  const generation=useRef(0),loadedKey=useRef('')
  const key=JSON.stringify([server.id,server.version,server.platform,term,category,sort,allVersions,offset,reload,kind])
  const loader=server.platform==='paper'?'Paper':server.platform==='fabric'?'Fabric':'NeoForge'
  useEffect(()=>{
    if(!active)return
    // Reopening the inventory preserves the current page and checked releases.
    if(loadedKey.current===key&&results)return
    const sequence=++generation.current;let alive=true
    const current=()=>alive&&sequence===generation.current
    setSearching(true);setError(null);setResults(null);setAvailability({});setOpening(null)
    void api.searchContent(server.id,term,offset,{category,sort,allVersions,projectKind:kind}).then(async value=>{
      if(!current())return
      loadedKey.current=key;setResults(value);setSearching(false)
      if(!installable)return
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
  const openProject=async(id:string)=>{
    if(opening)return
    const sequence=generation.current;setOpening(id);setError(null)
    try{await api.openContentProject(id)}catch(cause){if(sequence===generation.current)setError(errorText(cause))}
    finally{if(sequence===generation.current)setOpening(null)}
  }
  const visibleResults=loadedKey.current===key?results:null
  return <div className="content-discovery" aria-busy={searching}>
    <div className="content-discovery-heading"><div><h3>{kind==='modpack'?'Encuentra tu próxima aventura':kind==='resourcepack'?'Dale otra mirada a tu mundo':kind==='shader'?'Luz y atmósfera para tu mundo':server.platform==='paper'?'Descubre tu próximo plugin':'Descubre tu próximo mod'}</h3><p>Modrinth · {installable?`${loader} · `:''}Minecraft {server.version}</p></div><span className="content-source-label">MODRINTH</span></div>
    <div className="content-kind-tabs" role="group" aria-label="Tipo de contenido">{kindOptions.map(value=><button key={value} aria-pressed={kind===value} disabled={locked} onClick={()=>filter(()=>{setKind(value);setCategory('')})}><span className={`content-kind-glyph ${value}`} aria-hidden="true"/><span>{kindLabels[value]}</span></button>)}</div>
    {!installable&&<div className="content-kind-context"><p>{kind==='modpack'?'Los modpacks se importan como un servidor nuevo. Descarga el archivo .mrpack de la publicación que elijas.':kind==='shader'?'Los shaders se usan en tu cliente de Minecraft. Aquí puedes explorar publicaciones; no se instalan en el servidor.':'Los paquetes de texturas se usan en el cliente. Aquí puedes explorar publicaciones; no se añaden a la carpeta de mods.'}</p>{kind==='modpack'&&<button disabled={locked} onClick={onImportPack}>Importar .mrpack</button>}</div>}
    <form className="content-search" onSubmit={submit}><label className="sr-only" htmlFor="content-search">Buscar en Modrinth</label><input id="content-search" type="search" value={query} maxLength={200} onChange={event=>setQuery(event.target.value)} placeholder="Nombre, mecánica o autor…"/><button disabled={searching||locked} type="submit">{searching?'Buscando…':'Buscar'}</button></form>
    <div className="content-filters"><label>Categoría<select disabled={locked} value={category} onChange={event=>filter(()=>setCategory(event.target.value))}><option value="">Todas las categorías</option>{availableCategories.map(([id,label])=><option key={id} value={id}>{label}</option>)}</select></label><label>Ordenar por<select disabled={locked} value={sort} onChange={event=>filter(()=>setSort(event.target.value))}><option value="relevance">Relevancia</option><option value="downloads">Más descargados</option><option value="follows">Más seguidos</option><option value="updated">Actualizados recientemente</option><option value="newest">Nuevos proyectos</option></select></label><label className="content-version-filter"><input type="checkbox" checked={allVersions} disabled={locked} onChange={event=>filter(()=>setAllVersions(event.target.checked))}/>Incluir otras versiones</label></div>
    <div className="content-results-label" role="status">{searching?'Buscando proyectos…':visibleResults?`${visibleResults.total_hits.toLocaleString('es-ES')} proyectos${allVersions?'':` para ${server.version}`}`:'Busca en Modrinth'}{visibleResults&&<span>{installable?'Compatibilidad comprobada por publicación':'Versiones anunciadas por el proyecto'}</span>}</div>
    {error&&<div className="form-error" role="alert">{error} <button disabled={searching} onClick={()=>setReload(value=>value+1)}>Reintentar</button></div>}
    {visibleResults?.hits.length===0&&!searching&&<div className="content-empty"><h3>No hay coincidencias</h3><p>Prueba otra categoría, nombre o incluye otras versiones para ver qué está disponible.</p><button onClick={()=>filter(()=>{setCategory('');setTerm('');setQuery('');setAllVersions(true)})}>Ampliar búsqueda</button></div>}
    <div className="content-results">{visibleResults?.hits.map(project=>{
      const status=availability[project.project_id],release=status?.value,installed=files.some(file=>file.projectId===project.project_id),beta=release?.channel&&release.channel!=='release'
      return <article className="content-project" key={project.project_id}>
        <ProjectIcon url={project.icon_url} title={project.title}/><div className="content-project-body"><div className="content-project-title"><h3>{project.title}</h3>{installed&&<span className="content-installed-badge">Instalado</span>}</div><span className="content-author">{project.author}</span><p>{project.description}</p><div className="content-tags">{project.categories.filter(value=>value!==server.platform).slice(0,3).map(value=><span key={value}>{[...categories,...pluginCategories,...resourceCategories,...shaderCategories,...packCategories].find(([id])=>id===value)?.[1]??value}</span>)}{project.downloads!=null&&<span>{new Intl.NumberFormat('es-ES',{notation:'compact'}).format(project.downloads)} descargas</span>}</div></div>
        <div className="content-project-footer">{!installable?<><div className={`content-availability ${project.versions.includes(server.version)?'compatible':'unavailable'}`}><strong>{project.versions.includes(server.version)?`El proyecto anuncia ${server.version}`:`No anuncia ${server.version}`}</strong><span>{kind==='modpack'?'Revisa el motor y los archivos para servidor.':'Contenido para el cliente de Minecraft.'}</span></div><button disabled={locked||opening!==null} onClick={()=>void openProject(project.project_id)}>{opening===project.project_id?'Abriendo…':'Ver publicaciones ↗'}</button></>:<><div className={`content-availability ${status?.error?'unknown':release?.versionId?beta?'experimental':'compatible':release?'unavailable':'checking'}`}><strong>{status?.error?'Sin comprobar':!release?'Comprobando versión…':release.versionId?`${server.version} · ${loader}${beta?` · ${release.channel}`:''}`:`Sin versión de servidor para ${server.version}`}</strong><span>{status?.error?status.error:release?.versionId?`Publicación ${release.versionNumber}`:release?'No se instalará un archivo de otra versión.':'Comprobando versión, cargador y entorno.'}</span></div>{status?.error?<button disabled={locked} onClick={()=>void retry(project.project_id)}>Recomprobar</button>:<button disabled={!stopped||locked||!release?.versionId} onClick={()=>{if(release?.versionId)onPrepare(project.project_id,release.versionId)}}>{preparing===project.project_id?'Revisando…':installed?'Revisar versión':beta?'Revisar beta':'Revisar instalación'}</button>}</>}</div>
      </article>
    })}</div>
    {visibleResults&&visibleResults.total_hits>visibleResults.limit&&<div className="content-pagination"><button disabled={searching||offset===0||locked} onClick={()=>setOffset(Math.max(0,offset-visibleResults.limit))}>Anterior</button><span>Página {Math.floor(visibleResults.offset/visibleResults.limit)+1} de {Math.ceil(visibleResults.total_hits/visibleResults.limit)}</span><button disabled={searching||visibleResults.offset+visibleResults.limit>=visibleResults.total_hits||locked} onClick={()=>setOffset(offset+visibleResults.limit)}>Siguiente</button></div>}
  </div>
}
