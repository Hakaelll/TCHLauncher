import { useLayoutEffect, useRef, useState, type RefObject } from 'react'
import {shouldReduceMotion} from './motionPreference'

export const motion = {instant:90,interaction:120,micro:160,standard:200,navigation:240,large:300,activity:1200,tooltipDelay:180,ease:'cubic-bezier(.2,.8,.2,1)',enter:'cubic-bezier(.16,1,.3,1)',exit:'cubic-bezier(.4,0,1,1)'} as const
if(typeof document!=='undefined')for(const [name,value] of Object.entries(motion))document.documentElement.style.setProperty(`--motion-${name}`,typeof value==='number'?`${value}ms`:value)
export const reducedMotion = () => shouldReduceMotion(document.documentElement.dataset.motion,window.matchMedia('(prefers-reduced-motion: reduce)').matches)

export function MovingIndicator({active,axis='vertical',className=''}:{active:string;axis?:'vertical'|'horizontal'|'both';className?:string}) {
  const ref=useRef<HTMLSpanElement>(null)
  const [position,setPosition]=useState<{x:number;y:number;width:number;height:number}|null>(null)
  useLayoutEffect(()=>{
    const parent=ref.current?.parentElement
    if(!parent)return
    const measure=()=>{
      const target=parent.querySelector<HTMLElement>('[aria-current="page"],[aria-current="step"]')
      if(!target){setPosition(null);return}
      const bounds=target.getBoundingClientRect(),container=parent.getBoundingClientRect()
      setPosition({x:bounds.left-container.left+parent.scrollLeft,y:bounds.top-container.top+parent.scrollTop,width:bounds.width,height:bounds.height})
    }
    measure()
    const observer=new ResizeObserver(measure);observer.observe(parent)
    for(const child of parent.children)if(child!==ref.current)observer.observe(child)
    return()=>observer.disconnect()
  },[active])
  return <span ref={ref} aria-hidden="true" className={`moving-indicator ${axis} ${className}`} style={position?axis==='both'?{opacity:1,width:position.width,height:position.height,transform:`translate(${position.x}px,${position.y}px)`}:axis==='horizontal'?{opacity:1,width:1,transform:`translateX(${position.x}px) scaleX(${position.width})`}:{opacity:1,height:position.height,transform:`translateY(${position.y}px)`}:{opacity:0}}/>
}

export function useNavigationMotion(ref:RefObject<HTMLElement|null>,identity:string,direction=1,distance=6) {
  const previous=useRef(identity)
  useLayoutEffect(()=>{
    const changed=previous.current!==identity;previous.current=identity
    if(!changed || reducedMotion() || !ref.current)return
    const animation=ref.current.animate([{opacity:.88,transform:`translateX(${direction*distance}px)`},{opacity:1,transform:'translateX(0)'}],{duration:motion.navigation,easing:motion.enter})
    return()=>animation.cancel()
  },[identity,direction,distance,ref])
}

export function useListMotion(ref:RefObject<HTMLElement|null>,identity:string) {
  const positions=useRef(new Map<string,{top:number;left:number}>())
  const snapshots=useRef(new Map<string,HTMLElement>())
  const previousHeight=useRef<number|null>(null)
  useLayoutEffect(()=>{
    const parent=ref.current;if(!parent)return
    const origin=parent.getBoundingClientRect(),next=new Map<string,{top:number;left:number}>(),nextSnapshots=new Map<string,HTMLElement>(),animations:Animation[]=[],ghosts:HTMLElement[]=[]
    parent.querySelectorAll<HTMLElement>('[data-layout-id]').forEach(element=>{
      const id=element.dataset.layoutId!,bounds=element.getBoundingClientRect(),position={top:bounds.top-origin.top+parent.scrollTop,left:bounds.left-origin.left+parent.scrollLeft},before=positions.current.get(id)
      next.set(id,position)
      nextSnapshots.set(id,element.cloneNode(true) as HTMLElement)
      if(reducedMotion())return
      if(before && (Math.abs(before.top-position.top)>1 || Math.abs(before.left-position.left)>1)) {
        animations.push(element.animate([{transform:`translate(${before.left-position.left}px,${before.top-position.top}px)`},{transform:'translate(0,0)'}],{duration:motion.standard,easing:motion.ease}))
      } else if(!before && previousHeight.current!=null) {
        animations.push(element.animate([{opacity:0,transform:'translateX(-4px)'},{opacity:1,transform:'translateX(0)'}],{duration:motion.micro,easing:motion.enter}))
      }
    })
    if(!reducedMotion() && positions.current.size) {
      if(previousHeight.current!=null && Math.abs(previousHeight.current-origin.height)>1)animations.push(parent.animate([{height:`${previousHeight.current}px`},{height:`${origin.height}px`}],{duration:motion.standard,easing:motion.ease}))
      for(const [id,before] of positions.current)if(!next.has(id)) {
        const ghost=snapshots.current.get(id);if(!ghost)continue
        ghost.removeAttribute('data-layout-id');ghost.setAttribute('aria-hidden','true');ghost.inert=true
        Object.assign(ghost.style,{position:'absolute',top:`${before.top}px`,left:`${before.left}px`,width:`${ghost.getAttribute('data-motion-width') ?? '100%'}`,pointerEvents:'none',zIndex:'0'})
        parent.appendChild(ghost);ghosts.push(ghost)
        const animation=ghost.animate([{opacity:.65},{opacity:0}],{duration:motion.interaction,easing:motion.exit,fill:'forwards'})
        animation.finished.then(()=>ghost.remove()).catch(()=>{});animations.push(animation)
      }
    }
    parent.querySelectorAll<HTMLElement>('[data-layout-id]').forEach(element=>{nextSnapshots.get(element.dataset.layoutId!)?.setAttribute('data-motion-width',`${element.offsetWidth}px`)})
    previousHeight.current=origin.height;snapshots.current=nextSnapshots
    positions.current=next
    return()=>{animations.forEach(animation=>animation.cancel());ghosts.forEach(ghost=>ghost.remove())}
  },[identity,ref])
}

export function AnimatedValue({value,format=(n:number)=>String(Math.round(n))}:{value:number|null|undefined;format?:(n:number)=>string}) {
  const [display,setDisplay]=useState(value)
  const current=useRef(value)
  useLayoutEffect(()=>{
    const start=current.current
    if(value==null || start==null || !Number.isFinite(value) || reducedMotion()){current.current=value;setDisplay(value);return}
    let frame=0;const began=performance.now()
    const tick=(now:number)=>{
      if(reducedMotion()){current.current=value;setDisplay(value);return}
      const progress=Math.min(1,(now-began)/motion.micro),next=start+(value-start)*(1-(1-progress)**3)
      current.current=next;setDisplay(next)
      if(progress<1)frame=requestAnimationFrame(tick)
    }
    frame=requestAnimationFrame(tick)
    return()=>cancelAnimationFrame(frame)
  },[value])
  return <>{display==null || !Number.isFinite(display)?'—':format(display)}</>
}

export function AnimatedLine({points,times,identity,stepped=false}:{points:[number,number][];times:number[];identity:string;stepped?:boolean}) {
  const element=useRef<SVGPolylineElement>(null)
  const previous=useRef<{identity:string;points:[number,number][];times:number[]}>({identity,points,times})
  const encode=(values:[number,number][])=>{const geometry=stepped?values.flatMap((point,index)=>index?[[point[0],values[index-1][1]],point]:[point]):values;return geometry.map(point=>point.map(value=>value.toFixed(1)).join(',')).join(' ')}
  const target=encode(points)
  const timeKey=times.join(',')
  useLayoutEffect(()=>{
    const before=previous.current;previous.current={identity,points,times}
    const line=element.current
    if(!line || reducedMotion() || before.identity!==identity || !before.points.length || Math.abs(points.length-before.points.length)>1)return
    const prior=new Map(before.times.map((time,index)=>[time,before.points[index]]))
    const from=times.map(time=>prior.get(time) ?? before.points[before.points.length-1])
    const began=performance.now();let frame=0
    const tick=(now:number)=>{
      if(reducedMotion()){line.setAttribute('points',target);return}
      const progress=Math.min(1,(now-began)/motion.navigation),weight=1-(1-progress)**3
      line.setAttribute('points',encode(points.map((point,index)=>point.map((value,axis)=>from[index][axis]+(value-from[index][axis])*weight) as [number,number])))
      if(progress<1)frame=requestAnimationFrame(tick)
    }
    frame=requestAnimationFrame(tick)
    return()=>{cancelAnimationFrame(frame);line.setAttribute('points',target)}
  },[target,identity,timeKey,stepped])
  return <polyline ref={element} className="chart-line" points={target}/>
}
