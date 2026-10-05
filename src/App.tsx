import {PixelIcon,PixelMeter} from './PixelIndicators'
import {CrashPanel} from './CrashPanel'
import { useCallback, useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '@tauri-apps/api/core'
import { api, type Backup, type CreateInput, type DeleteOutcome, type ImportInput, type ImportInspection, type LogLine, type Metrics, type PlayersSnapshot, type Progress, type Server, type ServerState, type Version, type WorldInspection } from './api'
import { PlayersView } from './PlayersView'
import {StartupProgress} from './StartupProgress'
import {StorageMeter} from './StorageMeter'
import {acceptStartupEvent,type StartupEvent} from './startupEvents'
import { SliderField } from './SliderField'
import { ConsoleView } from './ConsoleView'
import { WorldView } from './WorldView'
import { ContentManager } from './ContentManager'
import { NetworkPanel } from './NetworkPanel'
import { Modal } from './Modal'
import { ResourceMonitor } from './ResourceMonitor'
import { InstallProgress } from './InstallProgress'
import { RuntimePanel } from './RuntimePanel'
import { JvmPanel } from './JvmPanel'
import { BackupPolicyPanel } from './BackupPolicyPanel'
import { TasksPanel } from './TasksPanel'
import { StartupPanel } from './StartupPanel'
import { LoaderPanel } from './LoaderPanel'
import { MinecraftUpdatePanel } from './MinecraftUpdatePanel'
import { PackImport } from './PackImport'
import { ServerLibrary } from './ServerLibrary'
import { ApplicationSettings } from './ApplicationSettings'
import { applyAppearance, browserPreferenceStorage, readAppearance, saveAppearance, type Appearance } from './appearance'
import launcherEmblem from './assets/launcher-logo.png'
import { motion, MovingIndicator, AnimatedValue, useListMotion, useNavigationMotion } from './motion'
import { ServerMenu, type ServerMenuTarget } from './ServerMenu'
import { ServerDeleteDialog } from './ServerDeleteDialog'
import { recordSample, type MetricSample } from './telemetry'
import {CommandPalette} from './CommandPalette'
import {launcherCommands,type LauncherCommand} from './commands'
import {BackgroundTasks} from './BackgroundTasks'
import {dismissTask,isTaskActive,reconcileTasks} from './taskState'

type View = 'overview' | 'performance' | 'players' | 'worlds' | 'content' | 'console' | 'backups' | 'settings'
const backupReason:Record<string,string>={manual:'Manual',scheduled:'Programada',server_stop:'Tras detener el servidor',before_restore:'Antes de restaurar',before_minecraft_update:'Antes de actualizar Minecraft',before_loader_update:'Antes de actualizar el loader',before_content_update:'Antes de actualizar contenido',before_content_removal:'Antes de quitar contenido',before_world_change:'Antes de cambiar mundo',before_world_deletion:'Antes de borrar mundo'}
const labels: Record<ServerState, string> = {
  stopped: 'Detenido', installing: 'Instalando', starting: 'Iniciando',
  running: 'En ejecución', stopping: 'Deteniendo', crashed: 'Se ha cerrado inesperadamente', error: 'Error',
}
const viewOrder:View[]=['overview','performance','players','worlds','content','console','backups','settings']
const viewLabels:Record<View,string>={overview:'Resumen',performance:'Rendimiento',players:'Jugadores',worlds:'Mundos',content:'Mods',console:'Consola',backups:'Copias',settings:'Configuración'}
function SectionIcon({name}:{name:View|'library'|'appearance'}) {
  const paths={
    appearance:<><path d="m9 3-.5 3-2 1.2-2.8-1-2 3.5 2.3 2v2.3l-2.3 2 2 3.5 2.8-1 2 1.2.5 3h4l.5-3 2-1.2 2.8 1 2-3.5-2.3-2V11l2.3-2-2-3.5-2.8 1-2-1.2-.5-3z"/><circle cx="11" cy="12" r="3"/></>,
    library:<><rect x="4" y="4" width="16" height="16" rx="3"/><path d="M8 8h3v3H8zM14 8h2M8 14h2M14 14h2"/></>,
    overview:<><rect x="4" y="4" width="16" height="16" rx="3"/><path d="m7 13 3-4 3 5 4-5"/></>,
    performance:<><rect x="3" y="4" width="18" height="16" rx="3"/><path d="m6 14 4-5 4 6 4-5"/></>,
    players:<><circle cx="9" cy="8" r="3"/><path d="M3 20v-3a6 6 0 0 1 12 0v3M16 5a3 3 0 0 1 0 6M18 14a5 5 0 0 1 3 5"/></>,
    worlds:<><path d="m12 3 9 5v9l-9 5-9-5V8zM3 8l9 5 9-5M12 13v9"/></>,
    content:<><path d="M9 4h6v4h4v6h-4v6H9v-6H4V8h5z"/></>,
    console:<><rect x="3" y="4" width="18" height="16" rx="3"/><path d="m7 9 3 3-3 3M13 15h4"/></>,
    backups:<><path d="M3 7h7l3 3h8v10H3zM3 7V4h7l3 3h8"/></>,
    settings:<><path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3"/><circle cx="15" cy="17" r="3"/></>,
  }
  return <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{paths[name]}</svg>
}
const platformName = (platform:Server['platform']) => ({vanilla:'Vanilla',paper:'Paper',fabric:'Fabric',neoforge:'NeoForge'})[platform]
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
    : <span className="server-artwork default" aria-hidden="true">{server.name.trim().slice(0,2).toLocaleUpperCase()}</span>
}

function App() {
  const [servers, setServers] = useState<Server[]>([])
  const [railHovered,setRailHovered] = useState(false)
  const [railKeyboardFocus,setRailKeyboardFocus] = useState(false)
  const [railTasksOpen,setRailTasksOpen] = useState(false)
  const [sidebarTooltip,setSidebarTooltip] = useState<{name:string;subtitle:string;top:number;left:number}|null>(null)
  const [menuTarget,setMenuTarget] = useState<ServerMenuTarget|null>(null)
  const sidebarCollapsed = !railHovered && !railKeyboardFocus && !menuTarget && !railTasksOpen
  const menuTargetRef = useRef<ServerMenuTarget|null>(null)
  const [deleteTarget,setDeleteTarget] = useState<Server|null>(null)
  const closeMenu = useCallback((restoreFocus=false)=>{
    if(restoreFocus && menuTargetRef.current?.trigger.isConnected) menuTargetRef.current.trigger.focus({preventScroll:true})
    menuTargetRef.current=null;setMenuTarget(null)
  },[])
  const tooltipTimer=useRef<number>(0)
  const hideSidebarTooltip=()=>{clearTimeout(tooltipTimer.current);setSidebarTooltip(null)}
  useEffect(()=>()=>clearTimeout(tooltipTimer.current),[])
  const mainRef = useRef<HTMLElement>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const selectedRef = useRef<string | null>(null)
  const [view, setView] = useState<View>('overview')
  const [libraryOpen,setLibraryOpen]=useState(true)
  const [appearanceOpen,setAppearanceOpen]=useState(false)
  const [paletteOpen,setPaletteOpen]=useState(false)
  const [appearance,setAppearance]=useState(() => readAppearance(browserPreferenceStorage()))
  const [appearanceSaved,setAppearanceSaved]=useState(true)
  const changeAppearance=(next:Appearance)=>{applyAppearance(next);setAppearance(next);setAppearanceSaved(saveAppearance(browserPreferenceStorage(),next))}
  const railRef=useRef<HTMLElement>(null)
  const backupListRef=useRef<HTMLDivElement>(null)
  const contentRef=useRef<HTMLDivElement>(null)
  const [navigationDirection,setNavigationDirection]=useState(1)
  const navigate=(next:View)=>{setNavigationDirection(viewOrder.indexOf(next)>viewOrder.indexOf(view)?1:-1);setView(next)}
  const [creating, setCreating] = useState(false)
  const [importing, setImporting] = useState(false)
  const [packOpen,setPackOpen]=useState(false)
  const [packResume,setPackResume]=useState<string|null>(null)
  const [operationBusy,setBusy] = useState(false)
  const [installingIds,setInstallingIds] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [progressByServer,setProgressByServer]=useState<Record<string,Progress>>({})
  const progressRef=useRef<Record<string,Progress>>({})
  const progressRevision=useRef(0)
  const [taskSyncError,setTaskSyncError]=useState<string|null>(null)
  const activeInstallationIds=new Set([...installingIds,...Object.values(progressByServer).filter(isTaskActive).map(item=>item.serverId)])
  const busy=operationBusy || activeInstallationIds.has(selectedId ?? '')
  const progress=selectedId ? progressByServer[selectedId] ?? null : null
  const [metrics, setMetrics] = useState<Metrics | null>(null)
  const metricsGeneration=useRef<Record<string,number>>({})
  const [startupPhases,setStartupPhases]=useState<Record<string,StartupEvent>>({})
  const [metricsError,setMetricsError] = useState<string|null>(null)
  const [metricsHistory,setMetricsHistory] = useState<Record<string,MetricSample[]>>({})
  const [players, setPlayers] = useState<PlayersSnapshot | null>(null)
  const [logs, setLogs] = useState<LogLine[]>([])
  const [backups, setBackups] = useState<Backup[]>([])
  const [restoreTarget,setRestoreTarget]=useState<{serverId:string;backup:Backup}|null>(null)
  const [props, setProps] = useState<Record<string, string>>({})
  const propsDirtyRef = useRef(false)
  const [eula, setEula] = useState(false)
  const [downloadConsent,setDownloadConsent] = useState<{id:string;kind:Exclude<Progress['kind'],'content_install'|'pack_install'|'loader_update'|'minecraft_update'|'backup'|'restore'|'world_import'|'world_duplicate'|'world_export'|'world_activate'|'world_delete'|'server_import'|'content_remove'>}|null>(null)
  const [consentBusy,setConsentBusy] = useState(false)
  const [consentError,setConsentError] = useState<string|null>(null)
  const [memory, setMemory] = useState({ min: 1024, max: 2048 })
  const [storageTarget, setStorageTarget] = useState(20)
  const [systemMemory, setSystemMemory] = useState(0)
  const [closing, setClosing] = useState(false)
  const [startupOpen,setStartupOpen]=useState(false)
  const [startupBusy,setStartupBusy]=useState(false)
  const [closingBusy, setClosingBusy] = useState(false)
  const selected = servers.find(server => server.id === selectedId) ?? null


  useEffect(() => {
    hideSidebarTooltip()
  },[sidebarCollapsed])
  const showSidebarTooltip = (server: Server, button: HTMLButtonElement, immediate=false) => {
    clearTimeout(tooltipTimer.current)
    if(!sidebarCollapsed)return
    const rect = button.getBoundingClientRect()
    tooltipTimer.current=window.setTimeout(()=>setSidebarTooltip({
      name:server.name,
      subtitle:`${platformName(server.platform)} · ${server.version} · ${stateText(server.state)}`,
      top:Math.max(12,Math.min(window.innerHeight - 72,rect.top + rect.height / 2 - 28)),
      left:rect.right + 12,
    }),immediate?0:motion.tooltipDelay)
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
        if(event.payload.state==='starting'){
          setStartupPhases(current=>{const next={...current};delete next[event.payload.serverId];return next})
          metricsGeneration.current[event.payload.serverId]=(metricsGeneration.current[event.payload.serverId]??0)+1
          setMetrics(current=>current?.serverId===event.payload.serverId?null:current)
        }
        setServers(items => items.map(item => item.id === event.payload.serverId ? { ...item, state: event.payload.state } : item))
      }),
      listen<StartupEvent>('server-startup',event=>setStartupPhases(current=>acceptStartupEvent(current,event.payload))),
      listen<Progress>('install-progress', event => {progressRevision.current++;progressRef.current={...progressRef.current,[event.payload.serverId]:event.payload};setProgressByServer(progressRef.current)}),
      listen<LogLine>('server-log', event => {
        if (event.payload.serverId === selectedRef.current) setLogs(lines => [...lines.slice(-999), event.payload])
      }),
      listen('close-task-active',()=>setError('Espera a que termine la operación antes de cerrar. Puedes cancelar la copia desde Tareas.')),
      listen('close-with-servers', () => setClosing(true)),
      listen<[string,string]>('startup-result',event=>{if(event.payload[1].startsWith('Error:'))setError(`Inicio automático de «${event.payload[0]}»: ${event.payload[1]}`)}),
      listen<string>('scheduler-updated',event=>{
        const id=event.payload
        if(id===selectedRef.current)void api.backups(id).then(value=>{if(id===selectedRef.current)setBackups(value)}).catch(cause=>setError(errorText(cause)))
      }),
    ]
    return () => { listeners.forEach(p => p.then(unlisten => unlisten()).catch(() => {})) }
  }, [refresh])

  useEffect(()=>{
    if(!isTauri())return
    let active=true;let timer=0
    const sync=async()=>{
      const revision=progressRevision.current
      try{
        const snapshot=await api.installationProgress()
        if(!active)return
        if(revision===progressRevision.current){progressRef.current=reconcileTasks(progressRef.current,snapshot);setProgressByServer(progressRef.current)}
        setTaskSyncError(null)
      }catch(cause){if(active)setTaskSyncError(errorText(cause))}
      finally{if(active)timer=window.setTimeout(()=>void sync(),3000)}
    }
    void sync()
    return()=>{active=false;window.clearTimeout(timer)}
  },[])

  useEffect(() => { selectedRef.current = selectedId }, [selectedId])
  useEffect(() => {
    if (!metrics) return
    setMetricsHistory(current => ({...current,[metrics.serverId]:recordSample(current[metrics.serverId] ?? [],metrics)}))
  },[metrics])

  useEffect(() => {
    if (!selectedId) return
    let active = true
    const generation=metricsGeneration.current[selectedId]??0
    setMetrics(null); setMetricsError(null); setPlayers(null); setLogs([]); setBackups([]); setProps({}); setEula(false); propsDirtyRef.current = false
    Promise.all([api.metrics(selectedId), api.log(selectedId), api.backups(selectedId), api.eulaAccepted(selectedId)])
      .then(([m, log, b, accepted]) => {
        if (!active) return
        if(generation===(metricsGeneration.current[selectedId]??0))setMetrics(current => !current || m.sampledAt >= current.sampledAt ? m : current)
        setLogs(current => {
          const loaded = new Set(log.map(item => `${item.timestamp ?? ''}\0${item.line}`))
          return [...log, ...current.filter(item => !loaded.has(`${item.timestamp ?? ''}\0${item.line}`))].slice(-1000)
        })
        setBackups(b); setEula(accepted)
      }).catch(e => { if (active) setError(errorText(e)) })
    api.players(selectedId).then(value => { if (active) setPlayers(value) }).catch(e => { if (active) setError(errorText(e)) })
    return () => { active = false }
  }, [selectedId])

  useEffect(() => {
    if (!selectedId) return
    let active = true
    api.properties(selectedId).then(values => { if (active && !propsDirtyRef.current) setProps(values) })
      .catch(cause => { if (active) setError(errorText(cause)) })
    return () => { active = false }
  },[selectedId,selected?.state])

  useEffect(() => {
    if (!selected) return
    setMemory({ min: selected.memoryMinMb, max: selected.memoryMaxMb })
    setStorageTarget(selected.storageTargetGb)
  }, [selected?.id, selected?.memoryMinMb, selected?.memoryMaxMb, selected?.storageTargetGb])

  useEffect(() => {
    if (!selectedId) return
    let active = true
    let timer: number
    const poll = async () => {
      const generation=metricsGeneration.current[selectedId]??0
      const results = await Promise.allSettled([api.metrics(selectedId),api.players(selectedId)])
      if (!active) return
      const [sample,people] = results
      if(generation===(metricsGeneration.current[selectedId]??0)){
        if (sample.status === 'fulfilled') { setMetrics(sample.value); setMetricsError(null) }
        else setMetricsError(`No se pudo leer el proceso: ${errorText(sample.reason)}`)
      }
      if (people.status === 'fulfilled') setPlayers(people.value)
      timer = window.setTimeout(poll,3000)
    }
    timer = window.setTimeout(poll,3000)
    return () => { active = false; window.clearTimeout(timer) }
  }, [selectedId])

  useEffect(() => { mainRef.current?.scrollTo({top:0,left:0}) },[selectedId,view,creating,importing,libraryOpen,appearanceOpen])

  useEffect(() => {
    const shortcut = (event: KeyboardEvent) => {
      if(event.defaultPrevented||event.isComposing||event.repeat||event.altKey||event.shiftKey||(!event.ctrlKey&&!event.metaKey))return
      const key=event.key.toLowerCase()
      const dialog=document.querySelector('dialog[open]')
      if(key==='k') {
        if(dialog&&!dialog.classList.contains('command-palette'))return
        event.preventDefault();closeMenu();clearTimeout(tooltipTimer.current);setSidebarTooltip(null);setPaletteOpen(value=>!value);return
      }
      if(dialog)return
      if(key==='n'&&!operationBusy) {event.preventDefault();setAppearanceOpen(false);setImporting(false);setCreating(true)}
      if(key==='f') {
        const search=mainRef.current?.querySelector<HTMLInputElement>('input[type="search"]')
        if(search&&!search.disabled){event.preventDefault();search.focus();search.select()}
      }
    }
    window.addEventListener('keydown',shortcut)
    return () => window.removeEventListener('keydown',shortcut)
  }, [operationBusy,closeMenu])

  const act = async (fn: () => Promise<unknown>) => {
    setError(null); setBusy(true)
    try { await fn(); await refresh(); return true }
    catch (e) { if(errorText(e)!=='Instalación cancelada')setError(errorText(e)) }
    finally { setBusy(false) }
    return false
  }

  const runInstallation = async (id:string,kind:Exclude<Progress['kind'],'content_install'|'pack_install'|'loader_update'|'minecraft_update'|'backup'|'restore'|'world_import'|'world_duplicate'|'world_export'|'world_activate'|'world_delete'|'server_import'|'content_remove'>='install')=>{
    try {if(!await api.eulaConsentRecorded()){setConsentError(null);setDownloadConsent({id,kind});return;}}
    catch(cause){setError(errorText(cause));return;}
    setError(null);setInstallingIds(current=>new Set(current).add(id))
    try {if(kind==='install')await api.install(id);else await api.prepareRuntime(id,kind==='runtime_reinstall')}
    catch(cause){if(progressRef.current[id]?.stage!=='failed' && errorText(cause)!=='Instalación cancelada')setError(errorText(cause))}
    finally {setInstallingIds(current=>{const next=new Set(current);next.delete(id);return next});await refresh().catch(cause=>setError(errorText(cause)))}
  }
  const create = async(input:CreateInput,worldSource?:string)=>{
    let id:string|null=null
    const ready=await act(async()=>{const server=await api.create(input);setCreating(false);setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(server.id);setView('overview');await refresh();if(worldSource)await api.importWorld(server.id,worldSource,'world');id=server.id})
    if(ready && id)await runInstallation(id)
  }
  const importExisting = async(input:ImportInput)=>{
    mainRef.current?.scrollTo({top:0,left:0})
    let id:string|null=null
    const ready=await act(async()=>{const server=await api.importServer(input);setImporting(false);setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(server.id);setView('overview');await refresh();id=server.id})
    if(ready && id)await runInstallation(id)
  }

  const removeServer = async (server: Server): Promise<DeleteOutcome> => {
    setError(null); setNotice(null); setBusy(true)
    try {
      const outcome = await api.deleteServer(server.id)
      mainRef.current?.scrollTo({top:0,left:0})
      if(selectedRef.current===server.id){setSelectedId(null);setView('overview')}
      await refresh()
      setNotice(outcome.cleanupWarnings.length
        ? `Se quitó «${server.name}» de la biblioteca. Algunos archivos no se pudieron limpiar: ${outcome.cleanupWarnings.join(' ')}${outcome.retainedDirectory ? ` La carpeta original sigue en ${outcome.retainedDirectory}.` : ''}`
        : outcome.retainedDirectory
          ? `Se quitó «${server.name}» de la biblioteca. La carpeta original permanece en ${outcome.retainedDirectory}.`
          : `Se eliminó «${server.name}» y sus archivos locales.`)
      return outcome
    } catch (cause) {
      setError(errorText(cause))
      throw cause
    } finally { setBusy(false) }
  }

  useListMotion(backupListRef,`${view}:${backups.map(backup=>backup.id).join()}`)
  useListMotion(railRef,servers.map(server=>server.id).join(','))
  useNavigationMotion(contentRef,`${selectedId}:${view}:${appearanceOpen}:${creating}:${importing}:${libraryOpen}`,navigationDirection)
  const selectedName = selected?.name ?? 'Servidores'
  const openLibrary=()=>{setAppearanceOpen(false);setLibraryOpen(true);setCreating(false);setImporting(false);hideSidebarTooltip()}
  const openServer=(server:Server)=>{setAppearanceOpen(false);setLibraryOpen(false);setCreating(false);setImporting(false);setSelectedId(server.id);setView('overview')}
  const openCreate=()=>{setAppearanceOpen(false);setImporting(false);setCreating(true);hideSidebarTooltip()}
  const openImport=()=>{setAppearanceOpen(false);setCreating(false);setImporting(true);hideSidebarTooltip()}
  const openMenu=(server:Server,trigger:HTMLButtonElement,x:number,y:number)=>{const target={server,trigger,x,y};menuTargetRef.current=target;setMenuTarget(target)}
  const commands=launcherCommands(servers,operationBusy,activeInstallationIds)
  const executeCommand=(command:LauncherCommand)=>{
    const current=launcherCommands(servers,operationBusy,activeInstallationIds).find(item=>item.id===command.id)
    if(!current||current.disabledReason)return
    setPaletteOpen(false)
    if(current.action==='library'){openLibrary();return}
    if(current.action==='create'){openCreate();return}
    if(current.action==='import'){openImport();return}
    if(current.action==='appearance'){setAppearanceOpen(true);setCreating(false);setImporting(false);return}
    const server=servers.find(item=>item.id===current.serverId)
    if(!server)return
    openServer(server)
    if(current.action==='console'||current.action==='content'){setView(current.action);return}
    if(current.action==='start')void act(()=>api.start(server.id))
    if(current.action==='stop')void act(()=>api.stop(server.id))
    if(current.action==='backup') {
      setView('backups')
      void act(async()=>{const copy=await api.createBackup(server.id);const copies=await api.backups(server.id);if(selectedRef.current===server.id)setBackups(copies);if(copy.retentionWarning)throw new Error(copy.retentionWarning)})
    }
  }
  return <div className={`shell auto-rail ${sidebarCollapsed ? 'sidebar-collapsed' : 'rail-expanded'}`}>
    {paletteOpen&&<CommandPalette commands={commands} onClose={()=>setPaletteOpen(false)} onExecute={executeCommand}/>}
    <aside className="server-rail" aria-label="Servidores" onPointerEnter={event=>{if(event.pointerType!=='touch')setRailHovered(true)}} onPointerLeave={()=>setRailHovered(false)} onPointerDown={()=>setRailKeyboardFocus(false)} onFocusCapture={event=>{if(event.target.matches(':focus-visible'))setRailKeyboardFocus(true)}} onKeyDownCapture={event=>{if(event.key==='Tab')setRailKeyboardFocus(true)}} onBlurCapture={event=>{if(!(event.relatedTarget instanceof Node)||!event.currentTarget.contains(event.relatedTarget))setRailKeyboardFocus(false)}}>
      <button className={`rail-library ${libraryOpen && !creating && !importing && !appearanceOpen?'active':''}`} onClick={openLibrary} aria-label="Mis servidores" title="Mis servidores"><span className="brand-emblem"><img src={launcherEmblem} alt=""/></span><strong className="rail-brand-name">TCHLauncher</strong></button>
      <nav className="server-list" ref={railRef} aria-label="Lista de servidores" onScroll={() => hideSidebarTooltip()}>
        <MovingIndicator active={`${selectedId}:${creating}:${importing}:${libraryOpen}:${appearanceOpen}:${servers.map(s=>s.id).join()}`} className="rail-active-indicator"/>{servers.map(server => <button data-layout-id={server.id} key={server.id} className={`server-row ${selectedId === server.id && !creating && !importing && !libraryOpen && !appearanceOpen ? 'active' : ''}`} onClick={() => { hideSidebarTooltip(); setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(server.id); setCreating(false); setImporting(false); setView('overview') }} onContextMenu={event=>{event.preventDefault();hideSidebarTooltip();const rect=event.currentTarget.getBoundingClientRect();const target={server,x:event.clientX || rect.right,y:event.clientY || rect.top,trigger:event.currentTarget};menuTargetRef.current=target;setMenuTarget(target)}} onMouseEnter={event => showSidebarTooltip(server,event.currentTarget)} onMouseLeave={() => hideSidebarTooltip()} onFocus={event => showSidebarTooltip(server,event.currentTarget,true)} onBlur={() => hideSidebarTooltip()} aria-label={`${server.name}, ${platformName(server.platform)} ${server.version}, ${stateText(server.state)}`} aria-current={selectedId === server.id && !creating && !importing && !libraryOpen && !appearanceOpen ? 'page' : undefined}>
          <span className="server-artwork-wrap"><ServerArtwork server={server}/>{server.pinned && <span className="server-pin" title="Servidor fijado" aria-label="Fijado"><svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m15 3 6 6-4 1-4 5-4-4 5-4zM9 15l-6 6"/></svg></span>}<span className={`state-dot ${server.state}`} aria-hidden="true"/></span><span className="server-row-name">{server.name}<small>{platformName(server.platform)} · {server.version}</small></span>
        </button>)}
      </nav>
      <button className="rail-add" onClick={openCreate} aria-label="Nuevo servidor" title="Nuevo servidor"><Icon name="plus" size={19}/><span>Nuevo servidor</span></button>
      <button className="rail-import" onClick={openImport} aria-label="Importar servidor" title="Importar servidor"><Icon name="folder" size={18}/><span>Importar servidor</span></button>
      <BackgroundTasks tasks={progressByServer} servers={servers} error={taskSyncError} onOpenChange={setRailTasksOpen} onOpen={task=>{const server=servers.find(item=>item.id===task.serverId);if(!server)return;openServer(server);if((task.kind==='backup'||task.kind==='restore'))setView('backups');if((task.kind==='world_import'||task.kind==='world_duplicate'||task.kind==='world_export'||task.kind==='world_activate'||task.kind==='world_delete'))setView('worlds');if(task.kind==='content_install'||task.kind==='content_remove')setView('content');if(task.kind==='loader_update'||task.kind==='minecraft_update')setView('settings')}} onDismiss={task=>{progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>
      <button className="rail-startup" onClick={()=>setStartupOpen(true)} aria-label="Inicio automático" title="Inicio automático"><SectionIcon name="settings"/><span>Inicio automático</span></button>
      <button className={`rail-settings ${appearanceOpen?'active':''}`} onClick={()=>{setAppearanceOpen(true);setCreating(false);setImporting(false);hideSidebarTooltip();closeMenu()}} aria-current={appearanceOpen?'page':undefined} aria-label="Ajustes" title="Ajustes"><SectionIcon name="appearance"/><span>Ajustes</span></button>
      <span className="rail-version">v0.18.0</span>
    </aside>
    <main className="main" ref={mainRef}>
      {error && <div className="error-banner" role="alert"><div><strong>No se pudo completar la acción</strong><p>{error}</p></div><button onClick={() => setError(null)} aria-label="Cerrar error">×</button></div>}
      {notice && <div className="notice-banner" role="status"><span>{notice}</span><button onClick={() => setNotice(null)} aria-label="Cerrar aviso">×</button></div>}
      {appearanceOpen ? <ApplicationSettings appearance={appearance} saved={appearanceSaved} onAppearance={changeAppearance} servers={servers} selectedId={selectedId} onSelect={setSelectedId} busy={operationBusy} progress={progressByServer} onPrepare={(id,reinstall)=>void runInstallation(id,reinstall?'runtime_reinstall':'runtime')} onChanged={refresh} onOpenServer={(server,section)=>{openServer(server);setView(section)}} onDismiss={task=>{progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>
      : creating ? <CreateView onCancel={openLibrary} onCreate={create} busy={operationBusy} systemMemory={systemMemory}/>
      : importing ? <>{operationBusy && Object.values(progressByServer).filter(task=>task.kind==='server_import'&&isTaskActive(task)).map(task=><InstallProgress key={task.taskId} progress={task} busy={true} allowRetry={false} onRetry={()=>{}} onDismiss={()=>{}}/>)}<ImportView onCancel={openLibrary} onImport={importExisting} busy={operationBusy}/></>
      : libraryOpen || !selected ? <ServerLibrary servers={servers} onCreate={openCreate} onImport={openImport} onPack={()=>{setPackResume(null);setPackOpen(true)}} onSelect={openServer} onMenu={openMenu}/>
      : <>
        <header className="page-head">
          <div className="page-identity"><ServerArtwork server={selected}/><div><h1>{selectedName}</h1><div className="page-context">{platformName(selected.platform)} <span>·</span> Minecraft {selected.version}</div></div></div>
          <div className="head-actions">
            {view !== 'overview' && <span className={`status-label ${selected.state}`}><span className={`state-dot ${selected.state}`} aria-hidden="true"/>{stateText(selected.state)}</span>}
            {selected.jarPath && selected.javaPath ? selected.state === 'running' ? <>
              <button disabled={busy} onClick={() => act(() => api.restart(selected.id))}>Reiniciar</button>
              <button className="primary-button" disabled={busy} onClick={() => act(() => api.stop(selected.id))}>Detener</button>
            </> : selected.state === 'starting' ? <button disabled={busy} onClick={() => act(() => api.stop(selected.id))}>Detener inicio</button> : selected.state === 'stopped' || selected.state === 'crashed' || selected.state === 'error' ? <button className="primary-button" disabled={busy || !eula} onClick={() => act(() => api.start(selected.id))}>Iniciar servidor <Icon name="arrow" size={15}/></button> : null : <button className="primary-button" disabled={busy} onClick={() => runInstallation(selected.id)}>{selected.jarPath ? 'Preparar Java' : 'Instalar'} <Icon name="arrow" size={15}/></button>}
          </div>
        </header>
        <nav className="server-sections" aria-label="Secciones del servidor">
          <MovingIndicator active={`${selectedId}:${view}`} axis="both" className="section-active-indicator"/>
          {viewOrder.filter(item=>item!=='content'||selected.platform!=='vanilla').map(item=><button key={item} aria-current={view===item?'page':undefined} onClick={()=>navigate(item)}><SectionIcon name={item}/><span>{item==='content'&&selected.platform==='paper'?'Plugins':viewLabels[item]}</span></button>)}
        </nav>
        <div className="content" ref={contentRef}>
          {progress && !(view==='content'&&(progress.kind==='content_install'||progress.kind==='content_remove'))&&!(view==='settings'&&(progress.kind==='loader_update'||progress.kind==='minecraft_update')) && <InstallProgress key={progress.taskId} progress={progress} busy={busy} retryLabel={(progress.kind==='backup'||progress.kind==='restore')?'Ver copias':(progress.kind==='world_import'||progress.kind==='world_duplicate'||progress.kind==='world_export'||progress.kind==='world_activate'||progress.kind==='world_delete')?'Ver mundos':'Reintentar'} onRetry={()=>{if(progress.kind==='server_import'){setView('overview');return;}if((progress.kind==='world_import'||progress.kind==='world_duplicate'||progress.kind==='world_export'||progress.kind==='world_activate'||progress.kind==='world_delete')){setView('worlds');return;}if((progress.kind==='backup'||progress.kind==='restore')){setView('backups');return;}if(progress.kind==='pack_install'){setPackResume(selected.id);setPackOpen(true);return;}if(progress.kind==='content_install'||progress.kind==='content_remove'){setView('content');return;}if((progress.kind==='loader_update'||progress.kind==='minecraft_update')){setView('settings');return;}void runInstallation(selected.id,progress.kind)}} onDismiss={()=>{const next={...progressRef.current};delete next[selected.id];progressRef.current=next;setProgressByServer(next)}}/>}
          {view === 'overview' && <Overview key={selected.id} server={selected} observedPhase={startupPhases[selected.id]?.phase} metrics={metrics?.serverId === selected.id ? metrics : null} players={players} logs={logs} backups={backups} eula={eula} busy={busy} onAccept={() => act(async () => { await api.acceptEula(selected.id); setEula(true) })} onInstall={() => runInstallation(selected.id)} onConsole={() => setView('console')} onSettings={()=>setView('settings')} onContent={()=>setView('content')}/>}
          {view === 'performance' && <ResourceMonitor server={selected} metrics={metrics?.serverId === selected.id ? metrics : null} history={metricsHistory[selected.id] ?? []} error={metricsError}/>}
          {view === 'players' && <PlayersView key={selected.id} server={selected} players={players} busy={busy} onRefresh={async () => { try { setPlayers(await api.players(selected.id)) } catch(e) { setError(errorText(e)) } }} onAction={(action,target,reason) => act(() => api.managePlayer(selected.id,action,target,reason))} onToggleWhitelist={() => act(async () => { if (selected.state === 'running') await api.managePlayer(selected.id,players?.whitelistEnabled ? 'whitelist_off' : 'whitelist_on'); else { const values = { ...await api.properties(selected.id),'white-list':String(!players?.whitelistEnabled) }; await api.saveProperties(selected.id,values); setProps(values) } setPlayers(await api.players(selected.id)) })} onImportWhitelist={names => act(() => api.importWhitelist(selected.id,names))}/>}
          {view === 'worlds' && <WorldView server={selected} onBackupsChanged={async () => setBackups(await api.backups(selected.id))}/>}
          {view === 'content' && selected.platform!=='vanilla' && <ContentManager key={selected.id} server={selected} progress={progress} onInstalled={async()=>{await refresh();setBackups(await api.backups(selected.id))}} onDismissProgress={()=>{const next={...progressRef.current};delete next[selected.id];progressRef.current=next;setProgressByServer(next)}}/>}
          {view === 'console'  && <ConsoleView serverId={selected.id} state={selected.state} logs={logs} busy={busy} onSend={command => act(() => api.send(selected.id,command))}/> }
          {view === 'backups' && <section><div className="section-heading"><div><h2>Copias de seguridad</h2><p>Se guardan en el almacenamiento local de la aplicación.</p></div><button disabled={busy || !['stopped','crashed','error'].includes(selected.state)} onClick={() => act(async () => { const copy=await api.createBackup(selected.id); setBackups(await api.backups(selected.id));if(copy.retentionWarning)throw new Error(copy.retentionWarning) })}>Crear copia</button></div><BackupPolicyPanel key={selected.id} id={selected.id} disabled={busy||!['stopped','crashed','error'].includes(selected.state)} onChanged={async()=>setBackups(await api.backups(selected.id))}/>{!['stopped','crashed','error'].includes(selected.state) && <p className="inline-note">Detén el servidor para crear una copia consistente.</p>}{backups.length ? <div className="table" ref={backupListRef}><div className="table-head"><span>Fecha y motivo</span><span>Tamaño</span><span>Acción</span></div>{backups.map(backup => <div className="table-row" data-layout-id={backup.id} key={backup.id}><div className="backup-description"><span>{new Date(backup.createdAt).toLocaleString()}</span><strong>{backup.reason?backupReason[backup.reason]??'Motivo no disponible':'Motivo no registrado'}</strong><small>{backup.minecraftVersion?`${backup.platform??'Motor no registrado'} · Minecraft ${backup.minecraftVersion}`:'Versión no registrada'}</small>{backup.metadataError&&<small className="form-error">{backup.metadataError}</small>}</div><span>{fmtBytes(backup.sizeBytes)}</span><button className="text-button" disabled={busy || !['stopped','crashed','error'].includes(selected.state)} onClick={()=>{setError(null);setRestoreTarget({serverId:selected.id,backup})}}>Restaurar</button></div>)}</div> : <div className="empty-inline">Aún no hay copias de este servidor.</div>}</section>}
          {view === 'settings' && <Settings onConsole={()=>setView('console')} onRuntimeChanged={refresh} onPrepareRuntime={reinstall=>runInstallation(selected.id,reinstall?'runtime_reinstall':'runtime')} server={selected} props={props} setProps={values => { propsDirtyRef.current = true; setProps(values) }} memory={memory} setMemory={setMemory} storageTarget={storageTarget} setStorageTarget={setStorageTarget} systemMemory={systemMemory} busy={busy} progress={progress} onSave={() => act(async () => { await api.saveProperties(selected.id,props); await api.saveMemory(selected.id,memory.min,memory.max); await api.saveStorageTarget(selected.id,storageTarget); setProps(await api.properties(selected.id)); propsDirtyRef.current = false })} onPickImage={() => act(async () => { const image = await api.pickServerImage(); if (image) await api.setServerImage(selected.id,image.path) })} onClearImage={() => act(() => api.clearServerImage(selected.id))} onDelete={() => setDeleteTarget(selected)}/>}
        </div>
      </>}
    </main>
    {sidebarTooltip && createPortal(<div className="sidebar-tooltip" role="tooltip" style={{top:sidebarTooltip.top,left:sidebarTooltip.left}}><strong>{sidebarTooltip.name}</strong><span>{sidebarTooltip.subtitle}</span></div>,document.body)}
    {menuTarget && <ServerMenu target={{...menuTarget,server:servers.find(server=>server.id===menuTarget.server.id) ?? menuTarget.server}} busy={operationBusy || activeInstallationIds.has(menuTarget.server.id)} onClose={closeMenu} onPin={()=>act(()=>api.setPinned(menuTarget.server.id,!menuTarget.server.pinned))} onSettings={()=>{setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(menuTarget.server.id);setCreating(false);setImporting(false);setView('settings')}} onDelete={()=>setDeleteTarget(menuTarget.server)}/>}
    {deleteTarget && <ServerDeleteDialog key={deleteTarget.id} server={deleteTarget} onClose={()=>setDeleteTarget(null)} onDelete={()=>removeServer(deleteTarget)}/>}
    {startupOpen&&<Modal labelledBy="startup-title" busy={startupBusy} onClose={()=>setStartupOpen(false)}><h2 id="startup-title">Inicio automático</h2><StartupPanel servers={servers} onBusy={setStartupBusy}/><div className="modal-actions"><button data-modal-close disabled={startupBusy}>Cerrar</button></div></Modal>}
    {restoreTarget && <Modal labelledBy="restore-title" describedBy="restore-description" busy={operationBusy} onClose={()=>setRestoreTarget(null)}>
      <h2 id="restore-title">Restaurar copia de seguridad</h2>
      <p id="restore-description">Los archivos y la configuración volverán al estado de esta copia. Antes guardaremos una copia del estado actual.</p>
      <dl className="restore-preview-details"><dt>Fecha</dt><dd>{new Date(restoreTarget.backup.createdAt).toLocaleString()}</dd><dt>Tamaño</dt><dd>{fmtBytes(restoreTarget.backup.sizeBytes)}</dd><dt>Motivo</dt><dd>{restoreTarget.backup.reason?backupReason[restoreTarget.backup.reason]??'No disponible':'No registrado'}</dd><dt>Servidor guardado</dt><dd>{restoreTarget.backup.minecraftVersion?`${restoreTarget.backup.platform??'Motor no registrado'} · Minecraft ${restoreTarget.backup.minecraftVersion}`:'Versión no registrada'}</dd></dl>
      {progressByServer[restoreTarget.serverId]?.kind==='restore'&&<InstallProgress key={progressByServer[restoreTarget.serverId].taskId} progress={progressByServer[restoreTarget.serverId]} busy={operationBusy} allowRetry={false} onRetry={()=>{}} onDismiss={()=>{const task=progressByServer[restoreTarget.serverId];progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>}
      {error && <p className="form-error" role="alert">{error}</p>}
      <div className="modal-actions"><button data-modal-close disabled={operationBusy}>Cancelar</button><button className="primary-button" disabled={operationBusy} onClick={async()=>{
        const target=restoreTarget;const success=await act(async()=>{
          await api.restoreBackup(target.serverId,target.backup.id)
          const [values,copies,accepted]=await Promise.all([api.properties(target.serverId),api.backups(target.serverId),api.eulaAccepted(target.serverId)])
          if(selectedRef.current===target.serverId){propsDirtyRef.current=false;setProps(values);setBackups(copies);setEula(accepted)}
        });if(success)setRestoreTarget(null)
      }}>{operationBusy?'Restaurando…':'Restaurar copia'}</button></div>
    </Modal>}
    {packOpen && <PackImport resumeId={packResume} systemMemory={systemMemory} onClose={()=>{setPackOpen(false);void refresh().catch(cause=>setError(errorText(cause)))}} onInstalled={async id=>{await refresh();setPackOpen(false);setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(id);setView('overview')}}/>}
    {downloadConsent && <Modal labelledBy="download-consent-title" describedBy="download-consent-description" busy={consentBusy} onClose={()=>setDownloadConsent(null)}>
      <h2 id="download-consent-title">Antes de descargar Minecraft</h2>
      <p id="download-consent-description">Lee la <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">EULA de Minecraft</a>. Al aceptar, guardaremos tu decisión en este equipo para los siguientes servidores.</p>
      {consentError && <p className="form-error" role="alert">{consentError}</p>}
      <div className="modal-actions"><button data-modal-close disabled={consentBusy}>Ahora no</button><button className="primary-button" disabled={consentBusy} onClick={async()=>{
        const target=downloadConsent;setConsentBusy(true);setConsentError(null);
        try {await api.acceptEula(target.id);if(selectedRef.current===target.id)setEula(true);setDownloadConsent(null);await runInstallation(target.id,target.kind);}
        catch(cause){setConsentError(errorText(cause));}
        finally {setConsentBusy(false);}
      }}>{consentBusy?'Guardando…':'Aceptar y descargar'}</button></div>
    </Modal>}
    {closing && <Modal labelledBy="close-title" describedBy="close-description" busy={closingBusy} onClose={() => setClosing(false)}><h2 id="close-title">Hay servidores en ejecución</h2><p id="close-description">Elige qué hacer antes de cerrar.</p><div className="close-options"><button disabled={closingBusy} onClick={() => { setError(null); api.hideWindow().then(() => setClosing(false)).catch(e => setError(errorText(e))) }}>Mantener en segundo plano</button><button disabled={closingBusy} onClick={() => { setClosingBusy(true); api.stopAllAndExit().catch(e => { setClosingBusy(false); setError(errorText(e)); setClosing(false) }) }}>{closingBusy ? 'Deteniendo y guardando copias…' : 'Detener y salir'}</button><button data-modal-close autoFocus disabled={closingBusy} onClick={() => setClosing(false)}>Cancelar</button></div></Modal>}
  </div>
}

const eventLabels: Record<string,string> = {
  startup:'Servidor iniciado', shutdown:'Servidor deteniéndose', join:'Jugador conectado',
  world_load:'Cargando mundo',spawn_prepare:'Preparando zona inicial',
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

function Overview({server,metrics,observedPhase,players,logs,backups,eula,busy,onAccept,onInstall,onConsole,onSettings,onContent}: {
  observedPhase?:StartupEvent['phase']; server: Server; metrics: Metrics | null; players: PlayersSnapshot | null; logs: LogLine[]; backups: Backup[];
  eula: boolean; busy: boolean; onAccept: () => void; onInstall: () => void; onConsole: () => void; onSettings:()=>void; onContent:()=>void;
}) {
  const recentEvents = logs.filter(item => item.event && eventLabels[item.event]).slice(-4).reverse()
  const warnings = logs.filter(item => item.level === 'warn').length
  const errors = logs.filter(item => item.level === 'error').length
  const lastSave = logs.filter(item => item.event === 'save').at(-1)
  const description = server.state === 'running' ? 'Minecraft ha terminado de cargar el mundo.'
    : server.state === 'starting' ? 'Puedes seguir las fases del arranque o abrir la consola.'
    : server.state === 'stopping' ? 'Guardando el mundo y cerrando el proceso.'
    : server.state === 'installing' ? 'Descargando los archivos necesarios.'
    : server.state === 'crashed' || server.state === 'error' ? 'Revisa la consola antes de volver a iniciarlo.'
    : server.jarPath && server.javaPath ? 'Todo preparado para el próximo inicio.'
    : 'Instala Minecraft y Java para empezar.'
  return <div className="overview">
    <section className={`server-stage ${server.state}`} aria-label="Estado del servidor">
      <div className="stage-main"><div className="stage-copy" role="status"><h2><span className={`state-dot large ${server.state}`} aria-hidden="true"/>{stateText(server.state)}</h2><p>{description}</p></div>{['starting','stopping','crashed','error'].includes(server.state) && <button className="text-button" onClick={onConsole}>Ver consola <Icon name="arrow" size={15}/></button>}</div>
      <StartupProgress state={server.state} metrics={metrics} observedPhase={observedPhase}/>
      <div className="stage-stats">
        <div className="stat-players"><PixelIcon kind="players"/><span>Jugadores</span><strong>{players ? `${players.online.length} / ${players.maxPlayers}` : '—'}</strong><PixelMeter label="Plazas ocupadas" ratio={players&&players.maxPlayers>0?players.online.length/players.maxPlayers:null} valueText={players?`${players.online.length} de ${players.maxPlayers} plazas ocupadas`:undefined}/><small>{players ? `${Math.max(0,players.maxPlayers-players.online.length)} plazas libres` : 'Consultando jugadores'}</small></div>
        <div className="stat-uptime"><PixelIcon kind="clock"/><span>Tiempo activo</span><strong><AnimatedValue value={metrics?.uptimeSeconds} format={fmtUptime}/></strong><small>Desde el inicio de Java</small></div>
        <div className="stat-memory"><PixelIcon kind="memory"/><span>RAM del proceso</span><strong><AnimatedValue value={metrics?.memoryBytes} format={fmtBytes}/></strong><PixelMeter label="Heap usado respecto al máximo configurado" ratio={metrics?.heap&&server.memoryMaxMb>0?metrics.heap.usedBytes/(server.memoryMaxMb*1024**2):null}/><small>Heap: {metrics?.heap?fmtBytes(metrics.heap.usedBytes):'—'} / {(server.memoryMaxMb/1024).toFixed(1)} GB configurados</small></div>
        <div className="stat-cpu"><PixelIcon kind="cpu"/><span>CPU</span><strong><AnimatedValue value={metrics?.cpuPercent} format={value=>`${value.toFixed(1)} %`}/></strong><PixelMeter label="CPU del proceso" ratio={metrics?.cpuPercent==null?null:metrics.cpuPercent/100}/><small>Uso del proceso Java</small></div>
      </div>
    </section>
    {(!server.jarPath || !server.javaPath) && <div className="callout"><div><strong>Instalación pendiente</strong><p>{server.jarPath ? 'Prepararemos Java sin reemplazar los archivos de arranque existentes.' : `Descargaremos Minecraft ${server.version} y Java.`}</p></div><button disabled={busy} onClick={onInstall}>{server.jarPath ? 'Preparar Java' : 'Instalar ahora'}</button></div>}
    {server.jarPath && !eula && <div className="callout"><div><strong>Antes del primer inicio</strong><p>Lee la <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">EULA de Minecraft</a> antes de aceptarla. Se recordará en este equipo para tus próximos servidores.</p></div><button disabled={busy} onClick={onAccept}>He leído y acepto</button></div>}
    <CrashPanel server={server} onConsole={onConsole} onSettings={onSettings} onContent={onContent}/>
    <NetworkPanel server={server}/>
    <div className="overview-storage"><section className="resource-section"><h2>Almacenamiento</h2>
      <StorageMeter metrics={metrics} targetGb={server.storageTargetGb} onSettings={onSettings}/>
    </section><section><h2>Estado del registro</h2><dl className="resource-facts"><div><dt>Avisos</dt><dd>{warnings}</dd></div><div><dt>Errores</dt><dd>{errors}</dd></div><div><dt>Último guardado registrado</dt><dd>{eventTime(lastSave?.timestamp ?? null)}</dd></div></dl><p className="field-help">En las últimas {logs.length} líneas cargadas.</p></section></div>
    <div className="overview-lower"><section className="recent-activity"><div className="section-heading"><h2>Actividad reciente</h2><button className="text-button" onClick={onConsole}>Ver consola <Icon name="arrow" size={15}/></button></div>{recentEvents.length ? <div className="event-list">{recentEvents.map((item,index) => <div className="event-row" key={`${item.timestamp ?? 'legacy'}-${index}`}><span className={`event-mark ${item.level}`} aria-hidden="true"/><div><strong>{eventLabels[item.event!]}</strong><span>{item.message}</span></div><time>{eventTime(item.timestamp)}</time></div>)}</div> : <p className="empty-inline">Todavía no hay actividad.</p>}</section><section className="recent-backups"><div className="section-heading"><h2>Copias recientes</h2></div>{backups.length ? <div className="backup-mini-list">{backups.slice(0,3).map(backup => <div key={backup.id}><strong>{new Date(backup.createdAt).toLocaleString('es-ES')}</strong><span>{fmtBytes(backup.sizeBytes)}</span></div>)}</div> : <p className="empty-inline">Aún no hay copias guardadas.</p>}</section></div>
  </div>
}

function CreateView({onCancel,onCreate,busy,systemMemory}: {onCancel:()=>void;onCreate:(input:CreateInput,worldSource?:string)=>Promise<void>;busy:boolean;systemMemory:number}) {
  const steps = ['Identidad','Versión','Mundo','Capacidad','Revisar'] as const
  const [step,setStep] = useState(0)
  const wizardRef=useRef<HTMLDivElement>(null)
  const [stepDirection,setStepDirection]=useState(1)
  const moveStep=(next:number)=>{setStepDirection(next>step?1:-1);setStep(next)}
  useNavigationMotion(wizardRef,String(step),stepDirection,16)
  const [furthestStep,setFurthestStep] = useState(0)
  const [versions,setVersions] = useState<Version[]>([])
  const [versionLoading,setVersionLoading]=useState(true)
  const [experimental,setExperimental]=useState(false)
  const [catalogRevision,setCatalogRevision]=useState(0)
  const [versionError,setVersionError] = useState<string|null>(null)
  const [imagePreview,setImagePreview] = useState<string|null>(null)
  const [imageError,setImageError] = useState<string|null>(null)
  const [worldMode,setWorldMode] = useState<'new'|'existing'>('new')
  const [worldSource,setWorldSource] = useState<WorldInspection|null>(null)
  const [worldError,setWorldError] = useState<string|null>(null)
  const [input,setInput] = useState<CreateInput>({ name:'',version:'',platform:'vanilla',memoryMinMb:1024,memoryMaxMb:2048,maxPlayers:20,port:25565,storageTargetGb:20,imagePath:null })
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(input.memoryMaxMb,systemMemoryMb || 16384))
  const invalidCapacity = input.memoryMinMb < 512 || input.memoryMaxMb < input.memoryMinMb || input.memoryMaxMb > 65536 || input.maxPlayers < 1 || input.maxPlayers > 1000 || input.storageTargetGb < 1 || input.storageTargetGb > 2048 || input.port < 1 || input.port > 65535 || ![input.memoryMinMb,input.memoryMaxMb,input.maxPlayers,input.storageTargetGb,input.port].every(Number.isFinite)
  const readyToCreate = !!input.name.trim() && !!input.version && !versionError && !versionLoading && !invalidCapacity && (worldMode === 'new' || !!worldSource)
  const canAdvance = step === 0 ? !!input.name.trim() : step === 1 ? !!input.version && !versionError && !versionLoading : step === 2 ? worldMode === 'new' || !!worldSource : step === 3 ? !invalidCapacity : readyToCreate
  const loadVersions = () => setCatalogRevision(value=>value+1)
  useEffect(() => {
    let active=true;setVersionLoading(true);setVersionError(null)
    if(!isTauri()){setVersionError('Abre la aplicación de escritorio para consultar versiones.');setVersionLoading(false);return}
    api.versions(input.platform,experimental&&['vanilla','fabric'].includes(input.platform)).then(items=>{
      if(!active)return
      setVersions(items)
      if(!items.length)setVersionError('No hay versiones compatibles disponibles.')
      setInput(old=>({...old,version:items.some(item=>item.id===old.version)?old.version:items[0]?.id || ''}))
    }).catch(cause=>{if(active)setVersionError(errorText(cause))}).finally(()=>{if(active)setVersionLoading(false)})
    return ()=>{active=false}
  },[input.platform,experimental,catalogRevision])
  const chooseImage = async () => {
    setImageError(null)
    try {
      const image = await api.pickServerImage()
      if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) }
    } catch(e) { setImageError(errorText(e)) }
  }
  const chooseWorld = async () => {
    setWorldError(null)
    try { const selected = await api.pickWorldFolder(); if (selected) setWorldSource(selected) }
    catch(e) { setWorldError(errorText(e)) }
  }
  useEffect(() => { if (step > 0) document.querySelector<HTMLElement>('.wizard-panel h2')?.focus() },[step])
  const advance = () => {
    if (!canAdvance || busy) return
    if (step < steps.length - 1) {
      const next = step + 1
      moveStep(next)
      setFurthestStep(current => Math.max(current,next))
    } else onCreate(input,worldMode === 'existing' ? worldSource?.path : undefined)
  }
  return <div className="form-page">
    <div className="form-heading wizard-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/> Servidores</button><div className="wizard-title-row"><h1>Nuevo servidor</h1><span className="wizard-counter">Paso {step + 1} de {steps.length}</span></div></div>
    <nav className="wizard-steps" aria-label="Pasos de creación"><MovingIndicator active={String(step)} axis="horizontal"/>{steps.map((label,index) => <button type="button" key={label} className={index === step ? 'active' : index < step ? 'completed' : ''} aria-current={index === step ? 'step' : undefined} disabled={busy || index > furthestStep} onClick={() => moveStep(index)}><span>{String(index + 1).padStart(2,'0')}</span><strong>{label}</strong></button>)}</nav>
    <div className="form-layout wizard-layout"><form className="create-form wizard-form" onSubmit={e => { e.preventDefault(); advance() }}>
      <div className="wizard-panel" key={step} ref={wizardRef}>
      {step === 0 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>Nombre e imagen</h2><p>Así aparecerá en tu biblioteca.</p></div>
      <div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt="Vista previa de la imagen"/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage} disabled={!isTauri()}>Elegir imagen</button><p>PNG, JPG o WebP · hasta 2 MB</p>{imageError && <p className="form-error" role="alert">{imageError}</p>}</div><div className="form-row"><label htmlFor="new-name">Nombre del servidor</label><input id="new-name" autoFocus required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})} placeholder="Mi servidor"/><p className="field-help">Solo lo verás tú en este launcher.</p></div></div>
      </>}
      {step === 1 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>Versión y tipo</h2><p>Elige la versión de Minecraft que utilizarán tus jugadores.</p></div>
      <div className="form-pair"><div className="form-row"><label htmlFor="new-version">Versión de Minecraft</label>{versionError ? <div className="form-error" role="alert">No se pudo cargar el catálogo: {versionError} <button type="button" onClick={loadVersions}>Reintentar</button></div> : <select id="new-version" disabled={versionLoading} required value={input.version} onChange={e => setInput({...input,version:e.target.value})}><option value="">{versionLoading ? 'Cargando versiones…' : 'Selecciona una versión'}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}{v.kind==='snapshot'?' · Experimental':''}</option>)}</select>}</div><div className="form-row"><label htmlFor="new-platform">Tipo de servidor</label><select id="new-platform" value={input.platform} onChange={e => {setVersionLoading(true);setVersionError(null);if(!['vanilla','fabric'].includes(e.target.value))setExperimental(false);setInput({...input,platform:e.target.value as CreateInput['platform']})}}><option value="vanilla">Vanilla</option><option value="paper">Paper</option><option value="fabric">Fabric</option><option value="neoforge">NeoForge</option></select></div></div><p className="field-help type-help">{input.platform === 'neoforge' ? 'NeoForge permite mods de servidor. Se elegirá una versión estable compatible y su Java necesario.' : input.platform === 'fabric' ? 'Fabric permite mods de servidor. Se elegirá automáticamente un loader estable compatible.' : input.platform === 'paper' ? 'Paper admite plugins y opciones extra de rendimiento.' : 'Vanilla ofrece la experiencia original de Minecraft.'}</p>
      {['vanilla','fabric'].includes(input.platform)&&<div className="experimental-versions"><label><input type="checkbox" checked={experimental} disabled={busy} onChange={event=>{setVersionLoading(true);setExperimental(event.target.checked)}}/><span>Mostrar versiones experimentales</span></label>{experimental&&<p className="field-help">Incluye snapshots y versiones de prueba del catálogo oficial. Pueden fallar y cambiar el formato de los mundos; crea una instancia separada para probarlas.</p>}</div>}
      </>}
      {step === 2 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>Elige un mundo</h2><p>Genera uno nuevo o importa un mundo de Minecraft Java.</p></div>
      <div className="world-choice"><label><input type="radio" name="world-mode" checked={worldMode === 'new'} onChange={() => setWorldMode('new')}/><span><strong>Mundo nuevo</strong><small>Minecraft lo generará en el primer inicio.</small></span></label><label><input type="radio" name="world-mode" checked={worldMode === 'existing'} onChange={() => setWorldMode('existing')}/><span><strong>Usar un mundo existente</strong><small>Se copiará a este servidor; el original se conserva.</small></span></label></div>
      {worldMode === 'existing' && <div className="world-choice-source"><button type="button" onClick={chooseWorld}>Elegir carpeta del mundo</button>{worldSource && <div><strong>{worldSource.suggestedName}</strong><span>{fmtBytes(worldSource.sizeBytes)} · {worldSource.path}</span></div>}{worldError && <p className="form-error" role="alert">{worldError}</p>}</div>}
      </>}
      {step === 3 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>Capacidad del servidor</h2><p>Reserva memoria para el sistema y para jugar.</p></div>
      <div className="capacity-grid"><SliderField id="new-memory-min" label="RAM inicial" value={input.memoryMinMb} min={512} max={Math.max(input.memoryMinMb,input.memoryMaxMb)} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMinMb:value,memoryMaxMb:value >= 512 && value <= 65536 ? Math.max(input.memoryMaxMb,value) : input.memoryMaxMb})}/><SliderField id="new-memory-max" label="RAM máxima" value={input.memoryMaxMb} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMaxMb:value,memoryMinMb:value >= 512 && value <= 65536 ? Math.min(input.memoryMinMb,value) : input.memoryMinMb})}/><SliderField id="new-players" label="Jugadores máximos" value={input.maxPlayers} min={1} max={Math.max(100,input.maxPlayers)} maxInput={1000} step={1} onChange={value => setInput({...input,maxPlayers:value})}/><SliderField id="new-storage" label="Objetivo de espacio" value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/></div><p className="field-help">{systemMemory ? `Tu equipo tiene ${fmtBytes(systemMemory)} de RAM.` : 'Deja memoria libre para el sistema y para jugar.'} El objetivo de espacio sirve para avisarte; no impone una cuota al disco.</p>{systemMemoryMb > 0 && input.memoryMaxMb > systemMemoryMb * .75 && <p className="inline-note" role="status">La RAM máxima supera el 75 % de la memoria física. Deja recursos para el sistema y para jugar.</p>}
      <details className="advanced"><summary>Opciones de red</summary><div className="form-row"><label htmlFor="new-port">Puerto del servidor</label><input id="new-port" type="number" min="1" max="65535" value={input.port} onChange={e => setInput({...input,port:Number(e.target.value)})}/></div></details>
      {invalidCapacity && <p className="form-error" role="alert">Revisa RAM, jugadores, espacio y puerto antes de continuar.</p>}
      </>}
      {step === 4 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>Revisar y crear</h2><p>Comprueba los datos antes de instalar.</p></div>
      <div className="wizard-review"><div><span>Identidad</span><strong>{input.name.trim() || 'Sin nombre'}</strong><button type="button" className="text-button" onClick={() => moveStep(0)}>Editar</button></div><div><span>Versión y tipo</span><strong>{input.version || 'Sin versión'}{versions.find(version=>version.id===input.version)?.kind==='snapshot'?' · Experimental':''} · {platformName(input.platform)}</strong><button type="button" className="text-button" onClick={() => moveStep(1)}>Editar</button></div><div><span>Mundo</span><strong>{worldMode === 'existing' ? `Copia de ${worldSource?.suggestedName || 'mundo pendiente'}` : 'Mundo nuevo'}</strong><button type="button" className="text-button" onClick={() => moveStep(2)}>Editar</button></div><div><span>Recursos</span><strong>{fmtBytes(input.memoryMinMb * 1024 * 1024)}–{fmtBytes(input.memoryMaxMb * 1024 * 1024)} · {input.maxPlayers} jugadores · {input.storageTargetGb} GB</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>Editar</button></div><div><span>Puerto</span><strong>{input.port}</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>Editar</button></div></div>
      <p className="wizard-final-note">La instalación descargará Java y los archivos del servidor. En la primera descarga te pediremos aceptar la EULA de Minecraft. La decisión se recordará en este equipo.</p>
      </>}
      </div>
      <div className="form-actions wizard-actions"><span>{step === steps.length - 1 ? 'Se descargarán Java y Minecraft.' : ''}</span><div>{step > 0 && <button type="button" disabled={busy} onClick={() => moveStep(step - 1)}>Anterior</button>}<button type="submit" className="primary-button" disabled={busy || !canAdvance || (step === steps.length - 1 && !readyToCreate)}>{busy ? 'Preparando…' : step === steps.length - 1 ? 'Crear e instalar' : 'Continuar'}<Icon name="arrow" size={16}/></button></div></div>
    </form><aside className="form-aside"><div className="aside-title">Tu configuración</div><div className="aside-server"><div className="image-preview small">{imagePreview ? <img src={imagePreview} alt=""/> : <Icon name="server" size={22}/>}</div><div><strong>{input.name.trim() || 'Nuevo servidor'}</strong><span>{platformName(input.platform)} · {input.version || 'Versión pendiente'}</span></div></div><dl><div><dt>Mundo</dt><dd>{worldMode === 'existing' ? worldSource?.suggestedName || 'Por elegir' : 'Nuevo'}</dd></div><div><dt>Memoria</dt><dd><AnimatedValue value={input.memoryMaxMb * 1024 * 1024} format={fmtBytes}/></dd></div><div><dt>Jugadores</dt><dd>Hasta <AnimatedValue value={input.maxPlayers}/></dd></div><div><dt>Objetivo de espacio</dt><dd><AnimatedValue value={input.storageTargetGb}/> GB</dd></div><div><dt>Puerto</dt><dd>{input.port}</dd></div></dl><p>El servidor se ejecutará en este ordenador. Java y los archivos se descargarán al crearlo.</p></aside></div>
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
      setInput(old => ({path:info.path,name:info.suggestedName,version:info.detectedVersion || versions[0]?.id || '',platform:info.jarFiles.some(j=>j.toLowerCase().includes('neoforge')) ? 'neoforge' : info.jarFiles.some(j=>j.toLowerCase().includes('fabric')) ? 'fabric' : info.jarFiles.some(j => j.toLowerCase().includes('paper')) ? 'paper':'vanilla',jarFile:info.jarFiles.find(j=>j.toLowerCase().includes('neoforge')) || info.jarFiles.find(j=>j.toLowerCase().includes('fabric')) || info.jarFiles[0] || '',copy:true,storageTargetGb:old.storageTargetGb,imagePath:old.imagePath}))
    } catch(e) { setFormError(errorText(e)) }
    finally { setPicking(false) }
  }
  const chooseImage = async () => {
    setFormError(null)
    try { const image = await api.pickServerImage(); if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) } }
    catch(e) { setFormError(errorText(e)) }
  }
  return <div className="form-page import-page"><div className="form-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/> Servidores</button><h1>Importar servidor</h1><p>Trae un servidor existente a tu biblioteca sin perder sus archivos.</p></div><div className="import-source"><div className="import-source-icon"><Icon name="folder" size={22}/></div><div><strong>{inspection ? 'Carpeta seleccionada' : 'Selecciona la carpeta del servidor'}</strong><p>{inspection ? inspection.path : 'Debe contener server.properties y un archivo de arranque compatible'}</p></div><button onClick={choose} disabled={picking || busy}>{picking ? 'Inspeccionando…' : inspection ? 'Cambiar carpeta' : 'Elegir carpeta'}</button></div>{formError && <p className="form-error" role="alert">{formError}</p>}{inspection && <form className="create-form import-form" onSubmit={e => { e.preventDefault(); onImport(input) }}><div className="form-section-title"><span>01</span><div><h2>Identidad</h2><p>Así aparecerá en tu biblioteca.</p></div></div><div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt="Vista previa de la imagen"/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage}>Elegir imagen</button><p>PNG, JPG o WebP · hasta 2 MB</p></div><div className="form-row"><label htmlFor="import-name">Nombre del servidor</label><input id="import-name" required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})}/></div></div><div className="form-section-title"><span>02</span><div><h2>Datos del servidor</h2><p>Revisa los valores detectados.</p></div></div><div className="form-pair"><div className="form-row"><label htmlFor="import-version">Versión de Minecraft</label><input id="import-version" list="import-versions" required value={input.version} onChange={e => setInput({...input,version:e.target.value})}/><datalist id="import-versions">{versions.map(v => <option key={v.id} value={v.id}/>)}</datalist></div><div className="form-row"><label htmlFor="import-platform">Tipo</label><select id="import-platform" value={input.platform} onChange={e => setInput({...input,platform:e.target.value as ImportInput['platform']})}><option value="vanilla">Vanilla</option><option value="paper">Paper</option><option value="fabric">Fabric</option><option value="neoforge">NeoForge</option></select></div></div><p className="field-help">Comprueba la versión del servidor original. La usaremos para elegir Java compatible.</p><div className="form-row"><label htmlFor="import-jar">Archivo del servidor</label><select id="import-jar" value={input.jarFile} onChange={e => setInput({...input,jarFile:e.target.value})}>{inspection.jarFiles.map(jar => <option key={jar} value={jar}>{jar}</option>)}</select></div><div className="import-storage"><SliderField id="import-storage" label="Objetivo de espacio" value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/><p className="field-help">Se usa para avisar cuando el servidor ocupa más espacio. No limita el disco.</p></div><fieldset className="import-mode"><legend>Ubicación de los archivos</legend><label><input type="radio" checked={input.copy} onChange={() => setInput({...input,copy:true})}/> Copiar a la biblioteca de TCHLauncher</label><label><input type="radio" checked={!input.copy} onChange={() => setInput({...input,copy:false})}/> Administrar la carpeta actual</label><p className="field-help">Si administras la carpeta actual, los cambios y las restauraciones afectarán a los archivos originales.</p></fieldset><div className="form-actions"><button type="button" onClick={onCancel}>Cancelar</button><button className="primary-button" disabled={busy || !input.version || input.storageTargetGb < 1 || input.storageTargetGb > 2048}>{busy ? 'Importando…' : 'Importar servidor'}<Icon name="arrow" size={16}/></button></div></form>}</div>
}

type SettingField = {key:string;label:string;description:string;kind:'text'|'number'|'toggle'|'choice';min?:number;max?:number;options?:{value:string;label:string}[]}
const settingsGroups: {title:string;description:string;fields:SettingField[]}[] = [
  {title:'Identidad y conexión',description:'Lo que verán los jugadores y cómo se conectarán.',fields:[
    {key:'motd',label:'Mensaje del servidor',description:'Aparece en la lista de servidores de Minecraft.',kind:'text'},
    {key:'server-port',label:'Puerto',description:'Puerto TCP usado para las conexiones.',kind:'number',min:1,max:65535},
    {key:'online-mode',label:'Verificación de cuentas',description:'Comprueba las cuentas con los servicios oficiales de Minecraft.',kind:'toggle'},
  ]},
  {title:'Juego',description:'Reglas y distancia que se aplican al mundo.',fields:[
    {key:'gamemode',label:'Modo de juego',description:'Modo inicial de los jugadores nuevos.',kind:'choice',options:[{value:'survival',label:'Supervivencia'},{value:'creative',label:'Creativo'},{value:'adventure',label:'Aventura'},{value:'spectator',label:'Espectador'}]},
    {key:'difficulty',label:'Dificultad',description:'Afecta al combate y a las criaturas.',kind:'choice',options:[{value:'peaceful',label:'Pacífica'},{value:'easy',label:'Fácil'},{value:'normal',label:'Normal'},{value:'hard',label:'Difícil'}]},
    {key:'pvp',label:'Combate entre jugadores',description:'Permite que los jugadores se dañen entre sí.',kind:'toggle'},
    {key:'view-distance',label:'Distancia de visión',description:'Chunks que el servidor envía a cada jugador.',kind:'number',min:2,max:32},
    {key:'simulation-distance',label:'Distancia de simulación',description:'Chunks donde el mundo sigue activo.',kind:'number',min:2,max:32},
  ]},
  {title:'Acceso',description:'Quién puede entrar y cómo se aplica la lista blanca.',fields:[
    {key:'white-list',label:'Lista blanca',description:'Solo permite jugadores añadidos a la lista.',kind:'toggle'},
    {key:'enforce-whitelist',label:'Expulsar a no autorizados',description:'Saca a quien deje de estar autorizado.',kind:'toggle'},
  ]},
]
const knownSettings = new Set(settingsGroups.flatMap(group => group.fields.map(field => field.key)))

function SettingsPropertyGroups({values,onChange}: {values:Record<string,string>;onChange:(values:Record<string,string>)=>void}) {
  const update = (key: string, value: string) => onChange({...values,[key]:value})
  return <div className="settings-groups">{settingsGroups.map(group => <section className="settings-group" key={group.title} aria-label={group.title}><div className="settings-group-heading"><h4>{group.title}</h4><p>{group.description}</p></div><div className="settings-fields">{group.fields.map(field => <label className="setting-line" key={field.key}><span className="setting-copy"><strong>{field.label}</strong><small>{field.description}</small></span>{field.kind === 'toggle' ? <input className="setting-toggle" type="checkbox" checked={values[field.key] === 'true'} onChange={event => update(field.key,String(event.target.checked))}/> : field.kind === 'choice' ? <select value={values[field.key] || field.options?.[0]?.value || ''} onChange={event => update(field.key,event.target.value)}>{field.options?.map(option => <option key={option.value} value={option.value}>{option.label}</option>)}</select> : <input type={field.kind} min={field.min} max={field.max} value={values[field.key] ?? ''} onChange={event => update(field.key,event.target.value)}/>}</label>)}</div></section>)}</div>
}

function Settings({onConsole,onRuntimeChanged,onPrepareRuntime,server,props,setProps,memory,setMemory,storageTarget,setStorageTarget,systemMemory,busy,progress,onSave,onPickImage,onClearImage,onDelete}: {onConsole:()=>void;onRuntimeChanged:()=>Promise<void>;onPrepareRuntime:(reinstall:boolean)=>void;server:Server;props:Record<string,string>;setProps:(p:Record<string,string>)=>void;memory:{min:number;max:number};setMemory:(v:{min:number;max:number})=>void;storageTarget:number;setStorageTarget:(value:number)=>void;systemMemory:number;busy:boolean;progress:Progress|null;onSave:()=>void;onPickImage:()=>void;onClearImage:()=>void;onDelete:()=>void}) {
  const [newKey,setNewKey] = useState('')
  const [newValue,setNewValue] = useState('')
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(memory.max,systemMemoryMb || 16384))
  const maxPlayers = Number(props['max-players'] || 20)
  const invalidCapacity = memory.min < 512 || memory.max < memory.min || memory.max > 65536 || storageTarget < 1 || storageTarget > 2048 || maxPlayers < 1 || maxPlayers > 1000 || ![memory.min,memory.max,maxPlayers,storageTarget].every(Number.isFinite)
  return <section className="settings-view"><div className="section-heading"><div><h2>Configuración</h2><p>Personaliza la apariencia y los ajustes del servidor.</p></div><button className="primary-button" disabled={busy || invalidCapacity || !['stopped','crashed','error'].includes(server.state)} onClick={onSave}>Guardar cambios <Icon name="check" size={16}/></button></div>
    <h3>Imagen del servidor</h3><div className="settings-image"><ServerArtwork server={server}/><div><strong>{server.name}</strong><p>La misma imagen en el launcher y en la lista de Minecraft. Se adapta a 64 × 64. PNG, JPG o WebP, hasta 2 MB.</p><div className="settings-image-actions"><button disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onPickImage}>{server.imageData ? 'Cambiar imagen' : 'Elegir imagen'}</button>{server.imageData && <button className="text-button" disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onClearImage}>Quitar imagen</button>}</div></div></div>
    {!['stopped','crashed','error'].includes(server.state) && <p className="inline-note">Detén el servidor para editar su configuración.</p>}
    <RuntimePanel key={server.id} onChanged={onRuntimeChanged} server={server} busy={busy} onPrepare={onPrepareRuntime}/>
    <JvmPanel key={`jvm-${server.id}`} server={server} busy={busy}/>
    <MinecraftUpdatePanel key={`minecraft-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged} currentProgress={progress} onConsole={onConsole}/>
    <LoaderPanel key={`loader-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged}/>
    <TasksPanel key={`tasks-${server.id}`} id={server.id}/>
    <h3>Capacidad</h3><div className="capacity-grid settings-capacity"><SliderField id="settings-memory-min" label="RAM inicial" value={memory.min} min={512} max={Math.max(memory.min,memory.max)} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,min:value,max:value >= 512 && value <= 65536 ? Math.max(memory.max,value) : memory.max})}/><SliderField id="settings-memory-max" label="RAM máxima" value={memory.max} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,max:value,min:value >= 512 && value <= 65536 ? Math.min(memory.min,value) : memory.min})}/><SliderField id="settings-players" label="Jugadores máximos" value={maxPlayers} min={1} max={Math.max(100,maxPlayers)} maxInput={1000} step={1} onChange={value => setProps({...props,'max-players':String(value)})}/><SliderField id="settings-storage" label="Objetivo de espacio" value={storageTarget} min={1} max={Math.max(200,storageTarget)} maxInput={2048} step={1} unit="GB" onChange={setStorageTarget}/></div><p className="field-help">{systemMemory ? `Memoria física: ${fmtBytes(systemMemory)}.` : ''} El objetivo de espacio solo activa avisos; no impone una cuota al disco.</p>{systemMemoryMb > 0 && memory.max > systemMemoryMb * .75 && <p className="inline-note" role="status">La RAM máxima supera el 75 % de la memoria física.</p>}{invalidCapacity && <p className="form-error" role="alert">Revisa los límites de RAM, jugadores y espacio antes de guardar.</p>}
    <h3>Propiedades del servidor</h3><SettingsPropertyGroups values={props} onChange={setProps}/>
    <details className="advanced-properties"><summary>Otras propiedades</summary><div className="settings-fields">{Object.entries(props).filter(([key]) => key !== 'max-players' && !knownSettings.has(key)).map(([key,value]) => <label className="setting-line" key={key}><span>{key}</span><input value={value} onChange={e => setProps({...props,[key]:e.target.value})}/></label>)}</div><div className="add-property"><input aria-label="Nombre de propiedad" placeholder="Nombre de propiedad" value={newKey} onChange={e => setNewKey(e.target.value)}/><input aria-label="Valor" placeholder="Valor" value={newValue} onChange={e => setNewValue(e.target.value)}/><button onClick={() => { if (newKey.trim()) { setProps({...props,[newKey.trim()]:newValue}); setNewKey(''); setNewValue('') } }}>Añadir</button></div></details>
    <div className="delete-server-section"><div><h3>Eliminar servidor</h3><p>Quita este servidor de la biblioteca. Si sus archivos están guardados aquí, también se eliminarán junto con sus copias de seguridad.</p></div><button className="danger-outline-button" disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onDelete}>Eliminar servidor</button></div>
  </section>
}

export default App




