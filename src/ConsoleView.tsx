import {translatedMessage} from './locale.ts'
import {localizedStatic,t,getLanguage} from './locale.ts'
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import { api, type ServerState } from './api'
import { motion, reducedMotion } from './motion'
import {rememberCommand,recallCommand,countLogLevels} from './consoleTools'
import {consoleBoundary,consoleLinesAfterClear,type ConsoleLine} from './consoleTimeline'

const levels = localizedStatic([
  ['all',t("Todos")], ['info',t("Información")], ['system',t("Sistema")], ['player',t("Jugadores")],
  ['chat','Chat'], ['command',t("Comandos")], ['warn',t("Avisos")], ['error',t("Errores")],
] as const)

function clockTime(timestamp: string | null) {
  if (!timestamp) return '—'
  if (!timestamp.includes('T')) return timestamp
  const date = new Date(timestamp)
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleTimeString(getLanguage()==='en'?'en-US':'es-ES', { hour:'2-digit', minute:'2-digit', second:'2-digit' })
}

export function ConsoleView({serverId,state,logs,busy,onSend,historyLoading,historyError,onReload,initialQuery=''}: {
  initialQuery?: string;
  serverId: string;
  state: ServerState;
  logs: ConsoleLine[];
  historyLoading:boolean;
  historyError:string|null;
  onReload:()=>Promise<void>;
  busy: boolean;
  onSend: (command: string) => Promise<boolean>;
}) {
  const acceptsCommands=state==='running' || state==='starting'
  const [query,setQuery] = useState(initialQuery)
  const [level,setLevel] = useState('all')
  const [command,setCommand] = useState('')
  const [following,setFollowing] = useState(true)
  const [clearedThrough,setClearedThrough] = useState<number|null>(null)
  const [notice,setNotice] = useState<string|null>(null)
  const [exporting,setExporting] = useState(false)
  const [sending,setSending]=useState(false)
  const [commandHistory,setCommandHistory]=useState<string[]>([])
  const [historyCursor,setHistoryCursor]=useState<number|null>(null)
  const historyDraft=useRef('')
  const scope=useRef({id:serverId,active:true,sending:false,exporting:false})
  const windowRef = useRef<HTMLDivElement>(null)
  const automaticScroll=useRef(false)
  const scrollFrame=useRef(0)
  const pauseFollowing=()=>{cancelAnimationFrame(scrollFrame.current);automaticScroll.current=false;setFollowing(false)}
  useEffect(() => { setQuery(initialQuery); setLevel('all'); setCommand(''); setClearedThrough(null); setFollowing(true); setNotice(null) },[serverId,initialQuery])
  useEffect(()=>{
    const current={id:serverId,active:true,sending:false,exporting:false};scope.current=current
    setCommandHistory([]);setHistoryCursor(null);historyDraft.current='';setSending(false);setExporting(false)
    return()=>{current.active=false}
  },[serverId])
  const currentScope=(value:typeof scope.current)=>value.active&&scope.current===value
  const searched = useMemo(() => {const search=query.toLocaleLowerCase();return consoleLinesAfterClear(logs,serverId,clearedThrough).filter(item=>!search||item.line.toLocaleLowerCase().includes(search))},[logs,clearedThrough,query,serverId])
  const visible=useMemo(()=>level==='all'?searched:searched.filter(item=>item.level===level),[searched,level])
  const counts=useMemo(()=>countLogLevels(searched),[searched])
  useEffect(() => {
    const element = windowRef.current
    if (!following || !element)return
    const start=element.scrollTop,target=Math.max(0,element.scrollHeight-element.clientHeight)
    if(reducedMotion() || target-start>element.clientHeight){element.scrollTop=target;return}
    automaticScroll.current=true
    const began=performance.now()
    const tick=(now:number)=>{
      if(reducedMotion()){element.scrollTop=target;automaticScroll.current=false;return}
      const progress=Math.min(1,(now-began)/motion.interaction)
      element.scrollTop=start+(target-start)*(1-(1-progress)**3)
      if(progress<1)scrollFrame.current=requestAnimationFrame(tick)
      else automaticScroll.current=false
    }
    scrollFrame.current=requestAnimationFrame(tick)
    return()=>{cancelAnimationFrame(scrollFrame.current);automaticScroll.current=false}
  },[visible,following])
  const submit = async (event: FormEvent) => {
    event.preventDefault()
    const current=scope.current,sent=command.trim()
    if(!acceptsCommands||busy||!sent||current.sending||current.id!==serverId)return
    current.sending=true;setSending(true)
    try{
      if(await onSend(sent)){
        if(!currentScope(current))return
        setCommandHistory(previous=>rememberCommand(previous,sent));setHistoryCursor(null);historyDraft.current=''
        setCommand(previous=>previous.trim()===sent?'':previous)
      }
    }catch(cause){if(currentScope(current))setNotice(cause instanceof Error?cause.message:String(cause))}
    finally{current.sending=false;if(currentScope(current))setSending(false)}
  }
  const copy = async () => {
    const current=scope.current
    try {
      await navigator.clipboard.writeText(visible.map(item => `${clockTime(item.timestamp)}  ${item.level.toUpperCase()}  ${item.line}`).join('\n'))
      if(currentScope(current))setNotice(t("{0} líneas copiadas.", visible.length))
    } catch {if(currentScope(current))setNotice(t("No se pudo copiar. Selecciona las líneas y usa Ctrl+C."))}
  }
  const exportFile = async () => {
    const current=scope.current
    if(current.exporting||current.id!==serverId)return
    current.exporting=true
    setExporting(true); setNotice(null)
    try {
      const destination = await api.exportLog(current.id)
      if(destination&&currentScope(current))setNotice(t("Log exportado: {0}",destination))
    } catch (error) {if(currentScope(current))setNotice(error instanceof Error ? error.message : String(error))}
    finally {current.exporting=false;if(currentScope(current))setExporting(false)}
  }
  return <section className="console-view">
    <div className="section-heading console-heading"><div><h2>{t("Consola")}</h2><p>{t("Registro del servidor. Las líneas se pueden buscar, copiar y exportar.")}</p></div><span className="console-count">{visible.length}  {t("líneas")}</span></div>
    {(historyLoading||historyError)&&<div className={`panel-read-state ${historyError?'failed':''}`} role={historyError?'alert':'status'}><span className="read-state-mark" aria-hidden="true">{historyError?'!':'…'}</span><div><strong>{historyError?t("Historial sin actualizar"):t("Cargando historial")}</strong><p>{historyError?historyError:t("La salida en directo seguirá apareciendo durante la lectura.")}</p></div>{historyError&&<button disabled={historyLoading} onClick={onReload}>{historyLoading?t("Consultando…"):t("Reintentar")}</button>}</div>}
    <div className="console-tools"><label className="search-label">{t("Buscar")}<input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder={t("Texto del log")}/></label><div className="console-tool-actions"><button className="text-button" disabled={!visible.length} onClick={copy}>{t("Copiar vista")}</button><button className="text-button" disabled={exporting || !logs.length} onClick={exportFile}>{exporting ? t("Exportando…") : t("Exportar log")}</button><button className="text-button" disabled={!visible.length && clearedThrough===null} onClick={() => { if (clearedThrough!==null) setClearedThrough(null); else setClearedThrough(consoleBoundary(logs,serverId)); setNotice(null) }}>{clearedThrough!==null ? t("Mostrar historial") : t("Limpiar vista")}</button></div></div>
    <div className="console-filters" role="group" aria-label={t("Filtrar categorías del log")}>{levels.map(([value,label])=><button key={value} className={`console-filter ${value}`} aria-pressed={level===value} onClick={()=>setLevel(value)}><span className="console-filter-mark" aria-hidden="true"/>{translatedMessage(label)}<span className="console-filter-count">{counts[value]??0}</span></button>)}</div>
    <div className="log-window" ref={windowRef} role="log" tabIndex={0} aria-label={t("Salida del servidor")} aria-live="off" onKeyDown={event=>{if(['ArrowUp','ArrowDown','PageUp','PageDown','Home','End',' '].includes(event.key))pauseFollowing()}} onWheel={pauseFollowing} onTouchStart={pauseFollowing} onPointerDown={pauseFollowing} onScroll={event => { const el = event.currentTarget; if (!automaticScroll.current && el.scrollHeight - el.scrollTop - el.clientHeight > 48) setFollowing(false) }}>
      {visible.length ? visible.map((item,index) => <div className={`log-line ${item.level}`} key={`${item.timestamp ?? 'legacy'}-${index}`}><time dateTime={item.timestamp?.includes('T') ? item.timestamp : undefined}>{clockTime(item.timestamp)}</time><span className="log-level">{item.level}</span><span className="log-message">{item.message}</span></div>) : <div className="log-empty">{clearedThrough!==null ? t("Vista limpia. Las nuevas líneas aparecerán aquí.") : query || level !== 'all' ? t("No hay líneas que coincidan con el filtro.") : historyError ? t("El historial no está disponible. Puedes reintentar su lectura.") : historyLoading ? t("Leyendo el historial del servidor…") : t("Aún no hay salida. Inicia el servidor para ver su actividad.")}</div>}
    </div>
    <div className={`console-follow ${following?'following':'paused'}`}><span><i aria-hidden="true"/>{following?t("Seguimiento activo"):t("Seguimiento pausado")}</span>{!following&&<small>{t("Las nuevas líneas siguen llegando.")}</small>}<button aria-pressed={following} onClick={()=>following?pauseFollowing():setFollowing(true)}>{following?t("Dejar de seguir"):t("Ir a las últimas líneas")}</button></div>
    {notice && <p className="console-notice" role="status">{translatedMessage(notice)}</p>}
    <form autoComplete="off" className="command-form" onSubmit={submit}><label htmlFor="command">{t("Comando")}</label><input id="command" autoComplete="off" autoCorrect="off" spellCheck={false} value={command} onChange={event => {setCommand(event.target.value);setHistoryCursor(null);historyDraft.current=''}} onKeyDown={event=>{if(event.nativeEvent.isComposing||event.ctrlKey||event.altKey||event.metaKey||event.shiftKey||!['ArrowUp','ArrowDown'].includes(event.key)||!commandHistory.length)return;event.preventDefault();const next=recallCommand(commandHistory,historyCursor,historyDraft.current,command,event.key==='ArrowUp'?'up':'down');historyDraft.current=next.draft;setHistoryCursor(next.cursor);setCommand(next.value)}} placeholder={state === 'starting' ? t("Comando durante el inicio…") : acceptsCommands ? t("Escribe un comando…") : t("Inicia el servidor para enviar comandos")} disabled={!acceptsCommands} maxLength={512}/><button disabled={busy || sending || !acceptsCommands || !command.trim()}>{sending?t("Enviando…"):t("Enviar")}</button></form>
    {acceptsCommands&&<p className="command-help">{t("Enter para enviar. Flechas para recuperar comandos de esta sesión.")}</p>}
  </section>
}
