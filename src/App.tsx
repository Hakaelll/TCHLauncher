import { useCallback, useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '@tauri-apps/api/core'
import { api, type Backup, type CreateInput, type ImportInput, type ImportInspection, type LogLine, type Metrics, type PlayersSnapshot, type Progress, type Server, type ServerState, type Version } from './api'
import { PlayersView } from './PlayersView'
import { SliderField } from './SliderField'
import { ConsoleView } from './ConsoleView'

type View = 'overview' | 'players' | 'console' | 'backups' | 'settings'
const labels: Record<ServerState, string> = {
  stopped: 'Detenido', installing: 'Instalando', starting: 'Iniciando',
  running: 'En ejecución', stopping: 'Deteniendo', crashed: 'Se ha cerrado inesperadamente', error: 'Error',
}
const stateText = (state: string) => labels[state as ServerState] || state
const fmtBytes = (n?: number | null) => n == null ? '—' : n < 1024 * 1024 ? `${Math.round(n / 1024)} KB` : n < 1024 * 1024 * 1024 ? `${(n / (1024 * 1024)).toFixed(1)} MB` : `${(n / (1024 * 1024 * 1024)).toFixed(1)} GB`
const fmtUptime = (n?: number | null) => n == null ? '—' : `${Math.floor(n / 3600)} h ${Math.floor(n % 3600 / 60)} min`
const errorText = (e: unknown) => e instanceof Error ? e.message : String(e)

function Icon({name, size = 18}: {name: 'plus' | 'folder' | 'arrow' | 'back' | 'server' | 'check' | 'collapse' | 'expand'; size?: number}) {
  const paths = {
    plus: <path d="M12 5v14M5 12h14"/>,
    folder: <path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>,
    arrow: <><path d="M4 12h15"/><path d="m13 6 6 6-6 6"/></>,
    back: <><path d="M20 12H5"/><path d="m11 6-6 6 6 6"/></>,
    server: <><rect x="4" y="4" width="16" height="7" rx="2"/><rect x="4" y="13" width="16" height="7" rx="2"/><path d="M8 7.5h.01M8 16.5h.01"/></>,
    check: <path d="m5 12 4.5 4.5L19 7"/>,
    collapse: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16m8-11-3 3 3 3"/></>,
    expand: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16m5-11 3 3-3 3"/></>,
  }
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{paths[name]}</svg>
}

function ServerArtwork({server}: {server: Server}) {
  return server.imageData
    ? <img className="server-artwork" src={server.imageData} alt=""/>
    : <span className="server-artwork default" aria-hidden="true"><Icon name="server" size={20}/></span>
}

function App() {
  const [servers, setServers] = useState<Server[]>([])
  const [sidebarCollapsed,setSidebarCollapsed] = useState(() => {
    try { return window.localStorage.getItem('tch-sidebar-collapsed') === 'true' } catch { return false }
  })
  const [sidebarTooltip,setSidebarTooltip] = useState<{name:string;subtitle:string;top:number;left:number}|null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const selectedRef = useRef<string | null>(null)
  const [view, setView] = useState<View>('overview')
  const [creating, setCreating] = useState(false)
  const [importing, setImporting] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [progress, setProgress] = useState<Progress | null>(null)
  const [metrics, setMetrics] = useState<Metrics | null>(null)
  const [players, setPlayers] = useState<PlayersSnapshot | null>(null)
  const [logs, setLogs] = useState<LogLine[]>([])
  const [backups, setBackups] = useState<Backup[]>([])
  const [props, setProps] = useState<Record<string, string>>({})
  const [eula, setEula] = useState(false)
  const [memory, setMemory] = useState({ min: 1024, max: 2048 })
  const [storageTarget, setStorageTarget] = useState(20)
  const [systemMemory, setSystemMemory] = useState(0)
  const [closing, setClosing] = useState(false)
  const [closingBusy, setClosingBusy] = useState(false)
  const selected = servers.find(server => server.id === selectedId) ?? null

  useEffect(() => {
    try { window.localStorage.setItem('tch-sidebar-collapsed',String(sidebarCollapsed)) } catch { /* La preferencia no impide usar la barra. */ }
    setSidebarTooltip(null)
  },[sidebarCollapsed])
  const showSidebarTooltip = (server: Server, button: HTMLButtonElement) => {
    if (!sidebarCollapsed) return
    const rect = button.getBoundingClientRect()
    setSidebarTooltip({
      name:server.name,
      subtitle:`${server.platform === 'paper' ? 'Paper' : 'Vanilla'} · ${server.version} · ${stateText(server.state)}`,
      top:Math.max(12,Math.min(window.innerHeight - 72,rect.top + rect.height / 2 - 28)),
      left:rect.right + 12,
    })
  }

  const refresh = useCallback(async () => {
    const items = await api.listServers()
    setServers(items)
    setSelectedId(current => current && items.some(s => s.id === current) ? current : items[0]?.id ?? null)
  }, [])

  useEffect(() => {
    if (!isTauri()) return
    refresh().catch(e => setError(errorText(e)))
    api.systemMemory().then(setSystemMemory).catch(() => {})
    const listeners = [
      listen<{ serverId: string; state: ServerState }>('server-state', event => {
        setServers(items => items.map(item => item.id === event.payload.serverId ? { ...item, state: event.payload.state } : item))
        if (event.payload.state === 'stopped' || event.payload.state === 'crashed' || event.payload.state === 'error') setProgress(null)
      }),
      listen<Progress>('install-progress', event => setProgress(event.payload)),
      listen<LogLine>('server-log', event => {
        if (event.payload.serverId === selectedRef.current) setLogs(lines => [...lines.slice(-999), event.payload])
      }),
      listen('close-with-servers', () => setClosing(true)),
    ]
    return () => { listeners.forEach(p => p.then(unlisten => unlisten()).catch(() => {})) }
  }, [refresh])

  useEffect(() => { selectedRef.current = selectedId }, [selectedId])

  useEffect(() => {
    if (!selectedId) return
    setMetrics(null); setPlayers(null); setLogs([]); setBackups([]); setProps({})
    Promise.all([api.metrics(selectedId), api.log(selectedId), api.backups(selectedId), api.properties(selectedId), api.eulaAccepted(selectedId)])
      .then(([m, log, b, p, accepted]) => {
        setMetrics(m); setLogs(log);
        setBackups(b); setProps(p); setEula(accepted)
      }).catch(e => setError(errorText(e)))
    api.players(selectedId).then(setPlayers).catch(e => setError(errorText(e)))
  }, [selectedId])

  useEffect(() => {
    if (!selected) return
    setMemory({ min: selected.memoryMinMb, max: selected.memoryMaxMb })
    setStorageTarget(selected.storageTargetGb)
  }, [selected?.id, selected?.memoryMinMb, selected?.memoryMaxMb, selected?.storageTargetGb])

  useEffect(() => {
    if (!selectedId) return
    const timer = window.setInterval(() => {
      api.metrics(selectedId).then(setMetrics).catch(() => {})
      api.players(selectedId).then(setPlayers).catch(() => {})
    }, 5000)
    return () => window.clearInterval(timer)
  }, [selectedId])

  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key.toLowerCase() === 'n') { event.preventDefault(); setImporting(false); setCreating(true) }
    }
    window.addEventListener('keydown',shortcut)
    return () => window.removeEventListener('keydown',shortcut)
  }, [])

  const act = async (fn: () => Promise<unknown>) => {
    setError(null); setBusy(true)
    try { await fn(); await refresh(); return true }
    catch (e) { setError(errorText(e)) }
    finally { setBusy(false) }
    return false
  }

  const create = async (input: CreateInput) => {
    await act(async () => {
      const server = await api.create(input)
      setCreating(false); setSelectedId(server.id); setView('overview')
      await refresh()
      await api.install(server.id)
      await refresh()
    })
  }

  const importExisting = async (input: ImportInput) => {
    await act(async () => {
      const server = await api.importServer(input)
      setImporting(false); setSelectedId(server.id); setView('overview')
      await refresh()
      await api.install(server.id)
      await refresh()
    })
  }

  const selectedName = selected?.name ?? 'Servidores'
  return <div className={`shell ${sidebarCollapsed ? 'sidebar-collapsed' : ''}`}>
    <aside className="sidebar" aria-label="Servidores">
      <div className="sidebar-top"><div className="brand-mark" aria-hidden="true"><Icon name="server" size={18}/></div><div className="wordmark">TCHLauncher</div><button className="sidebar-toggle icon-button" aria-label={sidebarCollapsed ? 'Ampliar barra lateral' : 'Reducir barra lateral'} title={sidebarCollapsed ? 'Ampliar barra lateral' : 'Reducir barra lateral'} aria-expanded={!sidebarCollapsed} onClick={() => setSidebarCollapsed(value => !value)}><Icon name={sidebarCollapsed ? 'expand' : 'collapse'} size={18}/></button></div>
      <div className="sidebar-label">Mis servidores <span>{servers.length}</span></div>
      <nav className="server-list" aria-label="Lista de servidores" onScroll={() => setSidebarTooltip(null)}>
        {servers.map(server => <button key={server.id} className={`server-row ${selectedId === server.id && !creating && !importing ? 'active' : ''}`} onClick={() => { setSidebarTooltip(null); setSelectedId(server.id); setCreating(false); setImporting(false); setView('overview') }} onMouseEnter={event => showSidebarTooltip(server,event.currentTarget)} onMouseLeave={() => setSidebarTooltip(null)} onFocus={event => showSidebarTooltip(server,event.currentTarget)} onBlur={() => setSidebarTooltip(null)} aria-label={sidebarCollapsed ? `${server.name}, ${server.platform === 'paper' ? 'Paper' : 'Vanilla'} ${server.version}, ${stateText(server.state)}` : undefined} aria-current={selectedId === server.id && !creating && !importing ? 'page' : undefined}>
          <span className="server-artwork-wrap"><ServerArtwork server={server}/><span className={`state-dot ${server.state}`} aria-hidden="true"/></span><span className="server-row-name">{server.name}<small>{server.platform === 'paper' ? 'Paper' : 'Vanilla'} · {server.version}</small></span>
        </button>)}
      </nav>
      <div className="sidebar-actions"><button className="new-server-link" aria-label="Nuevo servidor" title={sidebarCollapsed ? 'Nuevo servidor' : undefined} onClick={() => { setImporting(false); setCreating(true) }}><Icon name="plus" size={17}/><span>Nuevo servidor</span></button>
      <button className="new-server-link" aria-label="Importar servidor" title={sidebarCollapsed ? 'Importar servidor' : undefined} onClick={() => { setCreating(false); setImporting(true) }}><Icon name="folder" size={17}/><span>Importar servidor</span></button></div>
      <div className="sidebar-footer"><span>Servidores locales</span><span>v0.2</span></div>
    </aside>

    <main className="main">
      {error && <div className="error-banner" role="alert"><div><strong>No se pudo completar la acción</strong><p>{error}</p></div><button onClick={() => setError(null)} aria-label="Cerrar error">×</button></div>}
      {creating ? <CreateView onCancel={() => setCreating(false)} onCreate={create} busy={busy} systemMemory={systemMemory}/>
      : importing ? <ImportView onCancel={() => setImporting(false)} onImport={importExisting} busy={busy}/>
      : !selected ? <div className="empty-state"><div className="empty-intro"><div className="empty-symbol"><Icon name="server" size={31}/></div><h1>Prepara tu servidor.</h1><p>Crea un servidor nuevo o incorpora uno que ya tengas. TCHLauncher se ocupa de Java, los archivos y el proceso; tus mundos se quedan en este ordenador.</p></div><div className="empty-choices"><button className="empty-choice primary" onClick={() => setCreating(true)}><span className="empty-choice-icon"><Icon name="plus" size={25}/></span><span><strong>Crear servidor</strong><small>Elige versión, tipo y capacidad.</small></span><Icon name="arrow" size={21}/></button><button className="empty-choice" onClick={() => setImporting(true)}><span className="empty-choice-icon"><Icon name="folder" size={25}/></span><span><strong>Importar una carpeta</strong><small>Usa tus archivos actuales o cópialos a la biblioteca.</small></span><Icon name="arrow" size={21}/></button></div><p className="empty-footnote">Vanilla y Paper · Servidores locales · Datos bajo tu control</p></div>
      : <>
        <header className="page-head"><div className="page-identity"><ServerArtwork server={selected}/><div><div className="page-context">{selected.platform === 'paper' ? 'Paper' : 'Vanilla'} <span>/</span> Minecraft {selected.version}</div><h1>{selectedName}</h1></div></div><div className="head-actions"><span className={`status-label ${selected.state}`}><span className={`state-dot ${selected.state}`} aria-hidden="true"/>{stateText(selected.state)}</span>{selected.jarPath && selected.javaPath ? selected.state === 'running' ? <button className="primary-button" disabled={busy} onClick={() => act(() => api.stop(selected.id))}>Detener</button> : selected.state === 'stopped' || selected.state === 'crashed' || selected.state === 'error' ? <button className="primary-button" disabled={busy || !eula} onClick={() => act(() => api.start(selected.id))}>Iniciar servidor <Icon name="arrow" size={15}/></button> : null : <button className="primary-button" disabled={busy} onClick={() => act(() => api.install(selected.id))}>{selected.jarPath ? 'Preparar Java' : 'Instalar'} <Icon name="arrow" size={15}/></button>}</div></header>
        <nav className="tabs" aria-label="Secciones del servidor">{(['overview','players','console','backups','settings'] as View[]).map(item => <button key={item} aria-current={view === item ? 'page' : undefined} className={view === item ? 'active' : ''} onClick={() => setView(item)}>{ {overview:'Resumen',players:'Jugadores',console:'Consola',backups:'Copias',settings:'Configuración'}[item] }</button>)}</nav>
        <div className="content">
          {progress?.serverId === selected.id && <div className="progress-block" role="status"><div><strong>{progress.label}</strong><span>{progress.total ? `${Math.round(progress.received / progress.total * 100)} %` : fmtBytes(progress.received)}</span></div>{progress.total ? <progress value={progress.received} max={progress.total}/> : <progress/>}</div>}
          {view === 'overview' && <Overview server={selected} metrics={metrics} players={players} logs={logs} backups={backups} port={props['server-port'] || '25565'} eula={eula} onAccept={() => act(async () => { await api.acceptEula(selected.id); setEula(true) })} onInstall={() => act(() => api.install(selected.id))} onConsole={() => setView('console')}/>} 
          {view === 'players' && <PlayersView server={selected} players={players} busy={busy} onRefresh={async () => { try { setPlayers(await api.players(selected.id)) } catch(e) { setError(errorText(e)) } }} onAction={(action,target,reason) => act(() => api.managePlayer(selected.id,action,target,reason))} onToggleWhitelist={() => act(async () => { if (selected.state === 'running') await api.managePlayer(selected.id,players?.whitelistEnabled ? 'whitelist_off' : 'whitelist_on'); else { const values = { ...await api.properties(selected.id),'white-list':String(!players?.whitelistEnabled) }; await api.saveProperties(selected.id,values); setProps(values) } setPlayers(await api.players(selected.id)) })} onImportWhitelist={names => act(() => api.importWhitelist(selected.id,names))}/>}
          {view === 'console' && <ConsoleView serverId={selected.id} state={selected.state} logs={logs} busy={busy} onSend={command => act(() => api.send(selected.id,command))}/> }
          {view === 'backups' && <section><div className="section-heading"><div><h2>Copias de seguridad</h2><p>Se guardan en el almacenamiento local de la aplicación.</p></div><button disabled={busy || !['stopped','crashed','error'].includes(selected.state)} onClick={() => act(async () => { await api.createBackup(selected.id); setBackups(await api.backups(selected.id)) })}>Crear copia</button></div>{!['stopped','crashed','error'].includes(selected.state) && <p className="inline-note">Detén el servidor para crear una copia consistente.</p>}{backups.length ? <div className="table"><div className="table-head"><span>Fecha</span><span>Tamaño</span><span>Acción</span></div>{backups.map(backup => <div className="table-row" key={backup.id}><span>{new Date(backup.createdAt).toLocaleString()}</span><span>{fmtBytes(backup.sizeBytes)}</span><button className="text-button" disabled={busy || !['stopped','crashed','error'].includes(selected.state)} onClick={() => { if (window.confirm('Se guardará una copia del estado actual y después se restaurará esta versión. ¿Continuar?')) act(() => api.restoreBackup(selected.id,backup.id)) }}>Restaurar</button></div>)}</div> : <div className="empty-inline">Aún no hay copias de este servidor.</div>}</section>}
          {view === 'settings' && <Settings server={selected} props={props} setProps={setProps} memory={memory} setMemory={setMemory} storageTarget={storageTarget} setStorageTarget={setStorageTarget} systemMemory={systemMemory} busy={busy} onSave={() => act(async () => { await api.saveProperties(selected.id,props); await api.saveMemory(selected.id,memory.min,memory.max); await api.saveStorageTarget(selected.id,storageTarget); setProps(await api.properties(selected.id)) })} onPickImage={() => act(async () => { const image = await api.pickServerImage(); if (image) await api.setServerImage(selected.id,image.path) })} onClearImage={() => act(() => api.clearServerImage(selected.id))}/>} 
        </div>
      </>}
    </main>
    {sidebarCollapsed && sidebarTooltip && createPortal(<div className="sidebar-tooltip" role="tooltip" style={{top:sidebarTooltip.top,left:sidebarTooltip.left}}><strong>{sidebarTooltip.name}</strong><span>{sidebarTooltip.subtitle}</span></div>,document.body)}
    {closing && <div className="modal-scrim"><div className="close-dialog" role="dialog" aria-modal="true" aria-labelledby="close-title"><div className="eyebrow">Servidores activos</div><h2 id="close-title">¿Qué quieres hacer al cerrar?</h2><p>Hay procesos Minecraft ejecutándose en este equipo.</p><div className="close-options"><button disabled={closingBusy} onClick={() => { setError(null); api.hideWindow().then(() => setClosing(false)).catch(e => setError(errorText(e))) }}>Mantener en segundo plano</button><button disabled={closingBusy} onClick={() => { setClosingBusy(true); api.stopAllAndExit().catch(e => { setClosingBusy(false); setError(errorText(e)); setClosing(false) }) }}>{closingBusy ? 'Deteniendo servidores…' : 'Detener y salir'}</button><button autoFocus disabled={closingBusy} onClick={() => setClosing(false)}>Cancelar</button></div></div></div>}
  </div>
}

const eventLabels: Record<string,string> = {
  startup:'Servidor iniciado', shutdown:'Servidor deteniéndose', join:'Jugador conectado',
  leave:'Jugador desconectado', save:'Mundo guardado', lag:'Retraso del servidor',
  warning:'Aviso', error:'Error', exception:'Excepción', plugin_error:'Error de plugin',
  mod_error:'Error de mod', crash:'Cierre inesperado', exit:'Proceso finalizado',
}
const eventTime = (timestamp: string | null) => {
  if (!timestamp) return '—'
  if (!timestamp.includes('T')) return timestamp
  const date = new Date(timestamp)
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleTimeString('es-ES',{hour:'2-digit',minute:'2-digit'})
}

function Overview({server,metrics,players,logs,backups,port,eula,onAccept,onInstall,onConsole}: {
  server: Server; metrics: Metrics | null; players: PlayersSnapshot | null; logs: LogLine[]; backups: Backup[];
  port: string; eula: boolean; onAccept: () => void; onInstall: () => void; onConsole: () => void;
}) {
  const storagePercent = metrics ? Math.round(metrics.sizeBytes / (server.storageTargetGb * 1024 * 1024 * 1024) * 100) : 0
  const memoryPercent = metrics?.memoryBytes == null ? 0 : Math.round(metrics.memoryBytes / (server.memoryMaxMb * 1024 * 1024) * 100)
  const recentEvents = logs.filter(item => item.event && eventLabels[item.event]).slice(-4).reverse()
  const description = server.state === 'running' ? 'Disponible para jugadores en tu red local.'
    : server.jarPath && server.javaPath ? 'Preparado para iniciar en este equipo.'
    : 'Prepara los archivos y Java para ponerlo en marcha.'
  return <div className="overview">
    <section className={`server-stage ${server.state}`} aria-label="Estado del servidor">
      <div className="stage-main"><div className="stage-art"><ServerArtwork server={server}/></div><div className="stage-copy"><div className="stage-type">{server.platform === 'paper' ? 'Paper' : 'Vanilla'} <span>·</span> Minecraft {server.version}</div><h2><span className={`state-dot large ${server.state}`} aria-hidden="true"/>{stateText(server.state)}</h2><p>{description}</p></div></div>
      <div className="stage-stats"><div><span>Jugadores</span><strong>{players ? `${players.online.length} / ${players.maxPlayers}` : '—'}</strong></div><div><span>Tiempo activo</span><strong>{fmtUptime(metrics?.uptimeSeconds)}</strong></div><div><span>Memoria</span><strong>{fmtBytes(metrics?.memoryBytes)}</strong></div><div><span>CPU</span><strong>{metrics?.cpuPercent == null ? '—' : `${metrics.cpuPercent.toFixed(1)} %`}</strong></div></div>
    </section>
    {(!server.jarPath || !server.javaPath) && <div className="callout"><div><strong>Instalación pendiente</strong><p>{server.jarPath ? 'Prepararemos Java sin reemplazar el archivo .jar existente.' : `Descargaremos Minecraft ${server.version} y Java en las carpetas de la aplicación.`}</p></div><button onClick={onInstall}>{server.jarPath ? 'Preparar Java' : 'Instalar ahora'}</button></div>}
    {server.jarPath && !eula && <div className="callout"><div><strong>Antes del primer inicio</strong><p>Lee la <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">EULA de Minecraft</a>. Solo tú puedes aceptarla.</p></div><button onClick={onAccept}>He leído y acepto</button></div>}
    <div className="overview-grid"><section className="resource-section"><div className="section-heading"><div><h2>Recursos</h2><p>Uso real de este proceso y sus archivos.</p></div></div>
      <div className="resource-meter"><div><span>Memoria</span><strong>{metrics?.memoryBytes == null ? '—' : `${fmtBytes(metrics.memoryBytes)} / ${fmtBytes(server.memoryMaxMb * 1024 * 1024)}`}</strong></div><progress aria-label="Memoria del proceso respecto al máximo configurado" value={Math.min(memoryPercent,100)} max={100}/></div>
      <div className="resource-meter"><div><span>Espacio del servidor</span><strong>{metrics ? `${fmtBytes(metrics.sizeBytes)} / ${server.storageTargetGb} GB` : 'Calculando…'}</strong></div><progress aria-label="Espacio ocupado respecto al objetivo" value={Math.min(storagePercent,100)} max={100}/><p>{storagePercent >= 100 ? 'Supera el objetivo de aviso. Revisa mundos y logs.' : 'Objetivo de aviso; no limita el disco.'}</p></div>
      <dl className="resource-facts"><div><dt>PID</dt><dd>{metrics?.pid ?? '—'}</dd></div><div><dt>RAM asignada</dt><dd>{fmtBytes(server.memoryMinMb * 1024 * 1024)} – {fmtBytes(server.memoryMaxMb * 1024 * 1024)}</dd></div></dl>
    </section><section className="connection-section"><div className="section-heading"><div><h2>Conexión</h2><p>Datos para entrar desde este equipo.</p></div></div><div className="connection-address"><span>Dirección local</span><strong>localhost:{port}</strong></div><dl className="data-list"><div><dt>Versión</dt><dd>Minecraft {server.version}</dd></div><div><dt>Tipo</dt><dd>{server.platform === 'paper' ? 'Paper' : 'Vanilla'}</dd></div><div><dt>Java</dt><dd>{server.javaPath ? 'Gestionado por la aplicación' : 'Pendiente'}</dd></div></dl><p className="muted-note">Para otros equipos de la red, usa la IP local de este ordenador y el mismo puerto.</p></section></div>
    <div className="overview-lower"><section className="recent-activity"><div className="section-heading"><div><h2>Actividad reciente</h2><p>Eventos detectados en la salida del servidor.</p></div><button className="text-button" onClick={onConsole}>Ver consola <Icon name="arrow" size={15}/></button></div>{recentEvents.length ? <div className="event-list">{recentEvents.map((item,index) => <div className="event-row" key={`${item.timestamp ?? 'legacy'}-${index}`}><span className={`event-mark ${item.level}`} aria-hidden="true"/><div><strong>{eventLabels[item.event!]}</strong><span>{item.message}</span></div><time>{eventTime(item.timestamp)}</time></div>)}</div> : <p className="empty-inline">Los eventos aparecerán aquí cuando el servidor produzca actividad.</p>}</section><section className="recent-backups"><div className="section-heading"><div><h2>Copias recientes</h2><p>Guardadas en este ordenador.</p></div></div>{backups.length ? <div className="backup-mini-list">{backups.slice(0,3).map(backup => <div key={backup.id}><strong>{new Date(backup.createdAt).toLocaleString('es-ES')}</strong><span>{fmtBytes(backup.sizeBytes)}</span></div>)}</div> : <p className="empty-inline">Todavía no hay copias de seguridad.</p>}</section></div>
  </div>
}

function CreateView({onCancel,onCreate,busy,systemMemory}: {onCancel:()=>void;onCreate:(input:CreateInput)=>Promise<void>;busy:boolean;systemMemory:number}) {
  const [versions,setVersions] = useState<Version[]>([])
  const [versionError,setVersionError] = useState<string|null>(null)
  const [imagePreview,setImagePreview] = useState<string|null>(null)
  const [imageError,setImageError] = useState<string|null>(null)
  const [input,setInput] = useState<CreateInput>({ name:'',version:'',platform:'vanilla',memoryMinMb:1024,memoryMaxMb:2048,maxPlayers:20,port:25565,storageTargetGb:20,imagePath:null })
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(input.memoryMaxMb,systemMemoryMb || 16384))
  const invalidCapacity = input.memoryMinMb < 512 || input.memoryMaxMb < input.memoryMinMb || input.memoryMaxMb > 65536 || input.maxPlayers < 1 || input.maxPlayers > 1000 || input.storageTargetGb < 1 || input.storageTargetGb > 2048
  const loadVersions = () => api.versions().then(items => {
    setVersions(items); setVersionError(null)
    setInput(old => ({ ...old,version:old.version || items[0]?.id || '' }))
  }).catch(e => setVersionError(errorText(e)))
  useEffect(() => {
    if (!isTauri()) { setVersionError('Abre la aplicación de escritorio para consultar versiones.'); return }
    loadVersions()
  },[])
  const chooseImage = async () => {
    setImageError(null)
    try {
      const image = await api.pickServerImage()
      if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) }
    } catch(e) { setImageError(errorText(e)) }
  }
  return <div className="form-page">
    <div className="form-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/> Servidores</button><h1>Crear servidor</h1><p>Lo esencial para poner tu mundo en marcha. Podrás ajustar todo después.</p></div>
    <div className="form-layout"><form className="create-form" onSubmit={e => { e.preventDefault(); onCreate(input) }}>
      <div className="form-section-title"><span>01</span><div><h2>Identidad</h2><p>Cómo aparecerá en tu biblioteca.</p></div></div>
      <div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt="Vista previa de la imagen"/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage} disabled={!isTauri()}>Elegir imagen</button><p>PNG, JPG o WebP · hasta 2 MB</p>{imageError && <p className="form-error" role="alert">{imageError}</p>}</div><div className="form-row"><label htmlFor="new-name">Nombre del servidor</label><input id="new-name" autoFocus required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})} placeholder="Mi servidor"/><p className="field-help">Solo lo verás tú en este launcher.</p></div></div>
      <div className="form-section-title"><span>02</span><div><h2>Versión y tipo</h2><p>Selecciona cómo quieres jugar.</p></div></div>
      <div className="form-pair"><div className="form-row"><label htmlFor="new-version">Versión de Minecraft</label>{versionError ? <div className="form-error" role="alert">No se pudo cargar el catálogo: {versionError} <button type="button" onClick={loadVersions}>Reintentar</button></div> : <select id="new-version" required value={input.version} onChange={e => setInput({...input,version:e.target.value})}><option value="">{versions.length ? 'Selecciona una versión' : 'Cargando versiones…'}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}</option>)}</select>}</div><div className="form-row"><label htmlFor="new-platform">Tipo de servidor</label><select id="new-platform" value={input.platform} onChange={e => setInput({...input,platform:e.target.value as CreateInput['platform']})}><option value="vanilla">Vanilla</option><option value="paper">Paper</option></select></div></div><p className="field-help type-help">{input.platform === 'paper' ? 'Paper admite plugins y opciones extra de rendimiento.' : 'Vanilla ofrece la experiencia original de Minecraft.'}</p>
      <div className="form-section-title"><span>03</span><div><h2>Capacidad</h2><p>Un punto de partida que puedes cambiar luego.</p></div></div>
      <div className="capacity-grid"><SliderField id="new-memory-min" label="RAM inicial" value={input.memoryMinMb} min={512} max={Math.max(input.memoryMinMb,input.memoryMaxMb)} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMinMb:value,memoryMaxMb:value >= 512 && value <= 65536 ? Math.max(input.memoryMaxMb,value) : input.memoryMaxMb})}/><SliderField id="new-memory-max" label="RAM máxima" value={input.memoryMaxMb} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMaxMb:value,memoryMinMb:value >= 512 && value <= 65536 ? Math.min(input.memoryMinMb,value) : input.memoryMinMb})}/><SliderField id="new-players" label="Jugadores máximos" value={input.maxPlayers} min={1} max={Math.max(100,input.maxPlayers)} maxInput={1000} step={1} onChange={value => setInput({...input,maxPlayers:value})}/><SliderField id="new-storage" label="Objetivo de espacio" value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/></div><p className="field-help">{systemMemory ? `Tu equipo tiene ${fmtBytes(systemMemory)} de RAM.` : 'Deja memoria libre para el sistema y para jugar.'} El objetivo de espacio sirve para avisarte; no impone una cuota al disco.</p>{systemMemoryMb > 0 && input.memoryMaxMb > systemMemoryMb * .75 && <p className="inline-note" role="status">La RAM máxima supera el 75 % de la memoria física. Deja recursos para el sistema y para jugar.</p>}
      <details className="advanced"><summary>Opciones de red</summary><div className="form-row"><label htmlFor="new-port">Puerto del servidor</label><input id="new-port" type="number" min="1" max="65535" value={input.port} onChange={e => setInput({...input,port:Number(e.target.value)})}/></div></details>
      {invalidCapacity && <p className="form-error" role="alert">Revisa los límites de RAM, jugadores y espacio antes de crear el servidor.</p>}
      <div className="form-actions"><button type="button" onClick={onCancel}>Cancelar</button><button className="primary-button" disabled={busy || !input.version || !!versionError || invalidCapacity}>{busy ? 'Preparando…' : 'Crear e instalar'}<Icon name="arrow" size={16}/></button></div>
    </form><aside className="form-aside"><div className="aside-title">Tu configuración</div><div className="aside-server"><div className="image-preview small">{imagePreview ? <img src={imagePreview} alt=""/> : <Icon name="server" size={22}/>}</div><div><strong>{input.name.trim() || 'Nuevo servidor'}</strong><span>{input.platform === 'paper' ? 'Paper' : 'Vanilla'} · {input.version || 'Versión pendiente'}</span></div></div><dl><div><dt>Memoria</dt><dd>{fmtBytes(input.memoryMaxMb * 1024 * 1024)}</dd></div><div><dt>Jugadores</dt><dd>Hasta {input.maxPlayers}</dd></div><div><dt>Objetivo de espacio</dt><dd>{input.storageTargetGb} GB</dd></div><div><dt>Puerto</dt><dd>{input.port}</dd></div></dl><p>El servidor se ejecutará en este ordenador. Java y los archivos se descargarán al crearlo.</p></aside></div>
  </div>
}

function ImportView({onCancel,onImport,busy}: {onCancel:()=>void;onImport:(input:ImportInput)=>Promise<void>;busy:boolean}) {
  const [inspection,setInspection] = useState<ImportInspection|null>(null)
  const [versions,setVersions] = useState<Version[]>([])
  const [formError,setFormError] = useState<string|null>(null)
  const [imagePreview,setImagePreview] = useState<string|null>(null)
  const [picking,setPicking] = useState(false)
  const [input,setInput] = useState<ImportInput>({path:'',name:'',version:'',platform:'vanilla',jarFile:'',copy:true,storageTargetGb:20,imagePath:null})
  useEffect(() => {
    if (!isTauri()) { setFormError('Abre la aplicación de escritorio para importar una carpeta.'); return }
    api.versions().then(items => { setVersions(items); setInput(old => old.version ? old : {...old,version:items[0]?.id || ''}) }).catch(e => setFormError(`No se pudo consultar el catálogo: ${errorText(e)}`))
  },[])
  const choose = async () => {
    setPicking(true); setFormError(null)
    try {
      const path = await api.pickImportFolder()
      if (!path) return
      const info = await api.inspectImport(path)
      setInspection(info)
      setInput(old => ({path:info.path,name:info.suggestedName,version:info.detectedVersion || versions[0]?.id || '',platform:info.jarFiles.some(j => j.toLowerCase().includes('paper')) ? 'paper':'vanilla',jarFile:info.jarFiles[0] || '',copy:true,storageTargetGb:old.storageTargetGb,imagePath:old.imagePath}))
    } catch(e) { setFormError(errorText(e)) }
    finally { setPicking(false) }
  }
  const chooseImage = async () => {
    setFormError(null)
    try { const image = await api.pickServerImage(); if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) } }
    catch(e) { setFormError(errorText(e)) }
  }
  return <div className="form-page import-page"><div className="form-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/> Servidores</button><h1>Importar servidor</h1><p>Trae un servidor existente a tu biblioteca sin perder sus archivos.</p></div><div className="import-source"><div className="import-source-icon"><Icon name="folder" size={22}/></div><div><strong>{inspection ? 'Carpeta seleccionada' : 'Selecciona la carpeta del servidor'}</strong><p>{inspection ? inspection.path : 'Debe contener server.properties y un archivo .jar'}</p></div><button onClick={choose} disabled={picking || busy}>{picking ? 'Inspeccionando…' : inspection ? 'Cambiar carpeta' : 'Elegir carpeta'}</button></div>{formError && <p className="form-error" role="alert">{formError}</p>}{inspection && <form className="create-form import-form" onSubmit={e => { e.preventDefault(); onImport(input) }}><div className="form-section-title"><span>01</span><div><h2>Identidad</h2><p>Así aparecerá en tu biblioteca.</p></div></div><div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt="Vista previa de la imagen"/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage}>Elegir imagen</button><p>PNG, JPG o WebP · hasta 2 MB</p></div><div className="form-row"><label htmlFor="import-name">Nombre del servidor</label><input id="import-name" required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})}/></div></div><div className="form-section-title"><span>02</span><div><h2>Datos del servidor</h2><p>Revisa los valores detectados.</p></div></div><div className="form-pair"><div className="form-row"><label htmlFor="import-version">Versión de Minecraft</label><input id="import-version" list="import-versions" required value={input.version} onChange={e => setInput({...input,version:e.target.value})}/><datalist id="import-versions">{versions.map(v => <option key={v.id} value={v.id}/>)}</datalist></div><div className="form-row"><label htmlFor="import-platform">Tipo</label><select id="import-platform" value={input.platform} onChange={e => setInput({...input,platform:e.target.value as ImportInput['platform']})}><option value="vanilla">Vanilla</option><option value="paper">Paper</option></select></div></div><p className="field-help">Comprueba la versión del servidor original. La usaremos para elegir Java compatible.</p><div className="form-row"><label htmlFor="import-jar">Archivo del servidor</label><select id="import-jar" value={input.jarFile} onChange={e => setInput({...input,jarFile:e.target.value})}>{inspection.jarFiles.map(jar => <option key={jar} value={jar}>{jar}</option>)}</select></div><div className="import-storage"><SliderField id="import-storage" label="Objetivo de espacio" value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/><p className="field-help">Se usa para avisar cuando el servidor ocupa más espacio. No limita el disco.</p></div><fieldset className="import-mode"><legend>Ubicación de los archivos</legend><label><input type="radio" checked={input.copy} onChange={() => setInput({...input,copy:true})}/> Copiar a la biblioteca de TCHLauncher</label><label><input type="radio" checked={!input.copy} onChange={() => setInput({...input,copy:false})}/> Administrar la carpeta actual</label><p className="field-help">Si administras la carpeta actual, los cambios y las restauraciones afectarán a los archivos originales.</p></fieldset><div className="form-actions"><button type="button" onClick={onCancel}>Cancelar</button><button className="primary-button" disabled={busy || !input.version || input.storageTargetGb < 1 || input.storageTargetGb > 2048}>{busy ? 'Importando…' : 'Importar servidor'}<Icon name="arrow" size={16}/></button></div></form>}</div>
}

const settingsFields = [
  ['motd','Mensaje del servidor','text'],['server-port','Puerto','number'],
  ['gamemode','Modo de juego','text'],['difficulty','Dificultad','text'],['pvp','Combate entre jugadores','boolean'],
  ['online-mode','Verificación de cuentas','boolean'],['white-list','Lista blanca','boolean'],['enforce-whitelist','Expulsar a no autorizados','boolean'],
  ['view-distance','Distancia de visión','number'],['simulation-distance','Distancia de simulación','number'],
] as const

function Settings({server,props,setProps,memory,setMemory,storageTarget,setStorageTarget,systemMemory,busy,onSave,onPickImage,onClearImage}: {server:Server;props:Record<string,string>;setProps:(p:Record<string,string>)=>void;memory:{min:number;max:number};setMemory:(v:{min:number;max:number})=>void;storageTarget:number;setStorageTarget:(value:number)=>void;systemMemory:number;busy:boolean;onSave:()=>void;onPickImage:()=>void;onClearImage:()=>void}) {
  const [newKey,setNewKey] = useState('')
  const [newValue,setNewValue] = useState('')
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(memory.max,systemMemoryMb || 16384))
  const maxPlayers = Number(props['max-players'] || 20)
  const invalidCapacity = memory.min < 512 || memory.max < memory.min || memory.max > 65536 || storageTarget < 1 || storageTarget > 2048 || maxPlayers < 1 || maxPlayers > 1000 || ![memory.min,memory.max,maxPlayers,storageTarget].every(Number.isFinite)
  return <section className="settings-view"><div className="section-heading"><div><h2>Configuración</h2><p>Personaliza la apariencia y los ajustes del servidor.</p></div><button className="primary-button" disabled={busy || invalidCapacity || !['stopped','crashed','error'].includes(server.state)} onClick={onSave}>Guardar cambios <Icon name="check" size={16}/></button></div>
    <h3>Imagen del servidor</h3><div className="settings-image"><ServerArtwork server={server}/><div><strong>{server.name}</strong><p>Esta imagen aparece junto al servidor en la barra lateral. PNG, JPG o WebP, hasta 2 MB.</p><div className="settings-image-actions"><button disabled={busy} onClick={onPickImage}>{server.imageData ? 'Cambiar imagen' : 'Elegir imagen'}</button>{server.imageData && <button className="text-button" disabled={busy} onClick={onClearImage}>Quitar imagen</button>}</div></div></div>
    {!['stopped','crashed','error'].includes(server.state) && <p className="inline-note">Detén el servidor para editar su configuración.</p>}
    <h3>Capacidad</h3><div className="capacity-grid settings-capacity"><SliderField id="settings-memory-min" label="RAM inicial" value={memory.min} min={512} max={Math.max(memory.min,memory.max)} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,min:value,max:value >= 512 && value <= 65536 ? Math.max(memory.max,value) : memory.max})}/><SliderField id="settings-memory-max" label="RAM máxima" value={memory.max} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,max:value,min:value >= 512 && value <= 65536 ? Math.min(memory.min,value) : memory.min})}/><SliderField id="settings-players" label="Jugadores máximos" value={maxPlayers} min={1} max={Math.max(100,maxPlayers)} maxInput={1000} step={1} onChange={value => setProps({...props,'max-players':String(value)})}/><SliderField id="settings-storage" label="Objetivo de espacio" value={storageTarget} min={1} max={Math.max(200,storageTarget)} maxInput={2048} step={1} unit="GB" onChange={setStorageTarget}/></div><p className="field-help">{systemMemory ? `Memoria física: ${fmtBytes(systemMemory)}.` : ''} El objetivo de espacio solo activa avisos; no impone una cuota al disco.</p>{systemMemoryMb > 0 && memory.max > systemMemoryMb * .75 && <p className="inline-note" role="status">La RAM máxima supera el 75 % de la memoria física.</p>}{invalidCapacity && <p className="form-error" role="alert">Revisa los límites de RAM, jugadores y espacio antes de guardar.</p>}    <h3>Propiedades del servidor</h3><div className="settings-fields">{settingsFields.map(([key,label,kind]) => <label key={key} className="setting-line"><span>{label}</span>{kind === 'boolean' ? <select value={props[key] || 'false'} onChange={e => setProps({...props,[key]:e.target.value})}><option value="true">Activado</option><option value="false">Desactivado</option></select> : <input type={kind} value={props[key] ?? ''} onChange={e => setProps({...props,[key]:e.target.value})}/>}</label>)}</div>
    <details className="advanced-properties"><summary>Otras propiedades</summary><div className="settings-fields">{Object.entries(props).filter(([key]) => key !== 'max-players' && !settingsFields.some(([known]) => known === key)).map(([key,value]) => <label className="setting-line" key={key}><span>{key}</span><input value={value} onChange={e => setProps({...props,[key]:e.target.value})}/></label>)}</div><div className="add-property"><input aria-label="Nombre de propiedad" placeholder="Nombre de propiedad" value={newKey} onChange={e => setNewKey(e.target.value)}/><input aria-label="Valor" placeholder="Valor" value={newValue} onChange={e => setNewValue(e.target.value)}/><button onClick={() => { if (newKey.trim()) { setProps({...props,[newKey.trim()]:newValue}); setNewKey(''); setNewValue('') } }}>Añadir</button></div></details>
  </section>
}

export default App




