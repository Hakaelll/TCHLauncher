import {useRef,useState} from 'react'
import type {ImportInspection} from './api'
import {Icon} from './ServerPresentation'
import {listWindow} from './listWindow'
import {formatNumber,t,translatedMessage} from './locale'
import './import.css'

const propertyLabels:Record<string,string>={
  'server-port':'Puerto', 'max-players':'Jugadores máximos', gamemode:'Modo de juego', difficulty:'Dificultad',
  'online-mode':'Verificación de cuentas', 'white-list':'Lista blanca', 'enforce-whitelist':'Exigir lista blanca',
  'level-name':'Mundo principal', 'view-distance':'Distancia de visión', 'simulation-distance':'Distancia de simulación',
}
const permissionLabels:Record<string,string>={
  'whitelist.json':'Lista blanca', 'ops.json':'Operadores', 'banned-players.json':'Jugadores bloqueados', 'banned-ips.json':'Direcciones bloqueadas',
}
const valueLabels:Record<string,string>={
  true:'Activado', false:'Desactivado', survival:'Supervivencia', creative:'Creativo', adventure:'Aventura',
  spectator:'Espectador', peaceful:'Pacífica', easy:'Fácil', normal:'Normal', hard:'Difícil',
}

function InventoryFiles({title,files}: {title:string;files:readonly string[]}) {
  const [requestedPage,setPage]=useState(0)
  const resultRef=useRef<HTMLUListElement>(null)
  const changePage=(value:number)=>{setPage(value);resultRef.current?.focus({preventScroll:true});resultRef.current?.scrollIntoView({block:'start'})}
  const page=listWindow(files,requestedPage,24)
  return <details className="import-inventory-files">
    <summary><Icon name="folder" size={16}/><span>{title}</span><strong>{formatNumber(files.length)}</strong><span className="import-inventory-toggle"><Icon name="arrow" size={13}/></span></summary>
    {files.length ? <>
      <ul ref={resultRef} tabIndex={-1} aria-label={title}>{page.rows.map(name=><li key={name}><span className="import-file-mark" aria-hidden="true"/>{name}</li>)}</ul>
      {page.pages>1&&<nav className="import-inventory-pages" aria-label={title}>
        <button type="button" disabled={page.page===0} onClick={()=>changePage(page.page-1)}>{t('Anterior')}</button>
        <span>{t('Página {0} de {1}',page.page+1,page.pages)}</span>
        <button type="button" disabled={page.page+1===page.pages} onClick={()=>changePage(page.page+1)}>{t('Siguiente')}</button>
      </nav>}
    </> : <p className="field-help">{t('No se encontraron archivos en esta categoría.')}</p>}
  </details>
}

export function ImportInventory({inventory}: {inventory:ImportInspection['inventory']}) {
  const properties=Object.entries(inventory.properties)
  return <section className="import-inventory" aria-labelledby="import-inventory-title">
    <header><div><h2 id="import-inventory-title">{t('Antes de importar')}</h2>
      <p>{t('Revisa los archivos y los ajustes de la carpeta seleccionada.')}</p></div>
      <span className="import-readonly"><Icon name="check" size={14}/>{t('Inspección de solo lectura')}</span>
    </header>
    <div className="import-inventory-columns">
      <div className="import-inventory-content">
        <h3>{t('Contenido encontrado')}</h3>
        <InventoryFiles title={t('Carpetas de mundo')} files={inventory.worlds}/>
        <InventoryFiles title={t('Archivos de mods')} files={inventory.mods}/>
        <InventoryFiles title={t('Archivos de plugins')} files={inventory.plugins}/>
        <p className="field-help">{t('Los archivos encontrados no confirman que Minecraft los haya cargado. Los archivos desactivados se muestran con su nombre completo.')}</p>
      </div>
      <div className="import-inventory-settings">
        <h3>{t('Configuración detectada')}</h3>
        {properties.length ? <dl>{properties.map(([key,value])=><div key={key}>
          <dt title={key}>{t(propertyLabels[key]??key)}</dt>
          <dd>{key==='level-name'?value:valueLabels[value]?t(valueLabels[value]):value}</dd>
        </div>)}</dl> : <p className="field-help">{t('No hay ajustes disponibles para mostrar.')}</p>}
      </div>
    </div>
    <div className="import-inventory-permissions">
      <h3>{t('Acceso y permisos')}</h3>
      <dl>{inventory.permissions.map(file=><div key={file.filename}>
        <dt title={file.filename}>{t(permissionLabels[file.filename]??file.filename)}</dt>
        <dd>{file.entries===null?t('No disponible'):formatNumber(file.entries)}</dd>
      </div>)}</dl>
      <p className="field-help">{t('Se muestran cantidades, sin exponer nombres, UUID ni direcciones IP.')}</p>
    </div>
    {(inventory.warnings.length>0||inventory.properties['online-mode']==='false')&&<aside className="import-inventory-warnings" aria-label={t('Revisar antes de importar')}>
      <h3>{t('Revisar antes de importar')}</h3>
      <ul>{inventory.properties['online-mode']==='false'&&<li>{t('La verificación de cuentas está desactivada. Revisa online-mode antes de jugar.')}</li>}
        {inventory.warnings.map(warning=><li key={warning}>{translatedMessage(warning)}</li>)}</ul>
    </aside>}
  </section>
}
