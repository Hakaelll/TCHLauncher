import {useLayoutEffect,useRef} from 'react'
import {createPortal} from 'react-dom'
import type {Server} from './api'

export type ServerMenuTarget={server:Server;x:number;y:number;trigger:HTMLButtonElement}
export function ServerMenu({target,busy,onClose,onPin,onSettings,onDelete}: {target:ServerMenuTarget;busy:boolean;onClose:(restoreFocus?:boolean)=>void;onPin:()=>void;onSettings:()=>void;onDelete:()=>void}) {
  const menuRef=useRef<HTMLDivElement>(null)
  useLayoutEffect(()=>{
    const menu=menuRef.current;if(!menu)return
    const rect=menu.getBoundingClientRect()
    menu.style.left=`${Math.max(8,Math.min(target.x,window.innerWidth-rect.width-8))}px`
    menu.style.top=`${Math.max(8,Math.min(target.y,window.innerHeight-rect.height-8))}px`
    menu.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus({preventScroll:true})
    const outside=(event:PointerEvent)=>{if(!menu.contains(event.target as Node))onClose(false)}
    const dismiss=()=>onClose(false)
    document.addEventListener('pointerdown',outside,true);window.addEventListener('resize',dismiss);window.addEventListener('scroll',dismiss,true)
    return ()=>{document.removeEventListener('pointerdown',outside,true);window.removeEventListener('resize',dismiss);window.removeEventListener('scroll',dismiss,true)}
  },[target.trigger,target.x,target.y,onClose])
  const select=(action:()=>void)=>{onClose(true);action()}
  return createPortal(<div ref={menuRef} className="server-context-menu" role="menu" aria-label={`Acciones de ${target.server.name}`} style={{left:target.x,top:target.y}} onKeyDown={event=>{
    const items=[...(menuRef.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [])]
    const current=items.indexOf(document.activeElement as HTMLButtonElement)
    if(['ArrowDown','ArrowUp','Home','End'].includes(event.key)){event.preventDefault();items[event.key==='Home'?0:event.key==='End'?items.length-1:(current+(event.key==='ArrowDown'?1:items.length-1))%items.length]?.focus()}
    else if(event.key==='Escape'){event.preventDefault();onClose(true)}else if(event.key==='Tab'){event.preventDefault();onClose(true)}
  }}>
    <div className="server-menu-name">{target.server.name}</div>
    <button role="menuitem" disabled={busy} onClick={()=>select(onPin)}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m15 3 6 6-4 1-4 5-4-4 5-4zM9 15l-6 6"/></svg>{target.server.pinned?'Desfijar servidor':'Fijar servidor'}</button>
    <button role="menuitem" onClick={()=>select(onSettings)}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3" fill="white"/><circle cx="15" cy="17" r="3" fill="white"/></svg>Configuración</button>
    <div role="separator"/>
    <button className="destructive-menu-item" role="menuitem" disabled={busy || !['stopped','crashed','error'].includes(target.server.state)} onClick={()=>select(onDelete)}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M4 6h16M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7m4-7v7"/></svg>Eliminar servidor</button>
  </div>,document.body)
}
