import type {LogLine,ServerState} from './api'

export type SaveState={phase:'unknown'|'saving'|'confirmed'|'failed'|'unconfirmed';lastConfirmed:LogLine|null;latest:LogLine|null}

/** Only observed whole-save completion updates the confirmed time. */
export function saveState(logs:LogLine[],serverState?:ServerState):SaveState {
  let state:SaveState={phase:'unknown',lastConfirmed:null,latest:null}
  for(const item of logs) {
    switch(item.event) {
      case 'world_load':case 'startup':
        state={phase:'unknown',lastConfirmed:null,latest:null};break
      case 'save_started':
        state={...state,phase:'saving',latest:item};break
      case 'save':
        state={phase:'confirmed',lastConfirmed:item,latest:item};break
      case 'save_failed':
        state={...state,phase:'failed',latest:item};break
      case 'crash':case 'exit':
        if(state.phase==='saving')state={...state,phase:'unconfirmed'}
        break
    }
  }
  if(state.phase==='saving'&&serverState&&['stopped','crashed','error','installing'].includes(serverState))state={...state,phase:'unconfirmed'}
  return state
}
