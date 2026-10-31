import { useEffect, useState } from 'react'
import { api, type Server } from './api'
import { listen } from '@tauri-apps/api/event'
import { MinecraftGlyph } from './MinecraftGlyph'
import { retryingSubscription } from './retryingSubscription'

export function ServerCover({server}:{server:Server}) {
  const [cover,setCover]=useState<string|null>(null)
  useEffect(()=>{
    let active=true,sequence=0;setCover(null)
    const refresh=()=>{const request=++sequence;return api.serverCover(server.id).then(value=>{if(active && request===sequence)setCover(value)}).catch(()=>{/* Existing icons remain usable when no full image is available. */})}
    if(server.imageData)refresh()
    let unavailable=false
    const stopEvents=retryingSubscription(()=>listen<string>('server-artwork-changed',event=>{if(active&&event.payload===server.id)void refresh()}),failed=>{
      if(!active)return
      if(unavailable&&!failed)void refresh()
      unavailable=failed
    })
    return()=>{active=false;stopEvents()}
  },[server.id,server.imageData])
  const source=cover ?? server.imageData
  return source?<img src={source} alt=""/>:<span className="cover-default" aria-hidden="true"><MinecraftGlyph kind={server.platform==='vanilla'?'world':'pack'}/><span className="cover-monogram">{server.name.trim().slice(0,2).toLocaleUpperCase()}</span></span>
}
