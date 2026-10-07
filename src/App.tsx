import {translatedMessage} from './locale.ts'
import {localizedStatic,t,getLanguage} from './locale.ts'
import {acknowledgedAction} from './acknowledgedAction'
import {useLanguage} from './useLanguage'
import {createLogBatch} from './logBatch'
import {appendConsoleLines,mergeConsoleHistory,type ConsoleLine} from './consoleTimeline'
import {startVisibilityPolling} from './visibilityPolling'
import { PackImport } from './PackImport'
import {deferredPanel} from './DeferredPanel'
import {useServerRead} from './useServerRead'
import {SettingsPropertyGroups,knownSettings} from './SettingsProperties'
import {useBackupList} from './useBackupList'
import {useServerProperties} from './useServerProperties'
import {saveState} from './saveState'
import {version as appVersion} from '../package.json'
import {PixelIcon,PixelMeter} from './PixelIndicators'
import {CrashPanel} from './CrashPanel'
import { useCallback, useEffect, useRef, useState, type ComponentProps } from 'react'
import { createPortal } from 'react-dom'
import { listen } from '@tauri-apps/api/event'
import { isTauri } from '@tauri-apps/api/core'
import { api, type Backup, type CreateInput, type DeleteOutcome, type ImportInput, type ImportInspection, type LogLine, type Metrics, type PlayersSnapshot, type Progress, type Server, type ServerState, type Version, type WorldInspection } from './api'
import {StartupProgress} from './StartupProgress'
import {StorageMeter} from './StorageMeter'
import {acceptStartupEvent,type StartupEvent} from './startupEvents'
import { SliderField } from './SliderField'
import { ConsoleView } from './ConsoleView'


import { NetworkPanel } from './NetworkPanel'
import { Modal } from './Modal'
import {ForceStopDialog} from './ForceStopDialog'

import { InstallProgress } from './InstallProgress'
import { RuntimePanel } from './RuntimePanel'
import { JvmPanel } from './JvmPanel'
import { BackupPolicyPanel } from './BackupPolicyPanel'
import { TasksPanel } from './TasksPanel'
import { StartupPanel } from './StartupPanel'
import { LoaderPanel } from './LoaderPanel'
import { MinecraftUpdatePanel } from './MinecraftUpdatePanel'

import { ServerLibrary } from './ServerLibrary'

import { applyAppearance, browserPreferenceStorage, readAppearance, saveAppearance, type Appearance } from './appearance'
import launcherEmblem from './assets/launcher-logo.png'
import worldIllustration from './assets/choice-world.png'
import { motion, MovingIndicator, AnimatedValue, useListMotion, useNavigationMotion } from './motion'
import { ServerMenu, type ServerMenuTarget } from './ServerMenu'
import { ServerDeleteDialog } from './ServerDeleteDialog'
import { recordSample, type MetricSample } from './telemetry'
import {CommandPalette} from './CommandPalette'
import {launcherCommands,type LauncherCommand} from './commands'
import {BackgroundTasks} from './BackgroundTasks'
import {dismissTask,isTaskActive,reconcileTasks} from './taskState'

const PlayersView=deferredPanel<ComponentProps<typeof import('./PlayersView').PlayersView>>(t("Jugadores"),()=>import('./PlayersView').then(module=>({default:module.PlayersView})))
const WorldView=deferredPanel<ComponentProps<typeof import('./WorldView').WorldView>>(t("Mundos"),()=>import('./WorldView').then(module=>({default:module.WorldView})))
const ContentManager=deferredPanel<ComponentProps<typeof import('./ContentManager').ContentManager>>(t("Mods y plugins"),()=>import('./ContentManager').then(module=>({default:module.ContentManager})))
const ResourceMonitor=deferredPanel<ComponentProps<typeof import('./ResourceMonitor').ResourceMonitor>>(t("Rendimiento"),()=>import('./ResourceMonitor').then(module=>({default:module.ResourceMonitor})))
const ApplicationSettings=deferredPanel<ComponentProps<typeof import('./ApplicationSettings').ApplicationSettings>>(t("Ajustes"),()=>import('./ApplicationSettings').then(module=>({default:module.ApplicationSettings})))


type View = 'overview' | 'performance' | 'players' | 'worlds' | 'content' | 'console' | 'backups' | 'settings'
const backupReason:Record<string,string>=localizedStatic({manual:'Manual',scheduled:'Programada',server_stop:'Tras detener el servidor',before_restore:'Antes de restaurar',before_minecraft_update:'Antes de actualizar Minecraft',before_loader_update:'Antes de actualizar el loader',before_content_update:'Antes de actualizar contenido',before_content_removal:'Antes de quitar contenido',before_world_change:'Antes de cambiar mundo',before_world_deletion:'Antes de borrar mundo'})
const labels: Record<ServerState, string> = localizedStatic({
  stopped: 'Detenido', installing: 'Instalando', starting: 'Iniciando',
  running: 'En ejecución', stopping: 'Deteniendo', crashed: 'Se ha cerrado inesperadamente', error: 'Error',
})
const viewOrder:View[]=['overview','performance','players','worlds','content','console','backups','settings']
const viewLabels:Record<View,string>=localizedStatic({overview:'Resumen',performance:'Rendimiento',players:'Jugadores',worlds:'Mundos',content:'Mods',console:'Consola',backups:'Copias',settings:'Configuración'})
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
  useLanguage()
  const [servers, setServers] = useState<Server[]>([])
  const [railHovered,setRailHovered] = useState(false)
  const [railKeyboardFocus,setRailKeyboardFocus] = useState(false)
  const [railTasksOpen,setRailTasksOpen] = useState(false)
  const [sidebarTooltip,setSidebarTooltip] = useState<{name:string;subtitle:string;top:number;left:number}|null>(null)
  const [menuTarget,setMenuTarget] = useState<ServerMenuTarget|null>(null)
  const sidebarCollapsed = !railHovered && !railKeyboardFocus && !menuTarget && !railTasksOpen
  const menuTargetRef = useRef<ServerMenuTarget|null>(null)
  const [deleteTarget,setDeleteTarget] = useState<Server|null>(null)
  const [forceStopTarget,setForceStopTarget]=useState<Server|null>(null)
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
  const navigate=(next:View)=>{setConsoleQuery('');setNavigationDirection(viewOrder.indexOf(next)>viewOrder.indexOf(view)?1:-1);setView(next)}
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
  const playerViewVisible=!libraryOpen&&!creating&&!importing&&!appearanceOpen&&['overview','players'].includes(view)
  const playerProcessActive=servers.some(server=>server.id===selectedId&&['starting','running','stopping'].includes(server.state))
  const {data:players,loading:playersLoading,error:playersError,refresh:refreshPlayers}=useServerRead(selectedId,api.players,playerViewVisible&&playerProcessActive?3000:0)
  const {data:logHistory,loading:logLoading,error:logError,refresh:refreshLog}=useServerRead(selectedId,api.log)
  const [consoleQuery,setConsoleQuery]=useState('')
  useEffect(()=>{if(view!=='console'||libraryOpen||creating||importing||appearanceOpen)setConsoleQuery('')},[view,libraryOpen,creating,importing,appearanceOpen])
  const [logs, setLogs] = useState<ConsoleLine[]>([])
  const logArrival=useRef(0)
  const logBatch=useRef<ReturnType<typeof createLogBatch<LogLine>>|null>(null)
  useEffect(()=>{
    const batch=createLogBatch<LogLine>(1000,lines=>{const incoming=lines.map(line=>({...line,arrival:++logArrival.current}));setLogs(current=>appendConsoleLines(current,incoming))})
    logBatch.current=batch
    return()=>{batch.dispose();if(logBatch.current===batch)logBatch.current=null}
  },[])
  useEffect(()=>{logBatch.current?.reset()},[selectedId])
  const {copies:backups,loaded:backupsLoaded,loading:backupsLoading,error:backupsError,refresh:refreshBackups}=useBackupList(selectedId)
  const [restoreSync,setRestoreSync]=useState<Record<string,{loading:boolean;error:string|null}>>({})
  const restoreSyncRevision=useRef<Record<string,number>>({})
  const [restoreTarget,setRestoreTarget]=useState<{serverId:string;backup:Backup}|null>(null)
  const {values:props,loaded:propsLoaded,loading:propsLoading,error:propsError,dirty:propsDirty,edit:setProps,refresh:refreshProperties,acknowledge:acknowledgeProperties}=useServerProperties(selectedId,servers.find(server=>server.id===selectedId)?.state)
  const [capacityDirty,setCapacityDirty]=useState(false)
  const capacityDirtyRef=useRef(false)
  const capacityServer=useRef<string|null>(null)
  const [eula, setEula] = useState(false)
  const [downloadConsent,setDownloadConsent] = useState<{id:string;kind:Exclude<Progress['kind'],'content_install'|'pack_install'|'loader_update'|'minecraft_update'|'backup'|'restore'|'world_import'|'world_duplicate'|'world_export'|'world_activate'|'world_delete'|'server_import'|'content_remove'|'content_toggle'>}|null>(null)
  const [consentBusy,setConsentBusy] = useState(false)
  const [consentError,setConsentError] = useState<string|null>(null)
  const [memory, setMemory] = useState({ min: 1024, max: 2048 })
  const [storageTarget, setStorageTarget] = useState(20)
  const [systemMemory, setSystemMemory] = useState(0)
  const [closing, setClosing] = useState(false)
  const [startupOpen,setStartupOpen]=useState(false)
  const [startupBusy,setStartupBusy]=useState(false)
  const [closingBusy, setClosingBusy] = useState(false)
  const [closeError,setCloseError]=useState<string|null>(null)
  const closeOwner=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};closeOwner.current=current;return()=>{current.active=false}},[])
  const closeApplication=async(action:'hide'|'stop')=>{
    const current=closeOwner.current;if(!current.active||current.working)return
    current.working=true;setClosingBusy(true);setCloseError(null)
    let exitRequested=false
    try{
      if(action==='hide'){await api.hideWindow();if(current.active)setClosing(false)}
      else {await api.stopAllAndExit();exitRequested=true}
    }catch(cause){if(current.active)setCloseError(errorText(cause))}
    finally{if(!exitRequested){current.working=false;if(current.active)setClosingBusy(false)}}
  }
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
        if(event.payload.serverId===selectedRef.current&&['stopped','crashed','error'].includes(event.payload.state))void refreshPlayers(event.payload.serverId)
      }),
      listen<StartupEvent>('server-startup',event=>setStartupPhases(current=>acceptStartupEvent(current,event.payload))),
      listen<Progress>('install-progress', event => {progressRevision.current++;progressRef.current={...progressRef.current,[event.payload.serverId]:event.payload};setProgressByServer(progressRef.current)}),
      listen<LogLine>('server-log', event => {
        if (event.payload.serverId === selectedRef.current) logBatch.current?.push(event.payload)
      }),
      listen('close-task-active',()=>setError(t("Espera a que termine la operación antes de cerrar. Puedes cancelar la copia desde Tareas."))),
      listen('close-with-servers', () => {setClosing(true);if(!closeOwner.current.working)setCloseError(null)}),
      listen<[string,string]>('startup-result',event=>{if(event.payload[1].startsWith('Error:'))setError(t("Inicio automático de «{0}»: {1}", event.payload[0], event.payload[1]))}),
      listen<string>('scheduler-updated',event=>{
        const id=event.payload
        if(id===selectedRef.current)void refreshBackups(id).catch(()=>{})
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
    const restoredRevision=restoreSyncRevision.current[selectedId]??0
    setMetrics(null); setMetricsError(null); setConsoleQuery(''); setLogs([]); setEula(false)
    api.eulaAccepted(selectedId).then(accepted=>{
      if(active&&restoredRevision===(restoreSyncRevision.current[selectedId]??0))setEula(accepted)
    }).catch(e=>{if(active)setError(errorText(e))})
    return () => { active = false }
  }, [selectedId])

  useEffect(()=>{
    if(!logHistory||!selectedId)return
    setLogs(current=>mergeConsoleHistory(current,logHistory,selectedId))
  },[logHistory,selectedId])

  useEffect(() => {
    if (!selected) return
    if(capacityServer.current!==selected.id){capacityServer.current=selected.id;capacityDirtyRef.current=false;setCapacityDirty(false)}
    if(capacityDirtyRef.current)return
    setMemory({ min: selected.memoryMinMb, max: selected.memoryMaxMb })
    setStorageTarget(selected.storageTargetGb)
  }, [selected?.id, selected?.memoryMinMb, selected?.memoryMaxMb, selected?.storageTargetGb])

  useEffect(() => {
    if (!selectedId) return
    let active = true
    const poll = async () => {
      const generation=metricsGeneration.current[selectedId]??0
      const results = await Promise.allSettled([api.metrics(selectedId)])
      if (!active) return
      const [sample] = results
      if(generation===(metricsGeneration.current[selectedId]??0)){
        if (sample.status === 'fulfilled') { setMetrics(sample.value); setMetricsError(null) }
        else setMetricsError(t("No se pudo leer el proceso: {0}", errorText(sample.reason)))
      }
    }
    const stop=startVisibilityPolling(poll,3000,{hiddenMs:15000,immediate:true})
    return () => { active = false;stop() }
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

  const [actionRefreshError,setActionRefreshError]=useState<string|null>(null)
  const act = async (fn: () => Promise<unknown>) => {
    setError(null);setActionRefreshError(null);setBusy(true)
    try {const result=await acknowledgedAction(fn,refresh);setActionRefreshError(result.refreshError);return true}
    catch (e) { if(errorText(e)!=='Instalación cancelada')setError(errorText(e)) }
    finally { setBusy(false) }
    return false
  }

  const syncRestored=async(id:string)=>{
    const revision=(restoreSyncRevision.current[id]??0)+1;restoreSyncRevision.current[id]=revision
    setRestoreSync(current=>({...current,[id]:{loading:true,error:null}}))
    const results=await Promise.allSettled([refreshProperties(id,true),refreshBackups(id),api.eulaAccepted(id),refresh()])
    if(restoreSyncRevision.current[id]!==revision)return
    const [,,consent]=results
    if(selectedRef.current===id){
      if(consent.status==='fulfilled')setEula(consent.value)
    }
    const labels=['propiedades','copias','EULA','servidores']
    const failures=results.flatMap((result,index)=>result.status==='rejected'?[`${labels[index]}: ${errorText(result.reason)}`]:[])
    setRestoreSync(current=>{const next={...current};if(failures.length)next[id]={loading:false,error:t("La copia se restauró, pero faltan datos por actualizar. {0}", failures.join(' · '))};else delete next[id];return next})
  }
  const saveSettings=async(id:string)=>{
    if(!propsLoaded||propsLoading||propsError)return
    const values={...props};const capacity={...memory};const target=storageTarget
    setError(null);setBusy(true)
    try{
      await api.saveServerSettings(id,values,capacity.min,capacity.max,target)
      acknowledgeProperties(id,values)
      if(selectedRef.current===id){capacityDirtyRef.current=false;setCapacityDirty(false)}
      try{await refresh()}catch(cause){setError(t("Ajustes guardados. No se pudo actualizar el servidor: {0}", errorText(cause)))}
    }catch(cause){setError(errorText(cause))}
    finally{setBusy(false)}
  }
  const createCopy=async(id:string)=>{
    setError(null);setBusy(true)
    try{
      const copy=await api.createBackup(id)
      const results=await Promise.allSettled([refreshBackups(id),refresh()])
      const failures=results.flatMap(result=>result.status==='rejected'?[errorText(result.reason)]:[])
      const warnings=[copy.retentionWarning,...failures].filter(Boolean)
      if(warnings.length)setError(t("La copia se creó. {0}", warnings.join(' · ')))
    }catch(cause){setError(errorText(cause))}
    finally{setBusy(false)}
  }

  const runInstallation = async (id:string,kind:Exclude<Progress['kind'],'content_install'|'pack_install'|'loader_update'|'minecraft_update'|'backup'|'restore'|'world_import'|'world_duplicate'|'world_export'|'world_activate'|'world_delete'|'server_import'|'content_remove'|'content_toggle'>='install')=>{
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
        ? t("Se quitó «{0}» de la biblioteca. Algunos archivos no se pudieron limpiar: {1}{2}", server.name, outcome.cleanupWarnings.join(' '), outcome.retainedDirectory ? ` La carpeta original sigue en ${outcome.retainedDirectory}.` : '')
        : outcome.retainedDirectory
          ? t("Se quitó «{0}» de la biblioteca. La carpeta original permanece en {1}.", server.name, outcome.retainedDirectory)
          : t("Se eliminó «{0}» y sus archivos locales.", server.name))
      return outcome
    } catch (cause) {
      setError(errorText(cause))
      throw cause
    } finally { setBusy(false) }
  }

  useListMotion(backupListRef,`${view}:${backups.map(backup=>backup.id).join()}`)
  useListMotion(railRef,servers.map(server=>server.id).join(','))
  useNavigationMotion(contentRef,`${selectedId}:${view}:${appearanceOpen}:${creating}:${importing}:${libraryOpen}`,navigationDirection)
  const selectedName = selected?.name ?? t("Servidores")
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
      void createCopy(server.id)
    }
  }
  return <div className={`shell auto-rail ${sidebarCollapsed ? 'sidebar-collapsed' : 'rail-expanded'}`}>
    {paletteOpen&&<CommandPalette commands={commands} onClose={()=>setPaletteOpen(false)} onExecute={executeCommand}/>}
    <aside className="server-rail" aria-label={t("Servidores")} onPointerEnter={event=>{if(event.pointerType!=='touch')setRailHovered(true)}} onPointerLeave={()=>setRailHovered(false)} onPointerDown={()=>setRailKeyboardFocus(false)} onFocusCapture={event=>{if(event.target.matches(':focus-visible'))setRailKeyboardFocus(true)}} onKeyDownCapture={event=>{if(event.key==='Tab')setRailKeyboardFocus(true)}} onBlurCapture={event=>{if(!(event.relatedTarget instanceof Node)||!event.currentTarget.contains(event.relatedTarget))setRailKeyboardFocus(false)}}>
      <button className={`rail-library ${libraryOpen && !creating && !importing && !appearanceOpen?'active':''}`} onClick={openLibrary} aria-label={t("Mis servidores")} title={t("Mis servidores")}><span className="brand-emblem"><img src={launcherEmblem} alt=""/></span><strong className="rail-brand-name">TCHLauncher</strong></button>
      <nav className="server-list" ref={railRef} aria-label={t("Lista de servidores")} onScroll={() => hideSidebarTooltip()}>
        <MovingIndicator active={`${selectedId}:${creating}:${importing}:${libraryOpen}:${appearanceOpen}:${servers.map(s=>s.id).join()}`} className="rail-active-indicator"/>{servers.map(server => <button data-layout-id={server.id} key={server.id} className={`server-row ${selectedId === server.id && !creating && !importing && !libraryOpen && !appearanceOpen ? 'active' : ''}`} onClick={() => { hideSidebarTooltip(); setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(server.id); setCreating(false); setImporting(false); setView('overview') }} onContextMenu={event=>{event.preventDefault();hideSidebarTooltip();const rect=event.currentTarget.getBoundingClientRect();const target={server,x:event.clientX || rect.right,y:event.clientY || rect.top,trigger:event.currentTarget};menuTargetRef.current=target;setMenuTarget(target)}} onMouseEnter={event => showSidebarTooltip(server,event.currentTarget)} onMouseLeave={() => hideSidebarTooltip()} onFocus={event => showSidebarTooltip(server,event.currentTarget,true)} onBlur={() => hideSidebarTooltip()} aria-label={`${server.name}, ${platformName(server.platform)} ${server.version}, ${stateText(server.state)}`} aria-current={selectedId === server.id && !creating && !importing && !libraryOpen && !appearanceOpen ? 'page' : undefined}>
          <span className="server-artwork-wrap"><ServerArtwork server={server}/>{server.pinned && <span className="server-pin" title={t("Servidor fijado")} aria-label={t("Fijado")}><svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="m15 3 6 6-4 1-4 5-4-4 5-4zM9 15l-6 6"/></svg></span>}<span className={`state-dot ${server.state}`} aria-hidden="true"/></span><span className="server-row-name">{server.name}<small>{platformName(server.platform)} · {server.version}</small></span>
        </button>)}
      </nav>
      <button className="rail-add" onClick={openCreate} aria-label={t("Nuevo servidor")} title={t("Nuevo servidor")}><Icon name="plus" size={19}/><span>{t("Nuevo servidor")}</span></button>
      <button className="rail-import" onClick={openImport} aria-label={t("Importar servidor")} title={t("Importar servidor")}><Icon name="folder" size={18}/><span>{t("Importar servidor")}</span></button>
      <BackgroundTasks tasks={progressByServer} servers={servers} error={taskSyncError} onOpenChange={setRailTasksOpen} onOpen={task=>{const server=servers.find(item=>item.id===task.serverId);if(!server)return;openServer(server);if((task.kind==='backup'||task.kind==='restore'))setView('backups');if((task.kind==='world_import'||task.kind==='world_duplicate'||task.kind==='world_export'||task.kind==='world_activate'||task.kind==='world_delete'))setView('worlds');if(task.kind==='content_install'||task.kind==='content_remove'||task.kind==='content_toggle')setView('content');if(task.kind==='loader_update'||task.kind==='minecraft_update')setView('settings')}} onDismiss={task=>{progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>
      <button className="rail-startup" onClick={()=>setStartupOpen(true)} aria-label={t("Inicio automático")} title={t("Inicio automático")}><SectionIcon name="settings"/><span>{t("Inicio automático")}</span></button>
      <button className={`rail-settings ${appearanceOpen?'active':''}`} onClick={()=>{setAppearanceOpen(true);setCreating(false);setImporting(false);hideSidebarTooltip();closeMenu()}} aria-current={appearanceOpen?'page':undefined} aria-label={t("Ajustes")} title={t("Ajustes")}><SectionIcon name="appearance"/><span>{t("Ajustes")}</span></button>
      <span className="rail-version">v{appVersion}</span>
    </aside>
    <main className="main" ref={mainRef}>
      {actionRefreshError&&<div className="panel-read-state failed" role="alert"><div><strong>{t("Acción completada · datos sin actualizar")}</strong><p>{translatedMessage(actionRefreshError)}</p></div><button disabled={operationBusy} onClick={async()=>{try{await refresh();setActionRefreshError(null)}catch(cause){setActionRefreshError(errorText(cause))}}}>{t("Actualizar datos")}</button><button className="text-button" onClick={()=>setActionRefreshError(null)}>{t("Cerrar aviso")}</button></div>}
      {error && <div className="error-banner" role="alert"><div><strong>{t("No se pudo completar la acción")}</strong><p>{translatedMessage(error)}</p></div><button onClick={() => setError(null)} aria-label={t("Cerrar error")}>×</button></div>}
      {notice && <div className="notice-banner" role="status"><span>{translatedMessage(notice)}</span><button onClick={() => setNotice(null)} aria-label={t("Cerrar aviso")}>×</button></div>}
      {appearanceOpen ? <ApplicationSettings appearance={appearance} saved={appearanceSaved} onAppearance={changeAppearance} servers={servers} selectedId={selectedId} onSelect={setSelectedId} busy={operationBusy} progress={progressByServer} onPrepare={(id,reinstall)=>void runInstallation(id,reinstall?'runtime_reinstall':'runtime')} onChanged={refresh} onOpenServer={(server,section)=>{openServer(server);setView(section)}} onDismiss={task=>{progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>
      : creating ? <CreateView onCancel={openLibrary} onCreate={create} busy={operationBusy} systemMemory={systemMemory}/>
      : importing ? <>{operationBusy && Object.values(progressByServer).filter(task=>task.kind==='server_import'&&isTaskActive(task)).map(task=><InstallProgress key={task.taskId} progress={task} busy={true} allowRetry={false} onRetry={()=>{}} onDismiss={()=>{}}/>)}<ImportView onCancel={openLibrary} onImport={importExisting} busy={operationBusy}/></>
      : libraryOpen || !selected ? <ServerLibrary servers={servers} onCreate={openCreate} onImport={openImport} onPack={()=>{setPackResume(null);setPackOpen(true)}} onSelect={openServer} onMenu={openMenu}/>
      : <>
        <header className="page-head">
          <div className="page-identity"><ServerArtwork server={selected}/><div><h1>{selectedName}</h1><div className="page-context">{platformName(selected.platform)} <span>·</span> Minecraft {selected.version}</div></div></div>
          <div className="head-actions">
            {selected.state==='stopping'&&<button className="danger-button" onClick={()=>setForceStopTarget(selected)}>{t("Forzar cierre…")}</button>}
            {view !== 'overview' && <span className={`status-label ${selected.state}`}><span className={`state-dot ${selected.state}`} aria-hidden="true"/>{stateText(selected.state)}</span>}
            {selected.jarPath && selected.javaPath ? selected.state === 'running' ? <>
              <button disabled={busy} onClick={() => act(() => api.restart(selected.id))}>{t("Reiniciar")}</button>
              <button className="primary-button" disabled={busy} onClick={() => act(() => api.stop(selected.id))}>{t("Detener")}</button>
            </> : selected.state === 'starting' ? <button disabled={busy} onClick={() => act(() => api.stop(selected.id))}>{t("Detener inicio")}</button> : selected.state === 'stopped' || selected.state === 'crashed' || selected.state === 'error' ? <button className="primary-button" disabled={busy || !eula} onClick={() => act(() => api.start(selected.id))}>{t("Iniciar servidor")} <Icon name="arrow" size={15}/></button> : null : <button className="primary-button" disabled={busy} onClick={() => runInstallation(selected.id)}>{selected.jarPath ? t("Preparar Java") : t("Instalar")} <Icon name="arrow" size={15}/></button>}
          </div>
        </header>
        <nav className="server-sections" aria-label={t("Secciones del servidor")}>
          <MovingIndicator active={`${selectedId}:${view}`} axis="both" className="section-active-indicator"/>
          {viewOrder.filter(item=>item!=='content'||selected.platform!=='vanilla').map(item=><button key={item} aria-current={view===item?'page':undefined} onClick={()=>navigate(item)}><SectionIcon name={item}/><span>{item==='content'&&selected.platform==='paper'?'Plugins':viewLabels[item]}</span></button>)}
        </nav>
        <div className="content" ref={contentRef}>
          {progress && !(view==='content'&&(progress.kind==='content_install'||progress.kind==='content_remove'||progress.kind==='content_toggle'))&&!(view==='settings'&&(progress.kind==='loader_update'||progress.kind==='minecraft_update')) && <InstallProgress key={progress.taskId} progress={progress} busy={busy} retryLabel={(progress.kind==='backup'||progress.kind==='restore')?t("Ver copias"):(progress.kind==='world_import'||progress.kind==='world_duplicate'||progress.kind==='world_export'||progress.kind==='world_activate'||progress.kind==='world_delete')?t("Ver mundos"):t("Reintentar")} onRetry={()=>{if(progress.kind==='server_import'){setView('overview');return;}if((progress.kind==='world_import'||progress.kind==='world_duplicate'||progress.kind==='world_export'||progress.kind==='world_activate'||progress.kind==='world_delete')){setView('worlds');return;}if((progress.kind==='backup'||progress.kind==='restore')){setView('backups');return;}if(progress.kind==='pack_install'){setPackResume(selected.id);setPackOpen(true);return;}if(progress.kind==='content_install'||progress.kind==='content_remove'||progress.kind==='content_toggle'){setView('content');return;}if((progress.kind==='loader_update'||progress.kind==='minecraft_update')){setView('settings');return;}void runInstallation(selected.id,progress.kind)}} onDismiss={()=>{const next={...progressRef.current};delete next[selected.id];progressRef.current=next;setProgressByServer(next)}}/>}
          {restoreSync[selected.id]&&<div className="restore-sync-notice"><p className={restoreSync[selected.id].error?'form-error':'field-help'} role={restoreSync[selected.id].error?'alert':'status'}>{restoreSync[selected.id].loading?t("Copia restaurada. Actualizando datos…"):restoreSync[selected.id].error}</p><button disabled={operationBusy||restoreSync[selected.id].loading} onClick={()=>void syncRestored(selected.id)}>{t("Actualizar datos")}</button></div>}
          {view === 'overview' && <Overview key={selected.id} server={selected} observedPhase={startupPhases[selected.id]?.phase} metrics={metrics?.serverId === selected.id ? metrics : null} players={players} playersStale={!!playersError} logs={logs} backups={backups} backupsStatus={backupsError?t("No se pudieron actualizar las copias."):!backupsLoaded?t("Cargando copias…"):backupsLoading?t("Actualizando copias…"):null} eula={eula} busy={busy} onAccept={() => act(async () => { await api.acceptEula(selected.id); setEula(true) })} onInstall={() => runInstallation(selected.id)} onConsole={() => setView('console')} onSettings={()=>setView('settings')} onContent={()=>setView('content')}/>}
          {view === 'performance' && <ResourceMonitor server={selected} metrics={metrics?.serverId === selected.id ? metrics : null} history={metricsHistory[selected.id] ?? []} error={metricsError}/>}
          {view === 'players' && <PlayersView key={selected.id} server={selected} players={players} loading={playersLoading} readError={playersError} busy={busy} onRefresh={() => refreshPlayers(selected.id)} onAction={(action,target,reason) => act(() => api.managePlayer(selected.id,action,target,reason))} onToggleWhitelist={async () => { const success=await act(async () => { if (selected.state === 'running') await api.managePlayer(selected.id,players?.whitelistEnabled ? 'whitelist_off' : 'whitelist_on'); else { const values = { ...await api.properties(selected.id),'white-list':String(!players?.whitelistEnabled) }; await api.saveProperties(selected.id,values) } });if(success&&selected.state!=='running')await refreshProperties(selected.id).catch(()=>{});return success }} onImportWhitelist={names => act(() => api.importWhitelist(selected.id,names))}/>}
          {view === 'worlds' && <WorldView server={selected} onBackupsChanged={()=>refreshBackups(selected.id)}/>}
          {view === 'content' && selected.platform!=='vanilla' && <ContentManager onImportPack={()=>{setPackResume(null);setPackOpen(true)}} key={selected.id} server={selected} progress={progress} logs={logs} onConsole={query=>{setConsoleQuery(query);setView('console')}} onInstalled={async()=>{await refresh();await refreshBackups(selected.id)}} onDismissProgress={()=>{const next={...progressRef.current};delete next[selected.id];progressRef.current=next;setProgressByServer(next)}}/>}
          {view === 'console'  && <ConsoleView initialQuery={consoleQuery} serverId={selected.id} state={selected.state} logs={logs} historyLoading={logLoading} historyError={logError} onReload={() => refreshLog(selected.id)} busy={busy} onSend={command => act(() => api.send(selected.id,command))}/> }
          {view === 'backups' && <section className="backup-view"><div className="section-heading"><div><h2>{t("Copias de seguridad")}</h2><p>{t("Se guardan en el almacenamiento local de la aplicación.")}</p></div><button disabled={busy || !['stopped','crashed','error'].includes(selected.state)} onClick={()=>void createCopy(selected.id)}>{t("Crear copia")}</button><button disabled={backupsLoading} onClick={()=>void refreshBackups(selected.id).catch(()=>{})}>{backupsLoading?t("Consultando…"):t("Actualizar lista")}</button></div><BackupPolicyPanel key={selected.id} id={selected.id} disabled={busy||!['stopped','crashed','error'].includes(selected.state)} onChanged={()=>refreshBackups(selected.id)}/>{!['stopped','crashed','error'].includes(selected.state) && <p className="inline-note">{t("Detén el servidor para crear una copia consistente.")}</p>}{backupsError&&<p className="form-error" role="alert">{t("No se pudo consultar la lista:")} {translatedMessage(backupsError)}{backupsLoaded?' Se muestran las últimas copias conocidas.':''}</p>}{!backupsLoaded&&!backupsError&&<p className="empty-inline" role="status">{t("Cargando copias…")}</p>}{backups.length ? <div className="table backup-table" ref={backupListRef}><div className="table-head"><span>{t("Fecha y motivo")}</span><span>{t("Tamaño")}</span><span>{t("Acción")}</span></div>{backups.map(backup => <div className="table-row" data-layout-id={backup.id} key={backup.id}><div className="backup-description"><span>{new Date(backup.createdAt).toLocaleString()}</span><strong>{backup.reason?backupReason[backup.reason]??t("Motivo no disponible"):t("Motivo no registrado")}</strong><small>{backup.minecraftVersion?`${backup.platform??t("Motor no registrado")} · Minecraft ${backup.minecraftVersion}`:t("Versión no registrada")}</small>{backup.metadataError&&<small className="form-error">{translatedMessage(backup.metadataError)}</small>}</div><span>{fmtBytes(backup.sizeBytes)}</span><button className="text-button" disabled={busy || backupsLoading || !!backupsError || !['stopped','crashed','error'].includes(selected.state)} onClick={()=>{setError(null);setRestoreTarget({serverId:selected.id,backup})}}>{t("Restaurar")}</button></div>)}</div> : backupsLoaded&&!backupsError?<div className="empty-inline">{t("Aún no hay copias de este servidor.")}</div>:null}</section>}
          {view === 'settings' && <Settings onConsole={()=>setView('console')} onRuntimeChanged={refresh} onPrepareRuntime={reinstall=>runInstallation(selected.id,reinstall?'runtime_reinstall':'runtime')} server={selected} props={props} setProps={setProps} propertiesLoaded={propsLoaded} propertiesLoading={propsLoading} propertiesError={propsError} propertiesDirty={propsDirty||capacityDirty} onReloadProperties={()=>void refreshProperties(selected.id).catch(()=>{})} memory={memory} setMemory={value=>{capacityDirtyRef.current=true;setCapacityDirty(true);setMemory(value)}} storageTarget={storageTarget} setStorageTarget={value=>{capacityDirtyRef.current=true;setCapacityDirty(true);setStorageTarget(value)}} systemMemory={systemMemory} busy={busy||!!restoreSync[selected.id]} progress={progress} onSave={()=>void saveSettings(selected.id)} onPickImage={() => act(async () => { const image = await api.pickServerImage(); if (image) await api.setServerImage(selected.id,image.path) })} onClearImage={() => act(() => api.clearServerImage(selected.id))} onDelete={() => setDeleteTarget(selected)}/>}
        </div>
      </>}
    </main>
    {sidebarTooltip && createPortal(<div className="sidebar-tooltip" role="tooltip" style={{top:sidebarTooltip.top,left:sidebarTooltip.left}}><strong>{sidebarTooltip.name}</strong><span>{sidebarTooltip.subtitle}</span></div>,document.body)}
    {menuTarget && <ServerMenu target={{...menuTarget,server:servers.find(server=>server.id===menuTarget.server.id) ?? menuTarget.server}} busy={operationBusy || activeInstallationIds.has(menuTarget.server.id)} onClose={closeMenu} onPin={()=>act(()=>api.setPinned(menuTarget.server.id,!menuTarget.server.pinned))} onSettings={()=>{setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(menuTarget.server.id);setCreating(false);setImporting(false);setView('settings')}} onDelete={()=>setDeleteTarget(menuTarget.server)}/>}
    {deleteTarget && <ServerDeleteDialog key={deleteTarget.id} server={deleteTarget} onClose={()=>setDeleteTarget(null)} onDelete={()=>removeServer(deleteTarget)}/>}
    {forceStopTarget&&<ForceStopDialog key={forceStopTarget.id} server={forceStopTarget} onClose={()=>setForceStopTarget(null)} onStopped={()=>void refresh()}/>}
    {startupOpen&&<Modal labelledBy="startup-title" busy={startupBusy} onClose={()=>setStartupOpen(false)}><h2 id="startup-title">{t("Inicio automático")}</h2><StartupPanel servers={servers} onBusy={setStartupBusy}/><div className="modal-actions"><button data-modal-close disabled={startupBusy}>{t("Cerrar")}</button></div></Modal>}
    {restoreTarget && <Modal labelledBy="restore-title" describedBy="restore-description" busy={operationBusy} onClose={()=>setRestoreTarget(null)}>
      <h2 id="restore-title">{t("Restaurar copia de seguridad")}</h2>
      <p id="restore-description">{t("Los archivos y la configuración volverán al estado de esta copia. Antes guardaremos una copia del estado actual.")}</p>
      <dl className="restore-preview-details"><dt>{t("Fecha")}</dt><dd>{new Date(restoreTarget.backup.createdAt).toLocaleString()}</dd><dt>{t("Tamaño")}</dt><dd>{fmtBytes(restoreTarget.backup.sizeBytes)}</dd><dt>{t("Motivo")}</dt><dd>{restoreTarget.backup.reason?backupReason[restoreTarget.backup.reason]??t("No disponible"):t("No registrado")}</dd><dt>{t("Servidor guardado")}</dt><dd>{restoreTarget.backup.minecraftVersion?`${restoreTarget.backup.platform??t("Motor no registrado")} · Minecraft ${restoreTarget.backup.minecraftVersion}`:t("Versión no registrada")}</dd></dl>
      {progressByServer[restoreTarget.serverId]?.kind==='restore'&&<InstallProgress key={progressByServer[restoreTarget.serverId].taskId} progress={progressByServer[restoreTarget.serverId]} busy={operationBusy} allowRetry={false} onRetry={()=>{}} onDismiss={()=>{const task=progressByServer[restoreTarget.serverId];progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>}
      {error && <p className="form-error" role="alert">{translatedMessage(error)}</p>}
      <div className="modal-actions"><button data-modal-close disabled={operationBusy}>{t("Cancelar")}</button><button className="primary-button" disabled={operationBusy} onClick={async()=>{
        const target=restoreTarget;setError(null);setBusy(true)
        try{await api.restoreBackup(target.serverId,target.backup.id);if(selectedRef.current===target.serverId){capacityDirtyRef.current=false;setCapacityDirty(false)}setRestoreTarget(null);await syncRestored(target.serverId)}
        catch(cause){setError(errorText(cause))}
        finally{setBusy(false)}
      }}>{operationBusy?t("Restaurando…"):t("Restaurar copia")}</button></div>
    </Modal>}
    {packOpen && <PackImport resumeId={packResume} systemMemory={systemMemory} onClose={()=>{setPackOpen(false);void refresh().catch(cause=>setError(errorText(cause)))}} onInstalled={async id=>{await refresh();setPackOpen(false);setAppearanceOpen(false);setLibraryOpen(false);setSelectedId(id);setView('overview')}}/>}
    {downloadConsent && <Modal labelledBy="download-consent-title" describedBy="download-consent-description" busy={consentBusy} onClose={()=>setDownloadConsent(null)}>
      <h2 id="download-consent-title">{t("Antes de descargar Minecraft")}</h2>
      <p id="download-consent-description">{t("Lee la")} <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">{t("EULA de Minecraft")}</a>{t(". Al aceptar, guardaremos tu decisión en este equipo para los siguientes servidores.")}</p>
      {consentError && <p className="form-error" role="alert">{translatedMessage(consentError)}</p>}
      <div className="modal-actions"><button data-modal-close disabled={consentBusy}>{t("Ahora no")}</button><button className="primary-button" disabled={consentBusy} onClick={async()=>{
        const target=downloadConsent;setConsentBusy(true);setConsentError(null);
        try {await api.acceptEula(target.id);if(selectedRef.current===target.id)setEula(true);setDownloadConsent(null);await runInstallation(target.id,target.kind);}
        catch(cause){setConsentError(errorText(cause));}
        finally {setConsentBusy(false);}
      }}>{consentBusy?t("Guardando…"):t("Aceptar y descargar")}</button></div>
    </Modal>}
    {closing && <Modal labelledBy="close-title" describedBy="close-description" busy={closingBusy} onClose={() => {setClosing(false);setCloseError(null)}}>
      <h2 id="close-title">{t("Cerrar TCHLauncher")}</h2><p id="close-description">{t("Mantén los servidores en segundo plano o detenlos antes de salir.")}</p>
      {closeError&&<p className="form-error close-operation-error" role="alert">{translatedMessage(closeError)}</p>}
      {closingBusy&&<p className="field-help" role="status">{t("Procesando la solicitud de cierre…")}</p>}
      <div className="close-options"><button disabled={closingBusy} onClick={()=>void closeApplication('hide')}>{t("Mantener en segundo plano")}</button><button disabled={closingBusy} onClick={()=>void closeApplication('stop')}>{t("Detener y salir")}</button><button data-modal-close autoFocus disabled={closingBusy}>{t("Cancelar")}</button></div>
    </Modal>}
  </div>
}

const eventLabels: Record<string,string> = localizedStatic({
  startup:'Servidor iniciado', shutdown:'Servidor deteniéndose', join:'Jugador conectado',
  world_load:'Cargando mundo',spawn_prepare:'Preparando zona inicial',
  leave:'Jugador desconectado', save:'Guardado notificado', save_started:'Guardado iniciado',save_dimension:'Dimensión guardada',save_failed:'Fallo de guardado',lag:'Retraso del servidor',
  warning:'Aviso', error:'Error', exception:'Excepción', plugin_error:'Error de plugin',
  mod_error:'Error de mod', crash:'Cierre inesperado', exit:'Proceso finalizado',
})
const eventTime = (timestamp: string | null) => {
  if (!timestamp) return '—'
  if (!timestamp.includes('T')) return timestamp
  const date = new Date(timestamp)
  return Number.isNaN(date.getTime()) ? '—' : date.toLocaleTimeString(getLanguage()==='en'?'en-US':'es-ES',{hour:'2-digit',minute:'2-digit'})
}

function Overview({server,metrics,observedPhase,players,playersStale,logs,backups,backupsStatus,eula,busy,onAccept,onInstall,onConsole,onSettings,onContent}: {
  observedPhase?:StartupEvent['phase']; server: Server; metrics: Metrics | null; players: PlayersSnapshot | null; playersStale:boolean; logs: LogLine[]; backups: Backup[]; backupsStatus:string|null;
  eula: boolean; busy: boolean; onAccept: () => void; onInstall: () => void; onConsole: () => void; onSettings:()=>void; onContent:()=>void;
}) {
  const recentEvents = logs.filter(item => item.event && eventLabels[item.event]).slice(-4).reverse()
  const warnings = logs.filter(item => item.level === 'warn').length
  const errors = logs.filter(item => item.level === 'error').length
  const save=saveState(logs,server.state)
  const saveLabel={unknown:t("Sin confirmación registrada"),saving:t("Guardado en curso"),confirmed:t("Finalización registrada"),failed:t("Fallo registrado"),unconfirmed:t("Finalización sin confirmar")}[save.phase]
  const description = server.state === 'running' ? t("Minecraft ha terminado de cargar el mundo.")
    : server.state === 'starting' ? t("Puedes seguir las fases del arranque o abrir la consola.")
    : server.state === 'stopping' ? t("Guardando el mundo y cerrando el proceso.")
    : server.state === 'installing' ? t("Descargando los archivos necesarios.")
    : server.state === 'crashed' || server.state === 'error' ? t("Revisa la consola antes de volver a iniciarlo.")
    : server.jarPath && server.javaPath ? t("Todo preparado para el próximo inicio.")
    : t("Instala Minecraft y Java para empezar.")
  return <div className="overview">
    <section className={`server-stage ${server.state}`} aria-label={t("Estado del servidor")}>
      <div className="stage-main"><div className="stage-copy" role="status"><h2><span className={`state-dot large ${server.state}`} aria-hidden="true"/>{stateText(server.state)}</h2><p>{translatedMessage(description)}</p></div>{['starting','stopping','crashed','error'].includes(server.state) && <button className="text-button" onClick={onConsole}>{t("Ver consola")} <Icon name="arrow" size={15}/></button>}</div>
      <StartupProgress state={server.state} metrics={metrics} observedPhase={observedPhase}/>
      <div className="stage-stats">
        <div className="stat-players"><PixelIcon kind="players"/><span>{t("Jugadores")}</span><strong>{players ? `${players.online.length} / ${players.maxPlayers}` : '—'}</strong><PixelMeter label="Plazas ocupadas" ratio={players&&players.maxPlayers>0?players.online.length/players.maxPlayers:null} valueText={players?t("{0} de {1} plazas ocupadas", players.online.length, players.maxPlayers):undefined}/><small>{playersStale?t("Última lectura · sin actualizar"):players ? `${Math.max(0,players.maxPlayers-players.online.length)} plazas libres` : t("Consultando jugadores")}</small></div>
        <div className="stat-uptime"><PixelIcon kind="clock"/><span>{t("Tiempo activo")}</span><strong><AnimatedValue value={metrics?.uptimeSeconds} format={fmtUptime}/></strong><small>{t("Desde el inicio de Java")}</small></div>
        <div className="stat-memory"><PixelIcon kind="memory"/><span>{t("RAM del proceso")}</span><strong><AnimatedValue value={metrics?.memoryBytes} format={fmtBytes}/></strong><PixelMeter label={t("Heap usado respecto al máximo configurado")} ratio={metrics?.heap&&server.memoryMaxMb>0?metrics.heap.usedBytes/(server.memoryMaxMb*1024**2):null}/><small>Heap: {metrics?.heap?fmtBytes(metrics.heap.usedBytes):'—'} / {(server.memoryMaxMb/1024).toFixed(1)}  {t("GB configurados")}</small></div>
        <div className="stat-cpu"><PixelIcon kind="cpu"/><span>CPU</span><strong><AnimatedValue value={metrics?.cpuPercent} format={value=>`${value.toFixed(1)} %`}/></strong><PixelMeter label={t("CPU del proceso")} ratio={metrics?.cpuPercent==null?null:metrics.cpuPercent/100}/><small>{t("Uso del proceso Java")}</small></div>
      </div>
    </section>
    {(!server.jarPath || !server.javaPath) && <div className="callout"><div><strong>{t("Instalación pendiente")}</strong><p>{server.jarPath ? t("Prepararemos Java sin reemplazar los archivos de arranque existentes.") : `Descargaremos Minecraft ${server.version} y Java.`}</p></div><button disabled={busy} onClick={onInstall}>{server.jarPath ? t("Preparar Java") : t("Instalar ahora")}</button></div>}
    {server.jarPath && !eula && <div className="callout"><div><strong>{t("Antes del primer inicio")}</strong><p>{t("Lee la")} <a href="https://www.minecraft.net/en-us/eula" target="_blank" rel="noreferrer">{t("EULA de Minecraft")}</a>  {t("antes de aceptarla. Se recordará en este equipo para tus próximos servidores.")}</p></div><button disabled={busy} onClick={onAccept}>{t("He leído y acepto")}</button></div>}
    <CrashPanel server={server} onConsole={onConsole} onSettings={onSettings} onContent={onContent}/>
    <NetworkPanel server={server}/>
    <div className="overview-storage"><section className="resource-section"><h2>{t("Almacenamiento")}</h2>
      <StorageMeter metrics={metrics} targetGb={server.storageTargetGb} onSettings={onSettings}/>
    </section><section><h2>{t("Estado del registro")}</h2><dl className="resource-facts"><div><dt>{t("Avisos")}</dt><dd>{warnings}</dd></div><div><dt>{t("Errores")}</dt><dd>{errors}</dd></div><div><dt>{t("Guardado del mundo")}</dt><dd className={`save-state ${save.phase}`} role="status">{translatedMessage(saveLabel)}</dd></div><div><dt>{t("Última finalización registrada")}</dt><dd>{eventTime(save.lastConfirmed?.timestamp??null)}</dd></div></dl><p className="field-help">{t("En las últimas")} {logs.length}  {t("líneas cargadas. La finalización se basa en el mensaje del servidor.")}</p>{save.phase==='failed'&&save.latest&&<div className="save-failure"><strong>{t("Fallo de guardado ·")} {eventTime(save.latest.timestamp)}</strong><p>{save.latest.message}</p><button onClick={onConsole}>{t("Revisar consola")}</button></div>}</section></div>
    <div className="overview-lower"><section className="recent-activity"><div className="section-heading"><h2>{t("Actividad reciente")}</h2><button className="text-button" onClick={onConsole}>{t("Ver consola")} <Icon name="arrow" size={15}/></button></div>{recentEvents.length ? <div className="event-list">{recentEvents.map((item,index) => <div className="event-row" key={`${item.timestamp ?? 'legacy'}-${index}`}><span className={`event-mark ${item.level}`} aria-hidden="true"/><div><strong>{eventLabels[item.event!]}</strong><span>{item.message}</span></div><time>{eventTime(item.timestamp)}</time></div>)}</div> : <p className="empty-inline">{t("Todavía no hay actividad.")}</p>}</section><section className="recent-backups"><div className="section-heading"><h2>{t("Copias recientes")}</h2></div>{backupsStatus&&<p className="field-help">{translatedMessage(backupsStatus)}</p>}{backups.length ? <div className="backup-mini-list">{backups.slice(0,3).map(backup => <div key={backup.id}><strong>{new Date(backup.createdAt).toLocaleString(getLanguage()==='en'?'en-US':'es-ES')}</strong><span>{fmtBytes(backup.sizeBytes)}</span></div>)}</div> : !backupsStatus?<p className="empty-inline">{t("Aún no hay copias guardadas.")}</p>:null}</section></div>
  </div>
}

function CreateView({onCancel,onCreate,busy,systemMemory}: {onCancel:()=>void;onCreate:(input:CreateInput,worldSource?:string)=>Promise<void>;busy:boolean;systemMemory:number}) {
  const steps = [t("Identidad"),t("Versión"),t("Mundo"),t("Capacidad"),t("Revisar")] as const
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
  const [input,setInput] = useState<CreateInput>({ name:'',version:'',platform:'vanilla',memoryMinMb:1024,memoryMaxMb:2048,maxPlayers:20,port:25565,storageTargetGb:20,imagePath:null,seed:'' })
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(input.memoryMaxMb,systemMemoryMb || 16384))
  const invalidCapacity = input.memoryMinMb < 512 || input.memoryMaxMb < input.memoryMinMb || input.memoryMaxMb > 65536 || input.maxPlayers < 1 || input.maxPlayers > 1000 || input.storageTargetGb < 1 || input.storageTargetGb > 2048 || input.port < 1 || input.port > 65535 || ![input.memoryMinMb,input.memoryMaxMb,input.maxPlayers,input.storageTargetGb,input.port].every(Number.isFinite)
  const invalidSeed=worldMode==='new'&&(new TextEncoder().encode(input.seed??'').length>1024||/[\u0000-\u001f\u007f]/.test(input.seed??''))
  const readyToCreate = !invalidSeed && !!input.name.trim() && !!input.version && !versionError && !versionLoading && !invalidCapacity && (worldMode === 'new' || !!worldSource)
  const canAdvance = step === 0 ? !!input.name.trim() : step === 1 ? !!input.version && !versionError && !versionLoading : step === 2 ? !invalidSeed && (worldMode === 'new' || !!worldSource) : step === 3 ? !invalidCapacity : readyToCreate
  const loadVersions = () => setCatalogRevision(value=>value+1)
  useEffect(() => {
    let active=true;setVersionLoading(true);setVersionError(null)
    if(!isTauri()){setVersionError(t("Abre la aplicación de escritorio para consultar versiones."));setVersionLoading(false);return}
    api.versions(input.platform,experimental&&['vanilla','fabric'].includes(input.platform)).then(items=>{
      if(!active)return
      setVersions(items)
      if(!items.length)setVersionError(t("No hay versiones compatibles disponibles."))
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
  const previousStep=useRef(step)
  useEffect(() => { if(previousStep.current!==step)wizardRef.current?.querySelector<HTMLElement>('h2')?.focus();previousStep.current=step },[step])
  const advance = () => {
    if (!canAdvance || busy) return
    if (step < steps.length - 1) {
      const next = step + 1
      moveStep(next)
      setFurthestStep(current => Math.max(current,next))
    } else onCreate({...input,seed:worldMode==='new'?input.seed:''},worldMode === 'existing' ? worldSource?.path : undefined)
  }
  return <div className="form-page">
    <div className="form-heading wizard-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/>  {t("Servidores")}</button><div className="wizard-title-row"><h1>{t("Nuevo servidor")}</h1><span className="wizard-counter">{t("Paso")} {step + 1}  {t("de")} {steps.length}</span></div></div>
    <nav className="wizard-steps" aria-label={t("Pasos de creación")}><MovingIndicator active={String(step)} axis="horizontal"/>{steps.map((label,index) => <button type="button" key={label} className={index === step ? 'active' : index < step ? 'completed' : ''} aria-label={`Paso ${index+1}: ${label}`} aria-current={index === step ? 'step' : undefined} disabled={busy || index > furthestStep} onClick={() => moveStep(index)}><span aria-hidden="true">{index < step ? '✓' : String(index + 1).padStart(2,'0')}</span><strong>{translatedMessage(label)}</strong></button>)}</nav>
    <div className="form-layout wizard-layout"><form className="create-form wizard-form" onSubmit={e => { e.preventDefault(); advance() }}>
      <div className="wizard-panel" key={step} ref={wizardRef}>
      {step === 0 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Nombre e imagen")}</h2><p>{t("Así aparecerá en tu biblioteca.")}</p></div>
      <div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt={t("Vista previa de la imagen")}/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage} disabled={!isTauri()}>{t("Elegir imagen")}</button><p>{t("PNG, JPG o WebP · hasta 2 MB")}</p>{imageError && <p className="form-error" role="alert">{translatedMessage(imageError)}</p>}</div><div className="form-row"><label htmlFor="new-name">{t("Nombre del servidor")}</label><input id="new-name" autoFocus required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})} placeholder={t("Mi servidor")}/><p className="field-help">{t("Solo lo verás tú en este launcher.")}</p></div></div>
      </>}
      {step === 1 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Versión y tipo")}</h2><p>{t("Elige la versión de Minecraft que utilizarán tus jugadores.")}</p></div>
      <div className="form-pair version-step-grid"><div className="form-row"><label htmlFor="new-version">{t("Versión de Minecraft")}</label>{versionError ? <div className="form-error" role="alert">{t("No se pudo cargar el catálogo:")} {translatedMessage(versionError)} <button type="button" onClick={loadVersions}>{t("Reintentar")}</button></div> : <select id="new-version" className="minecraft-version-list" size={6} aria-describedby="new-version-help" disabled={versionLoading} required value={input.version} onChange={e => setInput({...input,version:e.target.value})}><option value="">{versionLoading ? t("Cargando versiones…") : t("Selecciona una versión")}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}{v.kind==='snapshot'?' · Experimental':''}</option>)}</select>}<p id="new-version-help" className="field-help">{!versionLoading&&!versionError?t("{0} versiones en el catálogo · Desplázate para ver las anteriores.", versions.length):t("Versiones del catálogo oficial del tipo seleccionado.")}</p></div><fieldset className="version-platform-picker"><legend>{t("Tipo de servidor")}</legend><div>{(['vanilla','paper','fabric','neoforge'] as const).map(platform=><label key={platform}><input type="radio" name="new-platform" value={platform} checked={input.platform===platform} onChange={()=>{setVersionLoading(true);setVersionError(null);if(!['vanilla','fabric'].includes(platform))setExperimental(false);setInput(old=>({...old,platform}))}}/><span><strong>{platformName(platform)}</strong><small>{({vanilla:t("Minecraft original"),paper:t("Servidor con plugins"),fabric:t("Mods de Fabric"),neoforge:t("Mods de NeoForge")})[platform]}</small></span></label>)}</div></fieldset></div><p className="field-help type-help">{input.platform === 'neoforge' ? t("NeoForge permite mods de servidor. Incluye la edición histórica para Minecraft 1.20.1 y el catálogo moderno desde 1.20.2. No hay ediciones anteriores; usa mods compatibles con la versión elegida.") : input.platform === 'fabric' ? t("Fabric permite mods de servidor. Se elegirá automáticamente un loader estable compatible.") : input.platform === 'paper' ? t("Paper admite plugins y opciones extra de rendimiento.") : t("Vanilla ofrece la experiencia original de Minecraft.")}</p>
      {['vanilla','fabric'].includes(input.platform)&&<div className="experimental-versions"><label><input type="checkbox" checked={experimental} disabled={busy} onChange={event=>{setVersionLoading(true);setExperimental(event.target.checked)}}/><span>{t("Mostrar versiones experimentales")}</span></label>{experimental&&<p className="field-help">{t("Incluye snapshots y versiones de prueba del catálogo oficial. Pueden fallar y cambiar el formato de los mundos; crea una instancia separada para probarlas.")}</p>}</div>}
      </>}
      {step === 2 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Elige un mundo")}</h2><p>{t("Genera uno nuevo o importa un mundo de Minecraft Java.")}</p></div>
      <div className="world-choice"><label><input type="radio" name="world-mode" checked={worldMode === 'new'} onChange={() => setWorldMode('new')}/><span><strong>{t("Mundo nuevo")}</strong><small>{t("Minecraft lo generará en el primer inicio.")}</small></span></label><label><input type="radio" name="world-mode" checked={worldMode === 'existing'} onChange={() => setWorldMode('existing')}/><span><strong>{t("Usar un mundo existente")}</strong><small>{t("Se copiará a este servidor; el original se conserva.")}</small></span></label></div>
      {worldMode === 'new' && <div className="world-generation"><label htmlFor="new-world-seed"><strong>{t("Semilla del mundo")}</strong><span className="field-optional">{t("Opcional")}</span></label><input id="new-world-seed" type="text" value={input.seed??''} disabled={busy} aria-invalid={invalidSeed} aria-describedby="new-world-seed-help" placeholder={t("Aleatoria si la dejas vacía")} onChange={event=>setInput({...input,seed:event.target.value})}/><p id="new-world-seed-help" className={invalidSeed?'form-error':'field-help'}>{invalidSeed?t("Usa hasta 1024 bytes, sin saltos de línea ni caracteres de control."):t("Número o texto. Se usará al generar el mundo en su primer inicio.")}</p></div>}
      {worldMode === 'existing' && <div className="world-choice-source"><button type="button" onClick={chooseWorld}>{t("Elegir carpeta del mundo")}</button>{worldSource && <div><strong>{worldSource.suggestedName}</strong><span>{fmtBytes(worldSource.sizeBytes)} · {worldSource.path}</span></div>}{worldError && <p className="form-error" role="alert">{translatedMessage(worldError)}</p>}</div>}
      </>}
      {step === 3 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Capacidad del servidor")}</h2><p>{t("Reserva memoria para el sistema y para jugar.")}</p></div>
      <div className="capacity-grid"><SliderField id="new-memory-min" label="RAM inicial" value={input.memoryMinMb} min={512} max={Math.max(input.memoryMinMb,input.memoryMaxMb)} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMinMb:value,memoryMaxMb:value >= 512 && value <= 65536 ? Math.max(input.memoryMaxMb,value) : input.memoryMaxMb})}/><SliderField id="new-memory-max" label={t("RAM máxima")} value={input.memoryMaxMb} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMaxMb:value,memoryMinMb:value >= 512 && value <= 65536 ? Math.min(input.memoryMinMb,value) : input.memoryMinMb})}/><SliderField id="new-players" label={t("Jugadores máximos")} value={input.maxPlayers} min={1} max={Math.max(100,input.maxPlayers)} maxInput={1000} step={1} onChange={value => setInput({...input,maxPlayers:value})}/><SliderField id="new-storage" label={t("Objetivo de espacio")} value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/></div><p className="field-help">{systemMemory ? t("Tu equipo tiene {0} de RAM.", fmtBytes(systemMemory)) : t("Deja memoria libre para el sistema y para jugar.")}  {t("El objetivo de espacio sirve para avisarte; no impone una cuota al disco.")}</p>{systemMemoryMb > 0 && input.memoryMaxMb > systemMemoryMb * .75 && <p className="inline-note" role="status">{t("La RAM máxima supera el 75 % de la memoria física. Deja recursos para el sistema y para jugar.")}</p>}
      <details className="advanced"><summary>{t("Opciones de red")}</summary><div className="form-row"><label htmlFor="new-port">{t("Puerto del servidor")}</label><input id="new-port" type="number" min="1" max="65535" value={input.port} onChange={e => setInput({...input,port:Number(e.target.value)})}/></div></details>
      {invalidCapacity && <p className="form-error" role="alert">{t("Revisa RAM, jugadores, espacio y puerto antes de continuar.")}</p>}
      </>}
      {step === 4 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Revisar y crear")}</h2><p>{t("Comprueba los datos antes de instalar.")}</p></div>
      <div className="wizard-review"><div><span>{t("Identidad")}</span><strong>{input.name.trim() || t("Sin nombre")}</strong><button type="button" className="text-button" onClick={() => moveStep(0)}>{t("Editar")}</button></div><div><span>{t("Versión y tipo")}</span><strong>{input.version || t("Sin versión")}{versions.find(version=>version.id===input.version)?.kind==='snapshot'?' · Experimental':''} · {platformName(input.platform)}</strong><button type="button" className="text-button" onClick={() => moveStep(1)}>{t("Editar")}</button></div><div><span>{t("Mundo")}</span><strong>{worldMode === 'existing' ? t("Copia de {0}", worldSource?.suggestedName || t("mundo pendiente")) : `Mundo nuevo · ${input.seed?t("Semilla elegida"):t("Semilla aleatoria")}`}</strong><button type="button" className="text-button" onClick={() => moveStep(2)}>{t("Editar")}</button></div><div><span>{t("Recursos")}</span><strong>{fmtBytes(input.memoryMinMb * 1024 * 1024)}–{fmtBytes(input.memoryMaxMb * 1024 * 1024)} · {input.maxPlayers}  {t("jugadores ·")} {input.storageTargetGb} GB</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>{t("Editar")}</button></div><div><span>{t("Puerto")}</span><strong>{input.port}</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>{t("Editar")}</button></div></div>
      {worldMode==='new'&&input.seed&&<div className="wizard-seed-review"><span>{t("Semilla")}</span><code>{input.seed}</code></div>}
      <p className="wizard-final-note">{t("La instalación descargará Java y los archivos del servidor. En la primera descarga te pediremos aceptar la EULA de Minecraft. La decisión se recordará en este equipo.")}</p>
      </>}
      </div>
      <div className="form-actions wizard-actions"><div>{step > 0 && <button type="button" disabled={busy} onClick={() => moveStep(step - 1)}>{t("Anterior")}</button>}<button type="submit" className="primary-button" disabled={busy || !canAdvance || (step === steps.length - 1 && !readyToCreate)}>{busy ? t("Preparando…") : step === steps.length - 1 ? t("Crear e instalar") : t("Continuar")}<Icon name="arrow" size={16}/></button></div></div>
    </form><aside className="form-aside wizard-preview"><div className="wizard-preview-art" aria-hidden="true"><img src={worldIllustration} alt=""/></div><div className="aside-title">{t("Tu configuración")}</div><div className="aside-server"><div className="image-preview small">{imagePreview ? <img src={imagePreview} alt=""/> : <Icon name="server" size={22}/>}</div><div><strong>{input.name.trim() || t("Nuevo servidor")}</strong><span>{platformName(input.platform)} · {input.version || t("Versión pendiente")}</span></div></div><dl><div><dt>{t("Mundo")}</dt><dd>{worldMode === 'existing' ? worldSource?.suggestedName || t("Por elegir") : t("Nuevo")}</dd></div><div><dt>{t("Memoria")}</dt><dd><AnimatedValue value={input.memoryMaxMb * 1024 * 1024} format={fmtBytes}/></dd></div><div><dt>{t("Jugadores")}</dt><dd>{t("Hasta")} <AnimatedValue value={input.maxPlayers}/></dd></div><div><dt>{t("Objetivo de espacio")}</dt><dd><AnimatedValue value={input.storageTargetGb}/> GB</dd></div><div><dt>{t("Puerto")}</dt><dd>{input.port}</dd></div></dl><p>{t("El servidor se ejecutará en este ordenador. Java y los archivos se descargarán al crearlo.")}</p></aside></div>
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
    if (!isTauri()) { setFormError(t("Abre la aplicación de escritorio para importar una carpeta.")); return }
    api.versions().then(items => { setVersions(items) }).catch(e => setFormError(t("No se pudo consultar el catálogo: {0}", errorText(e))))
  },[])
  const choose = async () => {
    setPicking(true); setFormError(null)
    try {
      const path = await api.pickImportFolder()
      if (!path) return
      const info = await api.inspectImport(path)
      setInspection(info)
      const known=info.launchers.filter(launcher=>launcher.platform)
      const selected=known.length===1?known[0]:info.launchers.length===1?info.launchers[0]:null
      setInput(old=>({path:info.path,name:info.suggestedName,version:selected?.version??'',platform:selected?.platform??'vanilla',jarFile:selected?.filename??'',copy:true,storageTargetGb:old.storageTargetGb,imagePath:old.imagePath}))
    } catch(e) { setFormError(errorText(e)) }
    finally { setPicking(false) }
  }
  const chooseImage = async () => {
    setFormError(null)
    try { const image = await api.pickServerImage(); if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) } }
    catch(e) { setFormError(errorText(e)) }
  }
  return <div className="form-page import-page"><div className="form-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/>  {t("Servidores")}</button><h1>{t("Importar servidor")}</h1><p>{t("Trae un servidor existente a tu biblioteca sin perder sus archivos.")}</p></div><div className="import-source"><div className="import-source-icon"><Icon name="folder" size={22}/></div><div><strong>{inspection ? t("Carpeta seleccionada") : t("Selecciona la carpeta del servidor")}</strong><p>{inspection ? inspection.path : t("Debe contener server.properties y un archivo de arranque compatible")}</p></div><button onClick={choose} disabled={picking || busy}>{picking ? t("Inspeccionando…") : inspection ? t("Cambiar carpeta") : t("Elegir carpeta")}</button></div>{formError && <p className="form-error" role="alert">{translatedMessage(formError)}</p>}{inspection && <form className="create-form import-form" onSubmit={e => { e.preventDefault(); onImport(input) }}><div className="form-section-title"><span>01</span><div><h2>{t("Identidad")}</h2><p>{t("Así aparecerá en tu biblioteca.")}</p></div></div><div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt={t("Vista previa de la imagen")}/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage}>{t("Elegir imagen")}</button><p>{t("PNG, JPG o WebP · hasta 2 MB")}</p></div><div className="form-row"><label htmlFor="import-name">{t("Nombre del servidor")}</label><input id="import-name" required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})}/></div></div><div className="form-section-title"><span>02</span><div><h2>{t("Datos del servidor")}</h2><p>{t("Revisa los valores detectados.")}</p></div></div><div className="form-pair"><div className="form-row"><label htmlFor="import-version">{t("Versión de Minecraft")}</label><input id="import-version" list="import-versions" required value={input.version} onChange={e => setInput({...input,version:e.target.value})}/><datalist id="import-versions">{versions.map(v => <option key={v.id} value={v.id}/>)}</datalist></div><div className="form-row"><label htmlFor="import-platform">{t("Tipo")}</label><select id="import-platform" value={input.platform} onChange={e => setInput({...input,platform:e.target.value as ImportInput['platform']})}><option value="vanilla">Vanilla</option><option value="paper">Paper</option><option value="fabric">Fabric</option><option value="neoforge">NeoForge</option></select></div></div><p className="field-help">{t("Comprueba la versión del servidor original. La usaremos para elegir Java compatible.")}</p><div className="form-row"><label htmlFor="import-jar">{t("Archivo del servidor")}</label><select id="import-jar" required value={input.jarFile} onChange={e=>{const filename=e.target.value;const launcher=inspection.launchers.find(launcher=>launcher.filename===filename);setInput(old=>({...old,jarFile:filename,platform:launcher?.platform??'vanilla',version:launcher?.version??''}))}}><option value="">{t("Selecciona el archivo de arranque")}</option>{inspection.launchers.map(launcher=><option key={launcher.filename} value={launcher.filename}>{launcher.filename}{launcher.platform?` · ${platformName(launcher.platform)}`:''}{launcher.version?` · ${launcher.version}`:''}</option>)}</select>{inspection.launchers.find(launcher=>launcher.filename===input.jarFile)?.note&&<p className="field-help">{translatedMessage(inspection.launchers.find(launcher=>launcher.filename===input.jarFile)?.note)}</p>}</div><div className="import-storage"><SliderField id="import-storage" label={t("Objetivo de espacio")} value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/><p className="field-help">{t("Se usa para avisar cuando el servidor ocupa más espacio. No limita el disco.")}</p></div><fieldset className="import-mode"><legend>{t("Ubicación de los archivos")}</legend><label><input type="radio" checked={input.copy} onChange={() => setInput({...input,copy:true})}/>  {t("Copiar a la biblioteca de TCHLauncher")}</label><label><input type="radio" checked={!input.copy} onChange={() => setInput({...input,copy:false})}/>  {t("Administrar la carpeta actual")}</label><p className="field-help">{t("Si administras la carpeta actual, los cambios y las restauraciones afectarán a los archivos originales.")}</p></fieldset><div className="form-actions"><button type="button" onClick={onCancel}>{t("Cancelar")}</button><button className="primary-button" disabled={busy || !input.version || !input.jarFile || input.storageTargetGb < 1 || input.storageTargetGb > 2048}>{busy ? t("Importando…") : t("Importar servidor")}<Icon name="arrow" size={16}/></button></div></form>}</div>
}

function Settings({onConsole,onRuntimeChanged,onPrepareRuntime,server,props,setProps,propertiesLoaded,propertiesLoading,propertiesError,propertiesDirty,onReloadProperties,memory,setMemory,storageTarget,setStorageTarget,systemMemory,busy,progress,onSave,onPickImage,onClearImage,onDelete}: {onConsole:()=>void;onRuntimeChanged:()=>Promise<void>;onPrepareRuntime:(reinstall:boolean)=>void;server:Server;props:Record<string,string>;setProps:(p:Record<string,string>)=>void;propertiesLoaded:boolean;propertiesLoading:boolean;propertiesError:string|null;propertiesDirty:boolean;onReloadProperties:()=>void;memory:{min:number;max:number};setMemory:(v:{min:number;max:number})=>void;storageTarget:number;setStorageTarget:(value:number)=>void;systemMemory:number;busy:boolean;progress:Progress|null;onSave:()=>void;onPickImage:()=>void;onClearImage:()=>void;onDelete:()=>void}) {
  const [newKey,setNewKey] = useState('')
  const [newValue,setNewValue] = useState('')
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(memory.max,systemMemoryMb || 16384))
  const maxPlayers = Number(props['max-players'] || 20)
  const invalidCapacity = memory.min < 512 || memory.max < memory.min || memory.max > 65536 || storageTarget < 1 || storageTarget > 2048 || maxPlayers < 1 || maxPlayers > 1000 || ![memory.min,memory.max,maxPlayers,storageTarget].every(Number.isFinite)
  return <section className="settings-view"><div className="section-heading"><div><h2>{t("Configuración")}</h2><p>{t("Personaliza la apariencia y los ajustes del servidor.")}</p></div><button className="primary-button" disabled={busy || !propertiesLoaded || propertiesLoading || !!propertiesError || invalidCapacity || !['stopped','crashed','error'].includes(server.state)} onClick={onSave}>{t("Guardar cambios")} <Icon name="check" size={16}/></button></div>
    <h3>{t("Imagen del servidor")}</h3><div className="settings-image"><ServerArtwork server={server}/><div><strong>{server.name}</strong><p>{t("La misma imagen en el launcher y en la lista de Minecraft. Se adapta a 64 × 64. PNG, JPG o WebP, hasta 2 MB.")}</p><div className="settings-image-actions"><button disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onPickImage}>{server.imageData ? t("Cambiar imagen") : t("Elegir imagen")}</button>{server.imageData && <button className="text-button" disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onClearImage}>{t("Quitar imagen")}</button>}</div></div></div>
    {!['stopped','crashed','error'].includes(server.state) && <p className="inline-note">{t("Detén el servidor para editar su configuración.")}</p>}
    <RuntimePanel key={server.id} onChanged={onRuntimeChanged} server={server} busy={busy} onPrepare={onPrepareRuntime}/>
    <JvmPanel key={`jvm-${server.id}`} server={server} busy={busy}/>
    <MinecraftUpdatePanel key={`minecraft-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged} currentProgress={progress} onConsole={onConsole}/>
    <LoaderPanel key={`loader-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged}/>
    <TasksPanel key={`tasks-${server.id}`} id={server.id}/>
    <div className={`properties-load-state ${propertiesError?'error':''}`} role={propertiesError?'alert':'status'}><div><strong>{propertiesLoading?t("Consultando propiedades…"):propertiesError?t("Propiedades no disponibles"):!propertiesLoaded?t("Cargando propiedades…"):propertiesDirty?t("Cambios sin guardar"):t("Propiedades cargadas")}</strong><p>{propertiesError?`${propertiesError}${propertiesLoaded?' Se conservan los últimos valores leídos.':''}`:propertiesDirty?t("Las actualizaciones automáticas conservarán tus cambios."):t("Los cambios se aplican en el próximo inicio del servidor.")}</p></div><button disabled={busy||propertiesLoading||propertiesDirty} onClick={onReloadProperties}>{t("Recargar propiedades")}</button></div>
    {propertiesLoaded&&<fieldset className="settings-edit-fields" aria-label={t("Propiedades y capacidad")} disabled={busy || propertiesLoading || !!propertiesError || !['stopped','crashed','error'].includes(server.state)}><h3>{t("Capacidad")}</h3><div className="capacity-grid settings-capacity"><SliderField id="settings-memory-min" label="RAM inicial" value={memory.min} min={512} max={Math.max(memory.min,memory.max)} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,min:value,max:value >= 512 && value <= 65536 ? Math.max(memory.max,value) : memory.max})}/><SliderField id="settings-memory-max" label={t("RAM máxima")} value={memory.max} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,max:value,min:value >= 512 && value <= 65536 ? Math.min(memory.min,value) : memory.min})}/><SliderField id="settings-players" label={t("Jugadores máximos")} value={maxPlayers} min={1} max={Math.max(100,maxPlayers)} maxInput={1000} step={1} onChange={value => setProps({...props,'max-players':String(value)})}/><SliderField id="settings-storage" label={t("Objetivo de espacio")} value={storageTarget} min={1} max={Math.max(200,storageTarget)} maxInput={2048} step={1} unit="GB" onChange={setStorageTarget}/></div><p className="field-help">{systemMemory ? t("Memoria física: {0}.", fmtBytes(systemMemory)) : ''}  {t("El objetivo de espacio solo activa avisos; no impone una cuota al disco.")}</p>{systemMemoryMb > 0 && memory.max > systemMemoryMb * .75 && <p className="inline-note" role="status">{t("La RAM máxima supera el 75 % de la memoria física.")}</p>}{invalidCapacity && <p className="form-error" role="alert">{t("Revisa los límites de RAM, jugadores y espacio antes de guardar.")}</p>}
    <h3>{t("Propiedades del servidor")}</h3><p className="field-help">{t("Configura el juego antes del primer arranque. Los valores guardados se conservan; las opciones ausentes usan valores iniciales al guardar.")}</p><SettingsPropertyGroups values={props} onChange={setProps} version={server.version}/>
    <details className="advanced-properties"><summary>{t("Otras propiedades")}</summary><div className="settings-fields">{Object.entries(props).filter(([key]) => key !== 'max-players' && !knownSettings.has(key)).map(([key,value]) => <label className="setting-line" key={key}><span>{key}</span><input value={value} onChange={e => setProps({...props,[key]:e.target.value})}/></label>)}</div><div className="add-property"><input aria-label={t("Nombre de propiedad")} placeholder={t("Nombre de propiedad")} value={newKey} onChange={e => setNewKey(e.target.value)}/><input aria-label={t("Valor")} placeholder={t("Valor")} value={newValue} onChange={e => setNewValue(e.target.value)}/><button onClick={() => { if (newKey.trim()) { setProps({...props,[newKey.trim()]:newValue}); setNewKey(''); setNewValue('') } }}>{t("Añadir")}</button></div></details>
    </fieldset>}<div className="delete-server-section"><div><h3>{t("Eliminar servidor")}</h3><p>{t("Quita este servidor de la biblioteca. Si sus archivos están guardados aquí, también se eliminarán junto con sus copias de seguridad.")}</p></div><button className="danger-outline-button" disabled={busy || !['stopped','crashed','error'].includes(server.state)} onClick={onDelete}>{t("Eliminar servidor")}</button></div>
  </section>
}

export default App




