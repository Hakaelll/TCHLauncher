import {useCallback,useEffect,useRef,useState} from 'react'
import {createServerRead,emptyServerRead} from './serverRead'

export function useServerRead<T>(id:string|null,read:(id:string)=>Promise<T>,pollMs=0){
  const [snapshot,setSnapshot]=useState(emptyServerRead<T>)
  const manager=useRef<ReturnType<typeof createServerRead<T>>|null>(null)
  useEffect(()=>{
    const current=createServerRead(read,setSnapshot);manager.current=current;current.select(id)
    let active=true,timer:ReturnType<typeof setTimeout>|undefined
    const load=async()=>{await current.refresh().catch(()=>{});if(active&&id&&pollMs)timer=setTimeout(load,pollMs)}
    void load()
    return()=>{active=false;clearTimeout(timer);current.dispose();if(manager.current===current)manager.current=null}
  },[id,read,pollMs])
  const refresh=useCallback(async(serverId?:string)=>{await manager.current?.refresh(serverId).catch(()=>{})},[])
  const acknowledge=useCallback((serverId:string,data:T)=>manager.current?.acknowledge(serverId,data)??false,[])
  return {...(snapshot.serverId===id?snapshot:{...emptyServerRead<T>(),serverId:id}),refresh,acknowledge}
}
