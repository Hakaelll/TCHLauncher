import type {StartupPhase} from './api'
export type StartupEvent={serverId:string;sessionId:string;phase:StartupPhase}
const phases:StartupPhase[]=['launching','loading_world','preparing_spawn','ready']
export function acceptStartupEvent(current:Record<string,StartupEvent>,event:StartupEvent):Record<string,StartupEvent>{
  const rank=phases.indexOf(event.phase)
  if(rank<0||!event.serverId||!event.sessionId)return current
  const previous=current[event.serverId]
  if(!previous||previous.sessionId!==event.sessionId){if(event.phase!=='launching')return current}
  else if(rank<=phases.indexOf(previous.phase))return current
  return {...current,[event.serverId]:event}
}
