import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {Modal} from './Modal'
import {filterCommands,type LauncherCommand} from './commands'

export function CommandPalette({commands,onClose,onExecute}:{commands:LauncherCommand[];onClose:()=>void;onExecute:(command:LauncherCommand)=>void}) {
  const [query,setQuery]=useState('')
  const [selectedId,setSelectedId]=useState<string|null>(null)
  const search=useRef<HTMLInputElement>(null)
  const list=useRef<HTMLDivElement>(null)
  const filtered=filterCommands(commands,query)
  const selected=filtered.find(command=>command.id===selectedId)??filtered[0]
  useEffect(()=>{search.current?.focus({preventScroll:true})},[])
  useEffect(()=>{if(selected)document.getElementById(`command-${selected.id}`)?.scrollIntoView({block:'nearest'})},[selected?.id])
  const execute=(command:LauncherCommand)=>{if(!command.disabledReason)onExecute(command)}
  return <Modal labelledBy="command-title" className="command-palette" onClose={onClose}>
    <div className="command-heading"><h2 id="command-title">{t("Buscar acciones")}</h2><button data-modal-close aria-label={t("Cerrar búsqueda")}>Esc</button></div>
    <input ref={search} type="search" role="combobox" aria-label={t("Buscar acciones y servidores")} aria-controls="command-results" aria-expanded="true" aria-autocomplete="list" aria-activedescendant={selected?`command-${selected.id}`:undefined} value={query} placeholder={t("Servidor o acción…")} autoComplete="off" onChange={event=>{setQuery(event.target.value);setSelectedId(null)}} onKeyDown={event=>{
      if(event.nativeEvent.isComposing)return
      if(event.key==='ArrowDown'||event.key==='ArrowUp') {
        event.preventDefault();if(!filtered.length)return
        const current=Math.max(0,filtered.findIndex(command=>command.id===selected?.id))
        setSelectedId(filtered[(current+(event.key==='ArrowDown'?1:-1)+filtered.length)%filtered.length].id)
      } else if(event.key==='Enter'&&!event.nativeEvent.isComposing) {event.preventDefault();if(selected)execute(selected)}
    }}/>
    <div className="command-results" id="command-results" ref={list} role="listbox" aria-label={t("Acciones disponibles")}>{filtered.map(command=><div key={command.id} id={`command-${command.id}`} role="option" aria-selected={selected?.id===command.id} aria-disabled={!!command.disabledReason} className={`command-result ${selected?.id===command.id?'selected':''}`} onMouseEnter={()=>setSelectedId(command.id)} onMouseDown={event=>event.preventDefault()} onClick={()=>execute(command)}><strong>{translatedMessage(command.label)}</strong><span>{command.disabledReason??command.detail}</span></div>)}</div>
    {!filtered.length&&<p className="command-empty" role="status">{t("No hay acciones que coincidan con «")}{query}».</p>}
    <p className="command-help">{t("↑ ↓ Elegir · Enter Ejecutar · Esc Cerrar")}</p>
  </Modal>
}
