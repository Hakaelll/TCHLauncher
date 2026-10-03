import type {ServerState} from './api'

export type StartupCheck = {kind:'ready'|'halted'|'unverified';state?:ServerState;error?:string}
type Options = {
  readState:()=>Promise<ServerState|null>
  wait:()=>Promise<void>
  now:()=>number
  timeoutMs:number
  signal?:AbortSignal
}

// Only the native running state, emitted after Minecraft's startup log,
// establishes readiness. A launched Java process alone is not sufficient.
export async function verifyStartup({readState,wait,now,timeoutMs,signal}:Options):Promise<StartupCheck> {
  const deadline=now()+timeoutMs
  while(!signal?.aborted && now()<deadline) {
    let state:ServerState|null
    try {state=await readState()} catch(error) {return {kind:'unverified',error:String(error)}}
    if(signal?.aborted)return {kind:'unverified'}
    if(state==='running')return {kind:'ready',state}
    if(state==='stopped'||state==='stopping'||state==='crashed'||state==='error')return {kind:'halted',state}
    if(state===null)return {kind:'unverified',error:'El servidor ya no está registrado.'}
    await wait()
  }
  return {kind:'unverified'}
}
