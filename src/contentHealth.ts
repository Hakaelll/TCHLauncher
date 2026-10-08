import type {ContentCheck, InstalledContent, LogLine} from './api'

export type ContentSignal = {level:'error'|'warn'; message:string; timestamp:string|null; query:string}
const escape=(value:string)=>value.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')

type ContentDiagnostic = Readonly<{message:string;raw:string;level:'error'|'warn';timestamp:string|null}>

/** Parse severity/continuation rules once for the complete inventory, not once per JAR. */
export function prepareContentDiagnostics(logs:readonly LogLine[]):readonly ContentDiagnostic[] {
  const diagnostics:ContentDiagnostic[]=[]
  let block:'error'|'warn'|null=null,remaining=0
  for(const log of logs) {
    const message=log.message.slice(0,16384).replace(/\x1b\[[0-9;]*m/g,'')
    const raw=log.line.slice(0,16384).replace(/\x1b\[[0-9;]*m/g,'').replace(/^(?:>\s*)+(?=\[)/,'')
    const header=raw.match(/^(?:\[[^\]]*\]\s*){1,4}/)?.[0]??''
    const plainHeader=raw.match(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3} \S{1,128} (TRACE|DEBUG|INFO|WARN|WARNING|ERROR|SEVERE|FATAL)\s/)
    const prefix=raw.match(/^(ERROR|Error|WARN|WARNING):/)
    const declaredLevel=plainHeader?.[1]??prefix?.[1]??header.match(/\b(ERROR|WARN|WARNING|SEVERE|FATAL)\]/i)?.[1]
    const severity=!!declaredLevel&&/^(ERROR|WARN|WARNING|SEVERE|FATAL)$/i.test(declaredLevel)
    if(raw.startsWith('[')||plainHeader||prefix) {block=severity?(/^(ERROR|SEVERE|FATAL)$/i.test(declaredLevel!)?'error':'warn'):null;remaining=32}
    const continuation=!raw.startsWith('[')&&remaining-->0&&block&&/^\s*(?:-\s*Mod\b|Mod\b|Caused by:|at\s)/i.test(message)
    const level=continuation?block:severity?block:log.level
    if(!['error','warn'].includes(level??''))continue
    // Chat and player commands are never diagnostic evidence, even with a mislabeled level.
    if(/^\s*(?:\[Not Secure\]\s*)?</.test(message)||/^\w+ issued server command:/.test(message))continue
    // INFO text containing the word "error" alone must not color an installed mod red.
    if(!severity&&!continuation&&!/^(?:Caused by: |[\w.$]+(?:Exception|Error):|Mixin (?:apply|transformation).*failed)/i.test(message))continue
    diagnostics.push({message,raw,level:level as 'error'|'warn',timestamp:log.timestamp})
  }
  return diagnostics
}

/** An explicit reference associates a diagnostic with a file; it doesn't prove causation. */
export function contentSignalsFromDiagnostics(file:InstalledContent,check:ContentCheck|undefined,diagnostics:readonly ContentDiagnostic[]):ContentSignal[] {
  const signals:ContentSignal[]=[]
  let latestError:ContentSignal|undefined
  const ids=[...new Set([check?.descriptor.id,...check?.descriptor.ids??[]].filter((id):id is string=>!!id&&id.length<=64))]
  const names=[...ids,check?.descriptor.name,file.title].filter((name):name is string=>!!name&&name.length<=128&&!name.endsWith('.jar'))
  // Compile per file, not per line: live logs can contain 1,000 rows per inventory item.
  const filePattern=new RegExp(`(?<![a-z0-9_.-])${escape(file.filename)}(?![a-z0-9_.-])`,'i')
  const idPatterns=ids.map(id=>({id,logger:new RegExp(`(?:\\[${escape(id)}(?:/[^\\]]+)?\\]|from mod\\s+['\"]?${escape(id)}(?![a-z0-9_.-])|provided by\\s+['\"]?${escape(id)}(?![a-z0-9_.-]))`,'i'),named:new RegExp(`mod\\s+['\"][^'\"]+['\"]\\s+\\(${escape(id)}\\)`,'i')}))
  const namePatterns=names.map(name=>({name,pattern:new RegExp(`(?:(?:mod|plugin)\\s+|(?:while|when)\\s+(?:enabling|loading|disabling)\\s+)(?:['\"]${escape(name)}['\"]|${escape(name)}(?![a-z0-9_.-]))`,'i')}))
  for(const {message,raw,level,timestamp} of diagnostics) {
    let reference=filePattern.test(message)?file.filename:null
    for(const {id,logger,named} of idPatterns){
      if(!reference&&(logger.test(raw)||named.test(message)))reference=id
    }
    if(!reference)for(const {name,pattern} of namePatterns) {
      if(pattern.test(message)){reference=name;break}
    }
    if(!reference)continue
    const signal={level,message:message.slice(0,2000),timestamp,query:reference}
    const duplicate=signals.findIndex(previous=>previous.message===signal.message&&previous.level===level)
    if(duplicate>=0)signals.splice(duplicate,1)
    signals.push(signal)
    if(level==='error')latestError=signal
    if(signals.length>3)signals.shift()
  }
  if(latestError&&!signals.includes(latestError))signals[0]=latestError
  return signals.reverse()
}

/** Convenience entry for callers matching a single file. */
export function contentSignals(file:InstalledContent,check:ContentCheck|undefined,logs:readonly LogLine[]):ContentSignal[] {
  return contentSignalsFromDiagnostics(file,check,prepareContentDiagnostics(logs))
}

export function contentStatus(check:ContentCheck|undefined,signals:ContentSignal[]):'error'|'warning'|'observed'|'unknown' {
  if(signals.some(signal=>signal.level==='error'))return 'error'
  if(signals.length||check?.issues.length||check?.listedInLog===false)return 'warning'
  return check?.listedInLog===true?'observed':'unknown'
}
