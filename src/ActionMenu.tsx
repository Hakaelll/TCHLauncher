import {useLayoutEffect,useRef,type ReactNode} from 'react'
import {createPortal} from 'react-dom'

export type MenuTarget={x:number;y:number;trigger:HTMLButtonElement}
export type MenuAction={key:string;label:string;disabled?:boolean;destructive?:boolean;separator?:boolean;icon?:ReactNode;run:()=>void}
const searchText=(text:string)=>text.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase().trim()

/** Shared viewport menu; action callbacks always use the current rendered state. */
export function ActionMenu({target,label,name,actions,onClose,id}:{target:MenuTarget;label:string;name:string;actions:MenuAction[];onClose:(restoreFocus?:boolean)=>void;id?:string}) {
  const menuRef=useRef<HTMLDivElement>(null)
  const typed=useRef({text:'',at:0})
  const closeRef=useRef(onClose)
  closeRef.current=onClose
  useLayoutEffect(()=>{
    const menu=menuRef.current;if(!menu)return
    typed.current={text:'',at:0}
    const rect=menu.getBoundingClientRect()
    menu.style.left=`${Math.max(8,Math.min(target.x,window.innerWidth-rect.width-8))}px`
    menu.style.top=`${Math.max(8,Math.min(target.y,window.innerHeight-rect.height-8))}px`
    ;(menu.querySelector<HTMLButtonElement>('button:not(:disabled)')??menu).focus({preventScroll:true})
    const outside=(event:PointerEvent)=>{if(!menu.contains(event.target as Node))closeRef.current(false)}
    const resize=()=>closeRef.current(false)
    const scroll=(event:Event)=>{if(!(event.target instanceof Node)||!menu.contains(event.target))closeRef.current(false)}
    document.addEventListener('pointerdown',outside,true);window.addEventListener('resize',resize);window.addEventListener('scroll',scroll,true)
    return()=>{document.removeEventListener('pointerdown',outside,true);window.removeEventListener('resize',resize);window.removeEventListener('scroll',scroll,true)}
  },[target.trigger,target.x,target.y])
  return createPortal(<div id={id} ref={menuRef} tabIndex={-1} className="server-context-menu" role="menu" aria-label={label} style={{left:target.x,top:target.y,maxHeight:'calc(100vh - 16px)',overflowY:'auto'}} onKeyDown={event=>{
    if(event.nativeEvent.isComposing)return
    const items=[...(menuRef.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')??[])]
    const current=items.indexOf(document.activeElement as HTMLButtonElement)
    if(['ArrowDown','ArrowUp','Home','End'].includes(event.key)){
      typed.current.text=''
      event.preventDefault();if(!items.length)return
      const next=event.key==='Home'?0:event.key==='End'?items.length-1:current<0?(event.key==='ArrowDown'?0:items.length-1):(current+(event.key==='ArrowDown'?1:items.length-1))%items.length
      items[next]?.focus()
    }else if(event.key==='Escape'){event.preventDefault();event.stopPropagation();onClose(true)}
    else if(event.key==='Tab')onClose(true)
    else if(event.key.length===1&&event.key!==' '&&!event.ctrlKey&&!event.metaKey&&!event.altKey){
      event.preventDefault()
      const now=performance.now(),letter=searchText(event.key)
      const previous=now-typed.current.at<700?typed.current.text:''
      const text=previous===letter?letter:previous+letter
      typed.current={text,at:now}
      const start=text.length===1?current+1:Math.max(0,current)
      for(let offset=0;offset<items.length;offset++){
        const item=items[(start+offset)%items.length]
        if(searchText(item.textContent??'').startsWith(text)){item.focus();break}
      }
    }
  }}>
    <div className="server-menu-name">{name}</div>
    {actions.map(action=><div key={action.key} role="none">{action.separator&&<div role="separator"/>}<button role="menuitem" className={action.destructive?'destructive-menu-item':undefined} disabled={action.disabled} onClick={()=>{onClose(true);action.run()}}>{action.icon}{action.label}</button></div>)}
  </div>,document.body)
}
