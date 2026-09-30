import { useEffect, useState } from 'react'
import { api, type Server } from './api'
import { listen } from '@tauri-apps/api/event'

export function ServerCover({server}:{server:Server}) {
  const [cover,setCover]=useState<string|null>(null)
  useEffect(()=>{
    let active=true,sequence=0;setCover(null)
    const refresh=()=>{const request=++sequence;return api.serverCover(server.id).then(value=>{if(active && request===sequence)setCover(value)}).catch(()=>{/* Existing icons remain usable when no full image is available. */})}
    if(server.imageData)refresh()
    const listener=listen<string>('server-artwork-changed',event=>{if(event.payload===server.id)refresh()}).catch(()=>null)
    return()=>{active=false;listener.then(unlisten=>unlisten?.())}
  },[server.id,server.imageData])
  const source=cover ?? server.imageData
  return source?<img src={source} alt=""/>:<span className="cover-monogram" aria-hidden="true">{server.name.trim().slice(0,2).toLocaleUpperCase()}</span>
}
