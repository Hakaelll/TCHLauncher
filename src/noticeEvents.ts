import type {Progress, ServerState} from './api'

export function taskSection(kind:Progress['kind']) {
  switch(kind) {
    case 'backup': case 'restore': return 'backups'
    case 'world_import': case 'world_duplicate': case 'world_export': case 'world_activate': case 'world_delete': return 'worlds'
    case 'content_install': case 'content_remove': case 'content_toggle': return 'content'
    case 'loader_update': case 'minecraft_update': case 'runtime': case 'runtime_reinstall': return 'settings'
    default: return 'overview'
  }
}

export type AppNotice = {id:string; serverId:string} & (
  {type:'crash'} | {type:'task'; task:Progress}
)

// Session-only observations: reopening the app must not replay old outcomes.
export function createNoticeObserver() {
  const tasks=new Map<string,boolean>()
  const states=new Map<string,ServerState>()
  let sequence=0
  function remember<K,V>(map:Map<K,V>,key:K,value:V) {
    map.delete(key);map.set(key,value)
    if(map.size>128){const oldest=map.keys().next();if(!oldest.done)map.delete(oldest.value)}
  }
  return {
    task(task:Progress,background:boolean,source:'event'|'snapshot'='event'):AppNotice|null {
      const previous=tasks.get(task.taskId)
      const terminal=['completed','failed','cancelled'].includes(task.stage)
      if(previous===true)return null
      remember(tasks,task.taskId,terminal)
      if(!terminal||task.stage==='cancelled'||(source==='snapshot'&&previous===undefined))return null
      if(task.stage==='completed'&&!background)return null
      return {id:`task:${task.taskId}`,serverId:task.serverId,type:'task',task}
    },
    state(serverId:string,state:ServerState):AppNotice|null {
      const previous=states.get(serverId)
      remember(states,serverId,state)
      return state==='crashed'&&previous!=='crashed'
        ? {id:`crash:${++sequence}`,serverId,type:'crash'} : null
    },
  }
}
