import type {LogLine} from './api'

/** Session-only command recall; only acknowledged sends belong in this list. */
export function rememberCommand(history:string[],command:string):string[] {
  const value=command.trim()
  if(!value||history.at(-1)===value)return history
  return [...history,value].slice(-50)
}

export function recallCommand(history:string[],cursor:number|null,draft:string,current:string,direction:'up'|'down') {
  if(!history.length)return {cursor:null,draft:current,value:current}
  const saved=cursor===null?current:draft
  const next=direction==='up'?cursor===null?history.length-1:Math.max(0,cursor-1):cursor===null?null:cursor+1>=history.length?null:cursor+1
  return {cursor:next,draft:saved,value:next===null?saved:history[next]}
}

/** Counts use exactly the searched view, including genuine unknown categories. */
export function countLogLevels(lines:LogLine[]):Record<string,number> {
  const result:Record<string,number>={all:lines.length}
  for(const line of lines)result[line.level]=(result[line.level]??0)+1
  return result
}
