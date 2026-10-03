import type {Progress} from './api'

export const isTaskActive=(progress:Progress)=>!['completed','cancelled','failed'].includes(progress.stage)

// Native snapshots are authoritative for active jobs. Session outcomes remain visible.
export function reconcileTasks(current:Record<string,Progress>,snapshot:Progress[]):Record<string,Progress>{
  const next:Record<string,Progress>={}
  for(const item of Object.values(current))if(!isTaskActive(item))next[item.serverId]=item
  for(const item of snapshot){
    const previous=current[item.serverId]
    if(previous?.taskId===item.taskId&&!isTaskActive(previous))continue
    next[item.serverId]=item
  }
  return next
}

export function dismissTask(current:Record<string,Progress>,serverId:string,taskId:string):Record<string,Progress>{
  const item=current[serverId]
  if(!item||item.taskId!==taskId||isTaskActive(item))return current
  const next={...current};delete next[serverId];return next
}

export function orderedTasks(current:Record<string,Progress>):Progress[]{
  const priority=(item:Progress)=>isTaskActive(item)?0:item.stage==='failed'?1:2
  return Object.values(current).sort((a,b)=>priority(a)-priority(b))
}
