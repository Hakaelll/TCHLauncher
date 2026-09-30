import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import { api, type LogLine, type ServerState } from './api'
import { motion, reducedMotion } from './motion'

const levels = [
  ['all','Todos'], ['info','Información'], ['system','Sistema'], ['player','Jugadores'],
  ['chat','Chat'], ['command','Comandos'], ['warn','Avisos'], ['error','Errores'],
] as const

function clockTime(timestamp: string | null) {
  if (!timestamp) return '—'
  if (!timestamp.includes('T')) return timestamp
  const date = new Date(timestamp)
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleTimeString('es-ES', { hour:'2-digit', minute:'2-digit', second:'2-digit' })
}

export function ConsoleView({serverId,state,logs,busy,onSend}: {
  serverId: string;
  state: ServerState;
  logs: LogLine[];
  busy: boolean;
  onSend: (command: string) => Promise<boolean>;
}) {
  const [query,setQuery] = useState('')
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
  useEffect(() => { setQuery(''); setLevel('all'); setCommand(''); setClearedAt(null); setFollowing(true); setNotice(null) },[serverId])
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
    if (await onSend(command)) setCommand('')
  }
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(visible.map(item => `${clockTime(item.timestamp)}  ${item.level.toUpperCase()}  ${item.line}`).join('\n'))
      setNotice(`${visible.length} líneas copiadas.`)
    } catch { setNotice('No se pudo copiar. Selecciona las líneas y usa Ctrl+C.') }
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
    <div className="section-heading console-heading"><div><h2>Consola</h2><p>Registro del servidor. Las líneas se pueden buscar, copiar y exportar.</p></div><span className="console-count">{visible.length} líneas</span></div>
    <div className="console-tools"><label className="search-label">Buscar<input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Texto del log"/></label><label className="console-level-select">Nivel<select value={level} onChange={event => setLevel(event.target.value)}>{levels.map(([value,label]) => <option key={value} value={value}>{label}</option>)}</select></label><label className="quiet-checkbox"><input type="checkbox" checked={following} onChange={event => setFollowing(event.target.checked)}/> Seguir salida</label><div className="console-tool-actions"><button className="text-button" disabled={!visible.length} onClick={copy}>Copiar vista</button><button className="text-button" disabled={exporting || !logs.length} onClick={exportFile}>{exporting ? 'Exportando…' : 'Exportar log'}</button><button className="text-button" disabled={!visible.length && !clearedAt} onClick={() => { if (clearedAt) setClearedAt(null); else setClearedAt(new Date().toISOString()); setNotice(null) }}>{clearedAt ? 'Mostrar historial' : 'Limpiar vista'}</button></div></div>
    <div className="log-window" ref={windowRef} role="log" aria-live="off" onWheel={pauseFollowing} onTouchStart={pauseFollowing} onPointerDown={pauseFollowing} onScroll={event => { const el = event.currentTarget; if (!automaticScroll.current && el.scrollHeight - el.scrollTop - el.clientHeight > 48) setFollowing(false) }}>
      {visible.length ? visible.map((item,index) => <div className={`log-line ${item.level}`} key={`${item.timestamp ?? 'legacy'}-${index}`}><time dateTime={item.timestamp?.includes('T') ? item.timestamp : undefined}>{clockTime(item.timestamp)}</time><span className="log-level">{item.level}</span><span className="log-message">{item.message}</span></div>) : <div className="log-empty">{clearedAt ? 'Vista limpia. Las nuevas líneas aparecerán aquí.' : query || level !== 'all' ? 'No hay líneas que coincidan con el filtro.' : 'Aún no hay salida. Inicia el servidor para ver su actividad.'}</div>}
    </div>
    {notice && <p className="console-notice" role="status">{notice}</p>}
    <form autoComplete="off" className="command-form" onSubmit={submit}><label htmlFor="command">Comando</label><input id="command" autoComplete="off" autoCorrect="off" spellCheck={false} value={command} onChange={event => setCommand(event.target.value)} placeholder={state === 'running' ? 'Escribe un comando…' : 'Inicia el servidor para enviar comandos'} disabled={state !== 'running'} maxLength={512}/><button disabled={busy || state !== 'running' || !command.trim()}>Enviar</button></form>
  </section>
}
