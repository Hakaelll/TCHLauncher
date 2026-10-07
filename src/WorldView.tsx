import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import { useEffect, useRef, useState, type FormEvent } from 'react'
import { useListMotion } from './motion'
import { Modal } from './Modal'
import { api, type Server, type World, type WorldInspection } from './api'

const sizeLabel = (bytes: number) => bytes < 1024 * 1024
  ? `${Math.round(bytes / 1024)} KB`
  : bytes < 1024 * 1024 * 1024
    ? `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    : `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
const errorText = (error: unknown) => error instanceof Error ? error.message : String(error)

function WorldIcon({world}: {world: World}) {
  return world.iconData
    ? <img className="world-icon" src={world.iconData} alt=""/>
    : <span className="world-icon default" aria-hidden="true"><svg width="27" height="27" viewBox="0 0 28 28" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M4 21.5V7l6.5-2.5 7 2.5L24 4.5V19l-6.5 2.5-7-2.5L4 21.5Z"/><path d="M10.5 4.5V19m7-12v14.5"/></svg></span>
}

type Confirmation = { kind: 'activate' | 'delete'; name: string }

export function WorldView({server,onBackupsChanged}: {server: Server; onBackupsChanged: () => Promise<void>}) {
  const [worlds,setWorlds] = useState<World[]>([])
  const listRef=useRef<HTMLDivElement>(null)
  useListMotion(listRef,`${server.id}:${worlds.map(world=>world.name).join(',')}`)
  const [loading,setLoading] = useState(true)
  const [busy,setBusy] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const [notice,setNotice] = useState<string|null>(null)
  const [selection,setSelection] = useState<WorldInspection|null>(null)
  const [importName,setImportName] = useState('')
  const [duplicate,setDuplicate] = useState<{source:string;name:string}|null>(null)
  const [confirm,setConfirm] = useState<Confirmation|null>(null)
  const [exporting,setExporting] = useState<{name:string;content:boolean;config:boolean}|null>(null)
  const exportOwner=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};exportOwner.current=current;return()=>{current.active=false}},[server.id])
  const cancelRef = useRef<HTMLButtonElement>(null)
  const stopped = ['stopped','crashed','error'].includes(server.state)

  useEffect(() => {
    let active = true
    setLoading(true); setWorlds([]); setError(null); setNotice(null); setSelection(null); setDuplicate(null); setConfirm(null); setExporting(null); setBusy(false)
    api.worlds(server.id).then(value => { if (active) setWorlds(value) })
      .catch(error => { if (active) setError(errorText(error)) })
      .finally(() => { if (active) setLoading(false) })
    return () => { active = false }
  },[server.id])


  const refresh = async (current = exportOwner.current) => {
    const value = await api.worlds(server.id)
    if(current.active)setWorlds(value)
  }
  const run = async (operation: () => Promise<void>, success: string, backupsChanged = false, acknowledged?:()=>void) => {
    const current=exportOwner.current
    if(current.working||!stopped)return
    current.working=true
    setBusy(true); setError(null); setNotice(null)
    let completed=false
    try {
      await operation()
      completed=true
      if(current.active){acknowledged?.();setNotice(success)}
      try { await refresh(current) } catch(error) { if(current.active)setError(t("La operación se completó, pero no se pudo actualizar la lista de mundos: {0}",errorText(error))) }
      if(backupsChanged)try{await onBackupsChanged()}catch{if(current.active)setError(t("No se pudo actualizar la lista de copias. Revisa la sección Copias."))}
    } catch(error) {
      if(backupsChanged&&!completed)try{await onBackupsChanged()}catch{}
      if(current.active&&errorText(error)!=='Instalación cancelada')setError(errorText(error))
    } finally { current.working=false;if(current.active)setBusy(false) }
  }
  const choose = async () => {
    const current=exportOwner.current
    if(current.working||!stopped)return
    current.working=true;setBusy(true);setError(null)
    try {
      const picked = await api.pickWorldFolder()
      if (current.active&&picked) { setSelection(picked); setImportName(picked.suggestedName) }
    } catch(error) { if(current.active)setError(errorText(error)) }
    finally{current.working=false;if(current.active)setBusy(false)}
  }
  const importSelected = (event: FormEvent) => {
    event.preventDefault()
    if (!selection) return
    const path = selection.path
    const name = importName.trim()
    run(async () => { await api.importWorld(server.id,path,name) },t("Se importó «{0}».", name),false,()=>setSelection(null))
  }
  const duplicateSelected = (event: FormEvent) => {
    event.preventDefault()
    if (!duplicate) return
    const {source} = duplicate
    const name = duplicate.name.trim()
    run(async () => { await api.duplicateWorld(server.id,source,name) },t("Se creó la copia «{0}».", name),false,()=>setDuplicate(null))
  }
  const exportSelected = async (event: FormEvent) => {
    event.preventDefault()
    if(!exporting||exportOwner.current.working||!stopped)return
    const current=exportOwner.current,{name,content,config}=exporting
    current.working=true
    setBusy(true); setError(null); setNotice(null)
    try {
      const destination = await api.exportWorld(server.id,name,content,config)
      if(current.active&&destination){setNotice(t("Exportación guardada en {0}", destination));setExporting(null)}
    } catch(error) { if(current.active&&errorText(error)!=='Instalación cancelada')setError(errorText(error)) }
    finally { current.working=false;if(current.active)setBusy(false) }
  }
  const openFolder = async (name: string) => {
    const current=exportOwner.current
    setError(null)
    try { await api.openWorldFolder(server.id,name) }
    catch(error) { if(current.active)setError(errorText(error)) }
  }
  const confirmAction = () => {
    if (!confirm) return
    const {kind,name} = confirm
    setConfirm(null)
    if (kind === 'activate') run(async () => { await api.setActiveWorld(server.id,name) },t("«{0}» será el mundo principal en el próximo inicio. Se guardó una copia de seguridad.", name),true)
    else run(async () => { await api.deleteWorld(server.id,name) },t("Se borró «{0}» y se guardó una copia de seguridad.", name),true)
  }

  return <section className="worlds-view">
    <div className="section-heading"><div><h2>{t("Mundos")}</h2><p>{t("Importa y organiza los mundos de este servidor. Los cambios de archivos se hacen con el servidor detenido.")}</p></div><button disabled={busy || !stopped} onClick={choose}>{t("Importar mundo")}</button></div>
    {error && <p className="world-message error" role="alert">{translatedMessage(error)}</p>}
    {notice && <p className="world-message" role="status">{translatedMessage(notice)}</p>}
    {!stopped && <p className="inline-note">{t("Detén el servidor para importar, duplicar, exportar o cambiar mundos. Puedes abrir una carpeta mientras está en marcha.")}</p>}
    {selection && <form className="world-import" onSubmit={importSelected}><div className="world-import-summary"><strong>{t("Mundo seleccionado")}</strong><span>{sizeLabel(selection.sizeBytes)} · {selection.path}</span></div><label className="form-row" htmlFor="world-import-name"><span>{t("Nombre en este servidor")}</span><input id="world-import-name" required maxLength={64} value={importName} onChange={event => setImportName(event.target.value)}/></label><div className="world-import-actions"><button type="button" className="text-button" onClick={() => setSelection(null)}>{t("Cancelar")}</button><button className="primary-button" disabled={busy || !stopped || !importName.trim()}>{t("Importar copia")}</button></div><p>{t("El mundo original se conserva en su ubicación.")}</p></form>}
    <div className="world-list-heading"><span>{loading ? t("Cargando mundos…") : `${worlds.length} ${worlds.length === 1 ? t("mundo") : t("mundos")}`}</span><span>{worlds.some(world => world.active) ? t("Principal primero") : t("Sin mundo principal generado")}</span></div>
    {!loading && !worlds.length && <div className="world-empty"><strong>{t("Aún no hay mundos en este servidor.")}</strong><p>{t("Inícialo para generar uno nuevo o importa una carpeta que contenga un `level.dat` de Minecraft Java.")}</p></div>}
    <div className="world-list" ref={listRef}>{worlds.map(world => <div className={`world-row ${world.active ? 'active' : ''}`} key={world.name} data-layout-id={world.name}>
      <div className="world-row-main"><WorldIcon world={world}/><div className="world-row-copy"><strong>{world.name}</strong><span>{sizeLabel(world.sizeBytes)}{world.active ? t(' · Mundo principal') : ''}</span></div>{world.active && <span className="world-active-mark" aria-label={t("Mundo principal")}>{t("Principal")}</span>}</div>
      <div className="world-row-actions">{!world.active && <button className="text-button" disabled={busy || !stopped} onClick={() => setConfirm({kind:'activate',name:world.name})}>{t("Usar como principal")}</button>}<button className="text-button" disabled={busy || !stopped} onClick={() => setDuplicate({source:world.name,name:t("{0} copia",world.name)})}>{t("Duplicar")}</button><button className="text-button world-export-button" disabled={busy || !stopped} aria-expanded={exporting?.name===world.name} onClick={() => {setExporting({name:world.name,content:true,config:true});setDuplicate(null)}}>{t("Exportar")}</button><button className="text-button" disabled={busy || !stopped} onClick={() => run(async () => { await api.createBackup(server.id) },t("Se guardó una copia del servidor que incluye «{0}».", world.name),true)}>{t("Guardar copia")}</button><button className="text-button" disabled={busy} onClick={() => openFolder(world.name)}>{t("Abrir carpeta")}</button>{!world.active && <button className="text-button danger" disabled={busy || !stopped} onClick={() => setConfirm({kind:'delete',name:world.name})}>{t("Eliminar")}</button>}</div>
      {exporting?.name===world.name&&<form className="world-export-options" onSubmit={exportSelected} aria-label={t("Exportar {0}", world.name)}>
        <div className="world-export-heading"><h3>{t("Prepara tu exportación")}</h3><span>{server.version} · {server.platform}</span></div>
        <fieldset disabled={busy}><legend>{t("Qué incluir")}</legend><div className="world-export-modes">
          <label><input type="radio" name="world-export-mode" checked={exporting.content} onChange={()=>setExporting({...exporting,content:true})}/><span><strong>{server.platform==='paper'?t("Mundo y plugins"):t("Mundo y mods")}</strong><small>{t("Un ZIP con el mundo y los archivos de contenido, incluidos los desactivados.")}</small></span></label>
          <label><input type="radio" name="world-export-mode" checked={!exporting.content} onChange={()=>setExporting({...exporting,content:false})}/><span><strong>{t("Solo el mundo")}</strong><small>{t("Copia de la carpeta, lista para importar.")}</small></span></label>
        </div></fieldset>
        {exporting.content&&<label className="world-export-config"><input type="checkbox" disabled={busy} checked={exporting.config} onChange={event=>setExporting({...exporting,config:event.target.checked})}/><span>{t("Incluir config y defaultconfigs")}</span></label>}
        {exporting.content&&<p>{t("Incluye datos de jugadores del mundo y las configuraciones elegidas. No incluye Java ni el arranque del servidor.")}{server.platform==='paper'?t(' Los plugins conservan sus carpetas de configuración.'):''}</p>}
        <div className="world-export-actions"><button className="text-button" type="button" disabled={busy} onClick={()=>setExporting(null)}>{t("Cancelar")}</button><button className="primary-button" disabled={busy||!stopped}>{busy?t("Exportando…"):exporting.content?t("Guardar ZIP"):t("Elegir destino")}</button></div>
      </form>}
      {duplicate?.source === world.name && <form className="world-duplicate" onSubmit={duplicateSelected}><label htmlFor="duplicate-world-name">{t("Nombre de la copia")}</label><input id="duplicate-world-name" autoFocus required maxLength={64} value={duplicate.name} onChange={event => setDuplicate({...duplicate,name:event.target.value})}/><button className="primary-button" disabled={busy || !duplicate.name.trim()}>{t("Crear copia")}</button><button className="text-button" type="button" onClick={() => setDuplicate(null)}>{t("Cancelar")}</button></form>}
    </div>)}</div>
    {!!worlds.length && <p className="world-footnote">{t("Las copias incluyen el servidor completo y todos sus mundos.")}</p>}
    {confirm && <Modal labelledBy="world-confirm-title" describedBy="world-confirm-description" busy={busy} initialFocusRef={cancelRef} onClose={() => setConfirm(null)}><h2 id="world-confirm-title">{confirm.kind === 'activate' ? t("Cambiar mundo principal") : t("Eliminar mundo")}</h2><p id="world-confirm-description">{confirm.kind === 'activate' ? t("«{0}» se cargará en el próximo inicio. Guardaremos una copia del estado actual antes de cambiar.", confirm.name) : t("«{0}» se borrará de la carpeta del servidor. Guardaremos una copia de seguridad antes.", confirm.name)}</p><div className="world-confirm-actions"><button data-modal-close ref={cancelRef} disabled={busy} onClick={() => setConfirm(null)}>{t("Cancelar")}</button><button className={confirm.kind === 'delete' ? 'danger-button' : 'primary-button'} disabled={busy} onClick={confirmAction}>{confirm.kind === 'activate' ? t("Cambiar mundo") : t("Guardar copia y eliminar")}</button></div></Modal>}
  </section>
}
