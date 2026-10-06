import {useCallback,useEffect,useRef,useState} from 'react'
import {api,type ServerState} from './api'
import {createPropertiesDraft,emptyPropertiesDraft} from './propertiesDraft'

export function useServerProperties(id:string|null,serverState:ServerState|undefined){
  const [snapshot,setSnapshot]=useState(emptyPropertiesDraft)
  const manager=useRef<ReturnType<typeof createPropertiesDraft>|null>(null)
  const previous=useRef({id,serverState})
  useEffect(()=>{
    const current=createPropertiesDraft(api.properties,setSnapshot);manager.current=current
    current.select(id);void current.refresh().catch(()=>{})
    return()=>{current.dispose();if(manager.current===current)manager.current=null}
  },[id])
  useEffect(()=>{
    const changed=previous.current.id===id&&previous.current.serverState!==serverState
    previous.current={id,serverState}
    if(changed)void manager.current?.refresh().catch(()=>{})
  },[id,serverState])
  const refresh=useCallback(async(serverId?:string,replace=false)=>{await manager.current?.refresh(serverId,replace)},[])
  const edit=useCallback((values:Record<string,string>)=>manager.current?.edit(values),[])
  const acknowledge=useCallback((serverId:string,values:Record<string,string>)=>manager.current?.acknowledge(serverId,values),[])
  return {...(snapshot.serverId===id?snapshot:{...emptyPropertiesDraft(),serverId:id}),refresh,edit,acknowledge}
}
