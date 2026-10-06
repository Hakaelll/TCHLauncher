import {useCallback,useEffect,useRef,useState} from 'react'
import {api} from './api'
import {createBackupList,emptyBackupList} from './backupList'

export function useBackupList(id:string|null){
  const [snapshot,setSnapshot]=useState(emptyBackupList)
  const manager=useRef<ReturnType<typeof createBackupList>|null>(null)
  useEffect(()=>{
    const current=createBackupList(api.backups,setSnapshot);manager.current=current
    current.select(id);void current.refresh().catch(()=>{})
    return()=>{current.dispose();if(manager.current===current)manager.current=null}
  },[id])
  const refresh=useCallback(async(serverId?:string)=>{await manager.current?.refresh(serverId)},[])
  return {...(snapshot.serverId===id?snapshot:{...emptyBackupList(),serverId:id}),refresh}
}
