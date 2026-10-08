import type {LogLine} from './api'

// Client-only identity: positive live arrivals, negative disk history. Native logs stay verbatim.
export type ConsoleLine=LogLine & {arrival:number}
const key=(line:LogLine)=>JSON.stringify([line.serverId,line.timestamp,line.line])

export function appendConsoleLines(current:ConsoleLine[],incoming:ConsoleLine[],capacity=1000):ConsoleLine[] {
  return [...current.slice(Math.max(0,current.length+incoming.length-capacity)),...incoming.slice(-capacity)]
}

/** Reconcile occurrence counts, retaining the arrival identity of live lines. */
export function mergeConsoleHistory(current:ConsoleLine[],history:LogLine[],serverId:string,capacity=1000):ConsoleLine[] {
  const live=current.filter(line=>line.serverId===serverId).slice(-capacity)
  const saved=history.filter(line=>line.serverId===serverId).slice(-capacity)
  const buckets=new Map<string,{lines:ConsoleLine[];used:number}>()
  for(const line of live){const signature=key(line);let bucket=buckets.get(signature);if(!bucket){bucket={lines:[],used:0};buckets.set(signature,bucket)}bucket.lines.push(line)}
  let historyArrival=live.reduce((lowest,line)=>Math.min(lowest,line.arrival),0)
  const consumed=new Set<ConsoleLine>()
  const merged=saved.map(line=>{
    const bucket=buckets.get(key(line)),existing=bucket?.lines[bucket.used]
    if(existing){bucket!.used++;consumed.add(existing);return existing}
    return {...line,arrival:--historyArrival}
  })
  return [...merged,...live.filter(line=>!consumed.has(line))].slice(-capacity)
}

export function consoleBoundary(lines:ConsoleLine[],serverId:string):number {
  return lines.reduce((latest,line)=>line.serverId===serverId?Math.max(latest,line.arrival):latest,0)
}

export function consoleLinesAfterClear(lines:ConsoleLine[],serverId:string,boundary:number|null):ConsoleLine[] {
  return lines.filter(line=>line.serverId===serverId&&(boundary===null||line.arrival>boundary))
}
