import {fmtBytes as sizeLabel,errorText} from './format'
import {translatedMessage,t} from './locale.ts'
import { useEffect, useRef, useState, useId, type FormEvent } from 'react'
import {useServerRead} from './useServerRead'
import { useListMotion } from './motion'
import {ActionMenu,type MenuTarget} from './ActionMenu'
import { Modal } from './Modal'
import { api, type Server, type World, type WorldInspection } from './api'
import {WorldImportPreview} from './WorldImportPreview'

function WorldIcon({world}: {world: World}) {
  return world.iconData
    ? <img className="world-icon" src={world.iconData} alt=""/>
    : <span className="world-icon default" aria-hidden="true"><svg width="27" height="27" viewBox="0 0 28 28" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><path d="M4 21.5V7l6.5-2.5 7 2.5L24 4.5V19l-6.5 2.5-7-2.5L4 21.5Z"/><path d="M10.5 4.5V19m7-12v14.5"/></svg></span>
}

type Confirmation = { kind: 'activate' | 'delete'; name: string }

export function WorldView({server,onBackupsChanged}: {server: Server; onBackupsChanged: () => Promise<void>}) {
  const menuId=useId()
  const [menu,setMenu]=useState<(MenuTarget&{name:string})|null>(null)
  const closeMenu=(restoreFocus=false)=>{if(restoreFocus&&menu?.trigger.isConnected)menu.trigger.focus({preventScroll:true});setMenu(null)}
  const inventory=useServerRead(server.id,api.worlds)
  const worlds=inventory.data??[]
  const loading=inventory.loading||(!inventory.data&&!inventory.error)
  const listRef=useRef<HTMLDivElement>(null)
  useListMotion(listRef,`${server.id}:${worlds.map(world=>world.name).join(',')}`)
  const [busy,setBusy] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const [notice,setNotice] = useState<string|null>(null)
  const [selection,setSelection] = useState<WorldInspection|null>(null)
  const [importName,setImportName] = useState('')
  const [duplicate,setDuplicate] = useState<{source:string;name:string}|null>(null)
  const [confirm,setConfirm] = useState<Confirmation|null>(null)
  const [exporting,setExporting] = useState<{name:string;content:boolean;config:boolean}|null>(null)
  const duplicateInput=useRef<HTMLInputElement>(null)
  const exportChoice=useRef<HTMLInputElement>(null)
  useEffect(()=>{duplicateInput.current?.focus({preventScroll:true})},[duplicate?.source])
  useEffect(()=>{exportChoice.current?.focus({preventScroll:true})},[exporting?.name])
  const exportOwner=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};exportOwner.current=current;return()=>{current.active=false}},[server.id])
  const cancelRef = useRef<HTMLButtonElement>(null)
  const stopped = ['stopped','crashed','error'].includes(server.state)

  useEffect(()=>{
    setMenu(null);setError(null);setNotice(null);setSelection(null);setDuplicate(null);setConfirm(null);setExporting(null);setBusy(false)
  },[server.id])
  const refresh=async(current=exportOwner.current)=>{if(current.active)await inventory.refresh(server.id)}
  const run = async (operation: () => Promise<void>, success: string, backupsChanged = false, acknowledged?:()=>void) => {
    const current=exportOwner.current
    if(current.working||!stopped||loading||inventory.error)return
    current.working=true
    setBusy(true); setError(null); setNotice(null)
    let completed=false
    try {
      await operation()
      completed=true
      if(current.active){acknowledged?.();setNotice(success)}
      await refresh(current)
      if(backupsChanged)try{await onBackupsChanged()}catch{if(current.active)setError(t("No se pudo actualizar la lista de copias. Revisa la sección Copias."))}
    } catch(error) {
      if(backupsChanged&&!completed)try{await onBackupsChanged()}catch{}
      if(current.active&&errorText(error)!=='Instalación cancelada')setError(errorText(error))
    } finally { current.working=false;if(current.active)setBusy(false) }
  }
  const choose = async () => {
    const current=exportOwner.current
    if(current.working||!stopped||loading||inventory.error)return
    current.working=true;setBusy(true);setError(null)
    try {
      const picked = await api.pickWorldFolder()
      if (current.active&&picked) { setSelection(picked); setImportName(picked.suggestedName) }
    } catch(error) { if(current.active)setError(errorText(error)) }
    finally{current.working=false;if(current.active)setBusy(false)}
  }
  const importSelected = (event: FormEvent) => {
    event.preventDefault()
    if (!selection||(selection.dimensions.length>0&&server.platform!=='paper')) return
    const name = importName.trim()
    run(async () => { await api.importWorld(server.id,selection,name) },t("Se importó «{0}».", name),false,()=>setSelection(null))
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
    if(!exporting||exportOwner.current.working||!stopped||loading||inventory.error)return
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
    if (!confirm||busy||!stopped||loading||inventory.error) return
    const {kind,name} = confirm
    setConfirm(null)
    if (kind === 'activate') run(async () => { await api.setActiveWorld(server.id,name) },t("«{0}» será el mundo principal en el próximo inicio. Se guardó una copia de seguridad.", name),true)
    else run(async () => { await api.deleteWorld(server.id,name) },t("Se borró «{0}» y se guardó una copia de seguridad.", name),true)
  }

  return <section className="worlds-view">
    <div className="section-heading"><div><h2>{t("Mundos")}</h2><p>{t("Importa y organiza los mundos de este servidor. Los cambios de archivos se hacen con el servidor detenido.")}</p></div><div className="world-heading-actions"><button disabled={busy||loading} onClick={()=>{void refresh()}}>{loading?t("Cargando mundos…"):t("Revisar mundos")}</button><button disabled={busy || !stopped || loading || !!inventory.error} onClick={choose}>{t("Importar mundo")}</button></div></div>
    {inventory.error&&<div className="panel-read-state failed"><div><strong>{t("Mundos sin confirmar")}</strong><p role="alert">{translatedMessage(inventory.error)}</p>{inventory.data&&<p>{t("Se conservan los últimos mundos leídos. Reintenta la lectura antes de cambiar archivos.")}</p>}</div><button disabled={busy||loading} onClick={()=>{void refresh()}}>{loading?t("Cargando mundos…"):t("Reintentar lectura")}</button></div>}
    {error && <p className="world-message error" role="alert">{translatedMessage(error)}</p>}
    {notice && <p className="world-message" role="status">{translatedMessage(notice)}</p>}
    {!stopped && <p className="inline-note">{t("Detén el servidor para importar, duplicar, exportar o cambiar mundos. Puedes abrir una carpeta mientras está en marcha.")}</p>}
    {selection && <form className="world-import" onSubmit={importSelected}><div className="world-import-summary"><strong>{t("Mundo seleccionado")}</strong><span>{sizeLabel(selection.sizeBytes)} · {selection.path}</span></div><label className="form-row" htmlFor="world-import-name"><span>{t("Nombre en este servidor")}</span><input id="world-import-name" disabled={busy} required maxLength={64} value={importName} onChange={event => setImportName(event.target.value)}/></label><WorldImportPreview source={selection} name={importName} platform={server.platform}/><div className="world-import-actions"><button type="button" className="text-button" disabled={busy} onClick={() => setSelection(null)}>{t("Cancelar")}</button><button className="primary-button" disabled={busy || !stopped || loading || !!inventory.error || !importName.trim() || (selection.dimensions.length>0&&server.platform!=='paper')}>{t("Importar copia")}</button></div><p>{t("El mundo original se conserva en su ubicación.")}</p></form>}
    <div className="world-list-heading"><span>{loading ? t("Cargando mundos…") : !inventory.data ? t("Mundos sin confirmar") : `${worlds.length} ${worlds.length === 1 ? t("mundo") : t("mundos")}`}</span>{inventory.data&&<span>{worlds.some(world => world.active) ? t("Principal primero") : t("Sin mundo principal generado")}</span>}</div>
    {!loading && !inventory.error && inventory.data!==null && !worlds.length && <div className="world-empty"><strong>{t("Aún no hay mundos en este servidor.")}</strong><p>{t("Inícialo para generar uno nuevo o importa una carpeta que contenga un `level.dat` de Minecraft Java.")}</p></div>}
    <div className="world-list" ref={listRef}>{worlds.map(world => <div className={`world-row ${world.active ? 'active' : ''}`} key={world.name} data-layout-id={world.name} onContextMenu={event=>{if(event.target instanceof Element&&event.target.closest('form,input,textarea'))return;const trigger=event.currentTarget.querySelector<HTMLButtonElement>('[data-world-menu]');if(trigger){event.preventDefault();setMenu({name:world.name,x:event.clientX,y:event.clientY,trigger})}}}>
      <div className="world-row-main"><WorldIcon world={world}/><div className="world-row-copy"><strong>{world.name}</strong><span>{sizeLabel(world.sizeBytes)}{world.active ? t(' · Mundo principal') : ''}</span>{!!world.dimensions?.length&&<span className="world-dimensions">{t("Dimensiones incluidas: {0}",world.dimensions.join(' · '))}</span>}</div>{world.active && <span className="world-active-mark" aria-label={t("Mundo principal")}>{t("Principal")}</span>}</div>
      <div className="world-row-actions"><button className="text-button world-export-button" disabled={busy || !stopped || loading || !!inventory.error} aria-expanded={exporting?.name===world.name} onClick={() => {setExporting({name:world.name,content:true,config:true});setDuplicate(null)}}>{t("Exportar")}</button><button className="world-menu-trigger" data-world-menu aria-label={t("Acciones de {0}",world.name)} aria-haspopup="menu" aria-expanded={menu?.name===world.name} aria-controls={menu?.name===world.name?menuId:undefined} onClick={event=>{if(menu?.name===world.name){closeMenu(true);return}const trigger=event.currentTarget,rect=trigger.getBoundingClientRect();setMenu({name:world.name,x:rect.right-218,y:rect.bottom+6,trigger})}}><svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/></svg></button></div>
      {exporting?.name===world.name&&<form className="world-export-options" onSubmit={exportSelected} aria-label={t("Exportar {0}", world.name)}>
        <div className="world-export-heading"><h3>{t("Prepara tu exportación")}</h3><span>{server.version} · {server.platform}</span></div>
        <fieldset disabled={busy}><legend>{t("Qué incluir")}</legend><div className="world-export-modes">
          <label><input ref={exportChoice} type="radio" name="world-export-mode" checked={exporting.content} onChange={()=>setExporting({...exporting,content:true})}/><span><strong>{server.platform==='paper'?t("Mundo y plugins"):t("Mundo y mods")}</strong><small>{t("Un ZIP con el mundo y los archivos de contenido, incluidos los desactivados.")}</small></span></label>
          <label><input type="radio" name="world-export-mode" checked={!exporting.content} onChange={()=>setExporting({...exporting,content:false})}/><span><strong>{t("Solo el mundo")}</strong><small>{t("Copia de la carpeta, lista para importar.")}</small></span></label>
        </div></fieldset>
        {exporting.content&&<label className="world-export-config"><input type="checkbox" disabled={busy} checked={exporting.config} onChange={event=>setExporting({...exporting,config:event.target.checked})}/><span>{server.platform==='paper'?t("Incluir configuración y datos de plugins"):t("Incluir config y defaultconfigs")}</span></label>}
        {exporting.content&&<p>{t("Incluye datos de jugadores del mundo y las configuraciones elegidas. No incluye Java ni el arranque del servidor.")}{server.platform==='paper'?exporting.config?t(' Incluye las carpetas de los plugins, config y defaultconfigs.'):t(' Solo se incluyen los JAR de plugins activos y desactivados, sin sus carpetas de datos.'):''}</p>}
        <div className="world-export-actions"><button className="text-button" type="button" disabled={busy} onClick={()=>setExporting(null)}>{t("Cancelar")}</button><button className="primary-button" disabled={busy||!stopped||loading||!!inventory.error}>{busy?t("Exportando…"):exporting.content?t("Guardar ZIP"):t("Elegir destino")}</button></div>
      </form>}
      {duplicate?.source === world.name && <form className="world-duplicate" onSubmit={duplicateSelected}><label htmlFor="duplicate-world-name">{t("Nombre de la copia")}</label><input ref={duplicateInput} id="duplicate-world-name" disabled={busy} required maxLength={64} value={duplicate.name} onChange={event => setDuplicate({...duplicate,name:event.target.value})}/><button className="primary-button" disabled={busy || !stopped || loading || !!inventory.error || !duplicate.name.trim()}>{t("Crear copia")}</button><button className="text-button" type="button" onClick={() => setDuplicate(null)}>{t("Cancelar")}</button></form>}
    </div>)}</div>
    {menu&&worlds.some(world=>world.name===menu.name)&&<ActionMenu id={menuId} target={menu} name={menu.name} label={t("Acciones de {0}",menu.name)} onClose={closeMenu} actions={[
      ...(!worlds.find(world=>world.name===menu.name)?.active?[{key:'activate',label:t("Usar como principal"),disabled:busy||!stopped||loading||!!inventory.error,run:()=>setConfirm({kind:'activate',name:menu.name})}]:[]),
      {key:'duplicate',label:t("Duplicar"),disabled:busy||!stopped||loading||!!inventory.error,run:()=>{setDuplicate({source:menu.name,name:t("{0} copia",menu.name)});setExporting(null)}},
      {key:'backup',label:t("Guardar copia"),disabled:busy||!stopped||loading||!!inventory.error,run:()=>{void run(async()=>{await api.createBackup(server.id)},t("Se guardó una copia del servidor que incluye «{0}».",menu.name),true)}},
      {key:'folder',label:t("Abrir carpeta"),disabled:busy,run:()=>{void openFolder(menu.name)}},
      ...(!worlds.find(world=>world.name===menu.name)?.active?[{key:'delete',label:t("Eliminar"),separator:true,destructive:true,disabled:busy||!stopped||loading||!!inventory.error,run:()=>setConfirm({kind:'delete',name:menu.name})}]:[]),
    ]}/>}
    {!!worlds.length && <p className="world-footnote">{t("Las copias incluyen el servidor completo y todos sus mundos.")}</p>}
    {confirm && <Modal labelledBy="world-confirm-title" describedBy="world-confirm-description" busy={busy} initialFocusRef={cancelRef} onClose={() => setConfirm(null)}><h2 id="world-confirm-title">{confirm.kind === 'activate' ? t("Cambiar mundo principal") : t("Eliminar mundo")}</h2><p id="world-confirm-description">{confirm.kind === 'activate' ? t("«{0}» se cargará en el próximo inicio. Guardaremos una copia del estado actual antes de cambiar.", confirm.name) : t("«{0}» se borrará de la carpeta del servidor. Guardaremos una copia de seguridad antes.", confirm.name)}</p>{!!worlds.find(world=>world.name===confirm.name)?.dimensions?.length&&<p className="field-help">{t(confirm.kind==='delete'?"También se borrarán estas dimensiones: {0}.":"Se cargarán junto con estas dimensiones: {0}.",worlds.find(world=>world.name===confirm.name)?.dimensions.join(', '))}</p>}<div className="world-confirm-actions"><button data-modal-close ref={cancelRef} disabled={busy} onClick={() => setConfirm(null)}>{t("Cancelar")}</button><button className={confirm.kind === 'delete' ? 'danger-button' : 'primary-button'} disabled={busy||!stopped||loading||!!inventory.error} onClick={confirmAction}>{confirm.kind === 'activate' ? t("Cambiar mundo") : t("Guardar copia y eliminar")}</button></div></Modal>}
  </section>
}
