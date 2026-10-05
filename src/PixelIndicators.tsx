import type {CSSProperties} from 'react'

type Kind='players'|'clock'|'memory'|'cpu'
const shapes:Record<Kind,string>={
  players:'M5 2h6v6H5zM3 10h10v8H3zM15 4h5v5h-5zM15 11h7v7h-7z',
  clock:'M5 2h14v2H5zM2 5h2v14H2zM20 5h2v14h-2zM5 20h14v2H5zM5 5h14v14H5z',
  memory:'M3 5h18v14H3zM6 2h2v3H6zM11 2h2v3h-2zM16 2h2v3h-2zM6 19h2v3H6zM11 19h2v3h-2zM16 19h2v3h-2z',
  cpu:'M6 3h12v3H6zM3 6h3v12H3zM18 6h3v12h-3zM6 18h12v3H6zM7 7h10v10H7z',
}
export function PixelIcon({kind}:{kind:Kind}){
  return <svg className={`pixel-icon pixel-${kind}`} viewBox="0 0 24 24" fill="currentColor" shapeRendering="crispEdges" aria-hidden="true"><path d={shapes[kind]}/>{kind==='clock'&&<path className="pixel-icon-cut" d="M11 7h2v5h4v2h-6z"/>}{kind==='memory'&&<path className="pixel-icon-cut" d="M6 8h3v8H6zM11 8h3v8h-3zM16 8h2v8h-2z"/>}{kind==='cpu'&&<path className="pixel-icon-cut" d="M10 10h4v4h-4z"/>}</svg>
}
export function PixelMeter({ratio,label,valueText}:{ratio:number|null|undefined;label:string;valueText?:string}){
  const value=ratio!=null&&Number.isFinite(ratio)?Math.max(0,Math.min(1,ratio)):null
  return <div className={`pixel-meter ${value===null?'unavailable':''}`} role={value===null?'img':'meter'} aria-label={value===null?`${label}: no disponible`:label} aria-valuemin={value===null?undefined:0} aria-valuemax={value===null?undefined:100} aria-valuenow={value===null?undefined:value*100} aria-valuetext={valueText}>
    {Array.from({length:20},(_,index)=><span className="pixel-meter-slot" key={index} aria-hidden="true"><i style={{'--slot-fill':`${value===null?0:Math.max(0,Math.min(1,value*20-index))*100}%`} as CSSProperties}/></span>)}
  </div>
}
