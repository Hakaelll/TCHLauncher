import {useEffect,useState} from 'react'
import {api} from './api'

export function PlayerHead({name,uuid}: {name:string;uuid?:string|null}) {
  const [source,setSource]=useState<string|null>(null)
  const [unavailable,setUnavailable]=useState(false)
  useEffect(()=>{
    let active=true;setSource(null);setUnavailable(false)
    if (uuid) api.playerHead(uuid).then(value=>{if(active){setSource(value);setUnavailable(!value)}}).catch(()=>{if(active)setUnavailable(true)})
    else setUnavailable(true)
    return ()=>{active=false}
  },[uuid])
  return source ? <img className="player-head" src={source} alt="" width={32} height={32} onError={()=>{setSource(null);setUnavailable(true)}}/>
    : <span className="player-head fallback" aria-hidden="true" title={unavailable ? 'Skin no disponible' : 'Cargando skin'}>{name.slice(0,1).toUpperCase()}</span>
}
