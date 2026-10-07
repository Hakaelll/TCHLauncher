import {translatedMessage} from './locale.ts'
import {localizedStatic,t,getLanguage} from './locale.ts'
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import { api, type LogLine, type ServerState } from './api'
import { motion, reducedMotion } from './motion'

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
  logs: LogLine[];
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
  const [clearedAt,setClearedAt] = useState<string|null>(null)
  const [notice,setNotice] = useState<string|null>(null)
  const [exporting,setExporting] = useState(false)
  const windowRef = useRef<HTMLDivElement>(null)
  const automaticScroll=useRef(false)
  const scrollFrame=useRef(0)
  const pauseFollowing=()=>{cancelAnimationFrame(scrollFrame.current);automaticScroll.current=false;setFollowing(false)}
  useEffect(() => { setQuery(initialQuery); setLevel('all'); setCommand(''); setClearedAt(null); setFollowing(true); setNotice(null) },[serverId,initialQuery])
  const visible = useMemo(() => logs.filter(item => {
    if (clearedAt && (!item.timestamp?.includes('T') || item.timestamp <= clearedAt)) return false
    if (level !== 'all' && item.level !== level) return false
    return !query || item.line.toLocaleLowerCase().includes(query.toLocaleLowerCase())
  }),[logs,clearedAt,level,query])
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
    if(!acceptsCommands || busy || !command.trim()) return
    if (await onSend(command)) setCommand('')
  }
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(visible.map(item => `${clockTime(item.timestamp)}  ${item.level.toUpperCase()}  ${item.line}`).join('\n'))
      setNotice(t("{0} líneas copiadas.", visible.length))
    } catch { setNotice(t("No se pudo copiar. Selecciona las líneas y usa Ctrl+C.")) }
  }
  const exportFile = async () => {
    setExporting(true); setNotice(null)
    try {
      const destination = await api.exportLog(serverId)
      if (destination) setNotice(`Log exportado: ${destination}`)
    } catch (error) { setNotice(error instanceof Error ? error.message : String(error)) }
    finally { setExporting(false) }
  }
  return <section className="console-view">
    <div className="section-heading console-heading"><div><h2>{t("Consola")}</h2><p>{t("Registro del servidor. Las líneas se pueden buscar, copiar y exportar.")}</p></div><span className="console-count">{visible.length}  {t("líneas")}</span></div>
    {(historyLoading||historyError)&&<div className={`panel-read-state ${historyError?'failed':''}`} role={historyError?'alert':'status'}><span className="read-state-mark" aria-hidden="true">{historyError?'!':'…'}</span><div><strong>{historyError?t("Historial sin actualizar"):t("Cargando historial")}</strong><p>{historyError?historyError:t("La salida en directo seguirá apareciendo durante la lectura.")}</p></div>{historyError&&<button disabled={historyLoading} onClick={onReload}>{historyLoading?t("Consultando…"):t("Reintentar")}</button>}</div>}
    <div className="console-tools"><label className="search-label">{t("Buscar")}<input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder={t("Texto del log")}/></label><label className="console-level-select">{t("Nivel")}<select value={level} onChange={event => setLevel(event.target.value)}>{levels.map(([value,label]) => <option key={value} value={value}>{translatedMessage(label)}</option>)}</select></label><label className="quiet-checkbox"><input type="checkbox" checked={following} onChange={event => setFollowing(event.target.checked)}/>  {t("Seguir salida")}</label><div className="console-tool-actions"><button className="text-button" disabled={!visible.length} onClick={copy}>{t("Copiar vista")}</button><button className="text-button" disabled={exporting || !logs.length} onClick={exportFile}>{exporting ? t("Exportando…") : t("Exportar log")}</button><button className="text-button" disabled={!visible.length && !clearedAt} onClick={() => { if (clearedAt) setClearedAt(null); else setClearedAt(new Date().toISOString()); setNotice(null) }}>{clearedAt ? t("Mostrar historial") : t("Limpiar vista")}</button></div></div>
    <div className="log-window" ref={windowRef} role="log" aria-live="off" onWheel={pauseFollowing} onTouchStart={pauseFollowing} onPointerDown={pauseFollowing} onScroll={event => { const el = event.currentTarget; if (!automaticScroll.current && el.scrollHeight - el.scrollTop - el.clientHeight > 48) setFollowing(false) }}>
      {visible.length ? visible.map((item,index) => <div className={`log-line ${item.level}`} key={`${item.timestamp ?? 'legacy'}-${index}`}><time dateTime={item.timestamp?.includes('T') ? item.timestamp : undefined}>{clockTime(item.timestamp)}</time><span className="log-level">{item.level}</span><span className="log-message">{item.message}</span></div>) : <div className="log-empty">{clearedAt ? t("Vista limpia. Las nuevas líneas aparecerán aquí.") : query || level !== 'all' ? t("No hay líneas que coincidan con el filtro.") : historyError ? t("El historial no está disponible. Puedes reintentar su lectura.") : historyLoading ? t("Leyendo el historial del servidor…") : t("Aún no hay salida. Inicia el servidor para ver su actividad.")}</div>}
    </div>
    {notice && <p className="console-notice" role="status">{translatedMessage(notice)}</p>}
    <form autoComplete="off" className="command-form" onSubmit={submit}><label htmlFor="command">{t("Comando")}</label><input id="command" autoComplete="off" autoCorrect="off" spellCheck={false} value={command} onChange={event => setCommand(event.target.value)} placeholder={state === 'starting' ? t("Comando durante el inicio…") : acceptsCommands ? t("Escribe un comando…") : t("Inicia el servidor para enviar comandos")} disabled={!acceptsCommands} maxLength={512}/><button disabled={busy || !acceptsCommands || !command.trim()}>{t("Enviar")}</button></form>
  </section>
}
