import { useEffect, useRef, useState, type FormEvent } from 'react'
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
  const [loading,setLoading] = useState(true)
  const [busy,setBusy] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const [notice,setNotice] = useState<string|null>(null)
  const [selection,setSelection] = useState<WorldInspection|null>(null)
  const [importName,setImportName] = useState('')
  const [duplicate,setDuplicate] = useState<{source:string;name:string}|null>(null)
  const [confirm,setConfirm] = useState<Confirmation|null>(null)
  const cancelRef = useRef<HTMLButtonElement>(null)
  const stopped = ['stopped','crashed','error'].includes(server.state)

  useEffect(() => {
    let active = true
    setLoading(true); setWorlds([]); setError(null); setNotice(null); setSelection(null); setDuplicate(null); setConfirm(null)
    api.worlds(server.id).then(value => { if (active) setWorlds(value) })
      .catch(error => { if (active) setError(errorText(error)) })
      .finally(() => { if (active) setLoading(false) })
    return () => { active = false }
  },[server.id])

  useEffect(() => { if (confirm) cancelRef.current?.focus() },[confirm])

  const refresh = async () => setWorlds(await api.worlds(server.id))
  const run = async (operation: () => Promise<void>, success: string, backupsChanged = false) => {
    setBusy(true); setError(null); setNotice(null)
    try {
      await operation()
      await refresh()
      if (backupsChanged) await onBackupsChanged()
      setNotice(success)
    } catch(error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  const choose = async () => {
    setError(null)
    try {
      const picked = await api.pickWorldFolder()
      if (picked) { setSelection(picked); setImportName(picked.suggestedName) }
    } catch(error) { setError(errorText(error)) }
  }
  const importSelected = (event: FormEvent) => {
    event.preventDefault()
    if (!selection) return
    const path = selection.path
    const name = importName.trim()
    run(async () => { await api.importWorld(server.id,path,name); setSelection(null) },`Se importó «${name}».`)
  }
  const duplicateSelected = (event: FormEvent) => {
    event.preventDefault()
    if (!duplicate) return
    const {source} = duplicate
    const name = duplicate.name.trim()
    run(async () => { await api.duplicateWorld(server.id,source,name); setDuplicate(null) },`Se creó la copia «${name}».`)
  }
  const exportSelected = async (name: string) => {
    setBusy(true); setError(null); setNotice(null)
    try {
      const destination = await api.exportWorld(server.id,name)
      if (destination) setNotice(`Mundo exportado a ${destination}`)
    } catch(error) { setError(errorText(error)) }
    finally { setBusy(false) }
  }
  const openFolder = async (name: string) => {
    setError(null)
    try { await api.openWorldFolder(server.id,name) }
    catch(error) { setError(errorText(error)) }
  }
  const confirmAction = () => {
    if (!confirm) return
    const {kind,name} = confirm
    setConfirm(null)
    if (kind === 'activate') run(async () => { await api.setActiveWorld(server.id,name) },`«${name}» será el mundo principal en el próximo inicio. Se guardó una copia de seguridad.`,true)
    else run(async () => { await api.deleteWorld(server.id,name) },`Se borró «${name}» y se guardó una copia de seguridad.`,true)
  }

  return <section className="worlds-view">
    <div className="section-heading"><div><h2>Mundos</h2><p>Importa y organiza los mundos de este servidor. Los cambios de archivos se hacen con el servidor detenido.</p></div><button disabled={busy || !stopped} onClick={choose}>Importar mundo</button></div>
    {error && <p className="world-message error" role="alert">{error}</p>}
    {notice && <p className="world-message" role="status">{notice}</p>}
    {!stopped && <p className="inline-note">Detén el servidor para importar, duplicar, exportar o cambiar mundos. Puedes abrir una carpeta mientras está en marcha.</p>}
    {selection && <form className="world-import" onSubmit={importSelected}><div className="world-import-summary"><strong>Mundo seleccionado</strong><span>{sizeLabel(selection.sizeBytes)} · {selection.path}</span></div><label className="form-row" htmlFor="world-import-name"><span>Nombre en este servidor</span><input id="world-import-name" required maxLength={64} value={importName} onChange={event => setImportName(event.target.value)}/></label><div className="world-import-actions"><button type="button" className="text-button" onClick={() => setSelection(null)}>Cancelar</button><button className="primary-button" disabled={busy || !stopped || !importName.trim()}>Importar copia</button></div><p>El mundo original se conserva en su ubicación.</p></form>}
    <div className="world-list-heading"><span>{loading ? 'Cargando mundos…' : `${worlds.length} ${worlds.length === 1 ? 'mundo' : 'mundos'}`}</span><span>{worlds.some(world => world.active) ? 'Principal primero' : 'Sin mundo principal generado'}</span></div>
    {!loading && !worlds.length && <div className="world-empty"><strong>Aún no hay mundos en este servidor.</strong><p>Inícialo para generar uno nuevo o importa una carpeta que contenga un `level.dat` de Minecraft Java.</p></div>}
    <div className="world-list">{worlds.map(world => <div className={`world-row ${world.active ? 'active' : ''}`} key={world.name}>
      <div className="world-row-main"><WorldIcon world={world}/><div className="world-row-copy"><strong>{world.name}</strong><span>{sizeLabel(world.sizeBytes)}{world.active ? ' · Mundo principal' : ''}</span></div>{world.active && <span className="world-active-mark" aria-label="Mundo principal">Principal</span>}</div>
      <div className="world-row-actions">{!world.active && <button className="text-button" disabled={busy || !stopped} onClick={() => setConfirm({kind:'activate',name:world.name})}>Usar como principal</button>}<button className="text-button" disabled={busy || !stopped} onClick={() => setDuplicate({source:world.name,name:`${world.name} copia`})}>Duplicar</button><button className="text-button" disabled={busy || !stopped} onClick={() => exportSelected(world.name)}>Exportar</button><button className="text-button" disabled={busy || !stopped} onClick={() => run(async () => { await api.createBackup(server.id) },`Se guardó una copia del servidor que incluye «${world.name}».`,true)}>Guardar copia</button><button className="text-button" disabled={busy} onClick={() => openFolder(world.name)}>Abrir carpeta</button>{!world.active && <button className="text-button danger" disabled={busy || !stopped} onClick={() => setConfirm({kind:'delete',name:world.name})}>Eliminar</button>}</div>
      {duplicate?.source === world.name && <form className="world-duplicate" onSubmit={duplicateSelected}><label htmlFor="duplicate-world-name">Nombre de la copia</label><input id="duplicate-world-name" autoFocus required maxLength={64} value={duplicate.name} onChange={event => setDuplicate({...duplicate,name:event.target.value})}/><button className="primary-button" disabled={busy || !duplicate.name.trim()}>Crear copia</button><button className="text-button" type="button" onClick={() => setDuplicate(null)}>Cancelar</button></form>}
    </div>)}</div>
    {!!worlds.length && <p className="world-footnote">Las copias incluyen el servidor completo y todos sus mundos.</p>}
    {confirm && <div className="modal-scrim" onMouseDown={event => { if (event.target === event.currentTarget && !busy) setConfirm(null) }}><div className="close-dialog world-confirm" role="dialog" aria-modal="true" aria-labelledby="world-confirm-title" onKeyDown={event => { if (event.key === 'Escape' && !busy) setConfirm(null); if (event.key === 'Tab') { const buttons = Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')); const index = buttons.indexOf(document.activeElement as HTMLButtonElement); if (event.shiftKey && index === 0) { event.preventDefault(); buttons.at(-1)?.focus() } else if (!event.shiftKey && index === buttons.length - 1) { event.preventDefault(); buttons[0]?.focus() } } }}><div className="eyebrow">Mundos del servidor</div><h2 id="world-confirm-title">{confirm.kind === 'activate' ? 'Cambiar mundo principal' : 'Eliminar mundo'}</h2><p>{confirm.kind === 'activate' ? `«${confirm.name}» se cargará en el próximo inicio. Guardaremos una copia del estado actual antes de cambiar.` : `«${confirm.name}» se borrará de la carpeta del servidor. Guardaremos una copia de seguridad antes.`}</p><div className="world-confirm-actions"><button ref={cancelRef} disabled={busy} onClick={() => setConfirm(null)}>Cancelar</button><button className={confirm.kind === 'delete' ? 'danger-button' : 'primary-button'} disabled={busy} onClick={confirmAction}>{confirm.kind === 'activate' ? 'Cambiar mundo' : 'Guardar copia y eliminar'}</button></div></div></div>}
  </section>
}
