import {focusSearch} from './focusSearch'
import {fmtBytes,errorText} from './format'
import {translatedMessage,localizedStatic,t,formatDate,formatLogTime as eventTime} from './locale.ts'
import {acknowledgedAction} from './acknowledgedAction'
import {useLanguage} from './useLanguage'
import {createLogBatch} from './logBatch'
import {appendConsoleLines,mergeConsoleHistory,type ConsoleLine} from './consoleTimeline'
import {startVisibilityPolling} from './visibilityPolling'
import {deferredPanel} from './DeferredPanel'
import {Icon,ServerArtwork,platformName} from './ServerPresentation'
import {useServerRead} from './useServerRead'
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
import { api, type Backup, type CreateInput, type DeleteOutcome, type ImportInput, type LogLine, type Metrics, type PlayersSnapshot, type Progress, type Server, type ServerState, type WhitelistSelection, type WhitelistOutcome, type AccessList, type AccessRemoval } from './api'
import {StartupProgress} from './StartupProgress'
import {StorageMeter} from './StorageMeter'
import {acceptStartupEvent,type StartupEvent} from './startupEvents'

import { NetworkPanel } from './NetworkPanel'
import { Modal } from './Modal'
import {ForceStopDialog} from './ForceStopDialog'

import { InstallProgress } from './InstallProgress'
import { StartupPanel } from './StartupPanel'

import { ServerLibrary } from './ServerLibrary'

import { applyAppearance, browserPreferenceStorage, readAppearance, saveAppearance, type Appearance } from './appearance'
import launcherEmblem from './assets/launcher-logo.png'
import { motion, MovingIndicator, AnimatedValue, useListMotion, useNavigationMotion } from './motion'
import { ServerMenu, type ServerMenuTarget } from './ServerMenu'
import { ServerDeleteDialog } from './ServerDeleteDialog'
import { recordSample, type MetricSample } from './telemetry'
import {CommandPalette} from './CommandPalette'
import {launcherCommands,type LauncherCommand} from './commands'
import {BackgroundTasks} from './BackgroundTasks'
import {AppNotices} from './AppNotices'
import {ActionError,GeneralActionError,actionContext,type ActionContext,type ActionFailure} from './ActionError'
import {createNoticeObserver,taskSection,type AppNotice} from './noticeEvents'
import {dismissTask,isTaskActive,reconcileTasks} from './taskState'

const CreateView=deferredPanel<ComponentProps<typeof import('./CreateView').CreateView>>(t("Nuevo servidor"),()=>import('./CreateView').then(module=>({default:module.CreateView})))
const ImportView=deferredPanel<ComponentProps<typeof import('./ImportView').ImportView>>(t("Importar servidor"),()=>import('./ImportView').then(module=>({default:module.ImportView})))
const PackImport=deferredPanel<ComponentProps<typeof import('./PackImport').PackImport>>(t("Importar modpack"),()=>import('./PackImport').then(module=>({default:module.PackImport})))

const ServerSettings=deferredPanel<ComponentProps<typeof import('./ServerSettings').ServerSettings>>(t("Configuración"),()=>import('./ServerSettings').then(module=>({default:module.ServerSettings})))
const PlayersView=deferredPanel<ComponentProps<typeof import('./PlayersView').PlayersView>>(t("Jugadores"),()=>import('./PlayersView').then(module=>({default:module.PlayersView})))
const WorldView=deferredPanel<ComponentProps<typeof import('./WorldView').WorldView>>(t("Mundos"),()=>import('./WorldView').then(module=>({default:module.WorldView})))
const ContentManager=deferredPanel<ComponentProps<typeof import('./ContentManager').ContentManager>>(t("Mods y plugins"),()=>import('./ContentManager').then(module=>({default:module.ContentManager})))
const ResourceMonitor=deferredPanel<ComponentProps<typeof import('./ResourceMonitor').ResourceMonitor>>(t("Rendimiento"),()=>import('./ResourceMonitor').then(module=>({default:module.ResourceMonitor})))
const ApplicationSettings=deferredPanel<ComponentProps<typeof import('./ApplicationSettings').ApplicationSettings>>(t("Ajustes"),()=>import('./ApplicationSettings').then(module=>({default:module.ApplicationSettings})))

const BackupsView=deferredPanel<ComponentProps<typeof import('./BackupsView').BackupsView>>(t("Copias"),()=>import('./BackupsView').then(module=>({default:module.BackupsView})))

const ConsoleView=deferredPanel<ComponentProps<typeof import('./ConsoleView').ConsoleView>>(t("Consola"),()=>import('./ConsoleView').then(module=>({default:module.ConsoleView})))

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
const stateText = (state: string) => labels[state as ServerState] || state
const fmtUptime = (n?: number | null) => n == null ? '—' : `${Math.floor(n / 3600)} h ${Math.floor(n % 3600 / 60)} min`

function App() {
  const language=useLanguage()
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
  const contentRef=useRef<HTMLDivElement>(null)
  const [navigationDirection,setNavigationDirection]=useState(1)
  const navigate=(next:View)=>{setConsoleQuery('');setNavigationDirection(viewOrder.indexOf(next)>viewOrder.indexOf(view)?1:-1);setView(next)}
  const [creating, setCreating] = useState(false)
  const [importing, setImporting] = useState(false)
  const [packOpen,setPackOpen]=useState(false)
  const [packResume,setPackResume]=useState<string|null>(null)
  const [operationBusy,setBusy] = useState(false)
  const [installingIds,setInstallingIds] = useState<Set<string>>(new Set())
  const [error, setError] = useState<string | ActionFailure | null>(null)
  useEffect(()=>{
    if(!isTauri())return
    let current=true
    void api.setApplicationLanguage(language).catch(cause=>{
      if(current)setError(t("No se pudo actualizar el idioma de la bandeja: {0}",errorText(cause)))
    })
    return()=>{current=false}
  },[language])
  const [notice, setNotice] = useState<string | null>(null)
  const [progressByServer,setProgressByServer]=useState<Record<string,Progress>>({})
  const progressRef=useRef<Record<string,Progress>>({})
  const progressRevision=useRef(0)
  const [taskSyncError,setTaskSyncError]=useState<string|null>(null)
  const [appNotices,setAppNotices]=useState<AppNotice[]>([])
  const [noticeObserver]=useState(createNoticeObserver)
  const noticeContext=useRef({serverId:selectedId,view,visible:false})
  noticeContext.current={serverId:selectedId,view,visible:!libraryOpen&&!creating&&!importing&&!appearanceOpen&&!packOpen}
  const receiveNotice=useCallback((item:AppNotice|null)=>{
    if(item)setAppNotices(current=>[...current.filter(existing=>existing.id!==item.id),item].slice(-8))
  },[])
  const observeTask=useCallback((task:Progress,source:'event'|'snapshot'='event')=>{
    const context=noticeContext.current
    const background=document.hidden||!document.hasFocus()||!context.visible||context.serverId!==task.serverId||context.view!==taskSection(task.kind)
    receiveNotice(noticeObserver.task(task,background,source))
  },[noticeObserver,receiveNotice])
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
    const visibility=()=>batch.setPaused(document.hidden)
    visibility();document.addEventListener('visibilitychange',visibility)
    return()=>{document.removeEventListener('visibilitychange',visibility);batch.dispose();if(logBatch.current===batch)logBatch.current=null}
  },[])
  useEffect(()=>{logBatch.current?.reset()},[selectedId])
  const {copies:backups,loaded:backupsLoaded,loading:backupsLoading,error:backupsError,refresh:refreshBackups}=useBackupList(selectedId)
  const [restoreSync,setRestoreSync]=useState<Record<string,{loading:boolean;error:string|null}>>({})
  const restoreSyncRevision=useRef<Record<string,number>>({})
  const [restoreTarget,setRestoreTarget]=useState<{serverId:string;backup:Backup}|null>(null)
  const {values:props,savedValues:savedProps,loaded:propsLoaded,loading:propsLoading,error:propsError,dirty:propsDirty,edit:setProps,refresh:refreshProperties,acknowledge:acknowledgeProperties,discard:discardProperties}=useServerProperties(selectedId,servers.find(server=>server.id===selectedId)?.state)
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
        receiveNotice(noticeObserver.state(event.payload.serverId,event.payload.state))
        if(event.payload.state==='starting'){
          setStartupPhases(current=>{const next={...current};delete next[event.payload.serverId];return next})
          metricsGeneration.current[event.payload.serverId]=(metricsGeneration.current[event.payload.serverId]??0)+1
          setMetrics(current=>current?.serverId===event.payload.serverId?null:current)
          setMetricsHistory(current=>{const next={...current};delete next[event.payload.serverId];return next})
        }
        setServers(items => items.map(item => item.id === event.payload.serverId ? { ...item, state: event.payload.state } : item))
        if(event.payload.serverId===selectedRef.current&&['stopped','crashed','error'].includes(event.payload.state))void refreshPlayers(event.payload.serverId)
      }),
      listen<StartupEvent>('server-startup',event=>setStartupPhases(current=>acceptStartupEvent(current,event.payload))),
      listen<Progress>('install-progress', event => {observeTask(event.payload);progressRevision.current++;progressRef.current={...progressRef.current,[event.payload.serverId]:event.payload};setProgressByServer(progressRef.current)}),
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
  }, [refresh,noticeObserver,receiveNotice,observeTask])

  useEffect(()=>{
    if(!isTauri())return
    let active=true
    const sync=async()=>{
      const revision=progressRevision.current
      try{
        const snapshot=await api.installationProgress()
        if(!active)return
        if(revision===progressRevision.current){snapshot.forEach(task=>observeTask(task,'snapshot'));progressRef.current=reconcileTasks(progressRef.current,snapshot);setProgressByServer(progressRef.current)}
        setTaskSyncError(null)
      }catch(cause){if(active)setTaskSyncError(errorText(cause))}
    }
    const stop=startVisibilityPolling(sync,3000,{hiddenMs:15000,immediate:true})
    return()=>{active=false;stop()}
  },[observeTask])

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
      if(key==='f'&&focusSearch(mainRef.current))event.preventDefault()
    }
    window.addEventListener('keydown',shortcut)
    return () => window.removeEventListener('keydown',shortcut)
  }, [operationBusy,closeMenu])

  const [actionRefreshError,setActionRefreshError]=useState<string|null>(null)
  const act = async (fn: () => Promise<unknown>,context?:ActionContext) => {
    setError(null);setActionRefreshError(null);setBusy(true)
    try {const result=await acknowledgedAction(fn,refresh);setActionRefreshError(result.refreshError);return true}
    catch (e) { if(errorText(e)!=='Instalación cancelada')setError(context?{...context,detail:errorText(e)}:errorText(e)) }
    finally { setBusy(false) }
    return false
  }

  const renameServer=(id:string,name:string)=>act(async()=>{
    const saved=await api.renameServer(id,name)
    setServers(current=>current.map(server=>server.id===id?{...server,name:saved}:server))
  })

  const runServerAction=(server:Server,operation:'start'|'restart')=>act(()=>operation==='start'?api.start(server.id):api.restart(server.id),actionContext(server,operation))

  const importWhitelist=async(id:string,selection:WhitelistSelection):Promise<WhitelistOutcome|null>=>{
    let outcome:WhitelistOutcome|null=null
    const success=await act(async()=>{outcome=await api.importWhitelist(id,selection)})
    return success?outcome:null
  }

  const removeIdentity=async(id:string,list:AccessList,uuid:string):Promise<AccessRemoval|null>=>{
    let outcome:AccessRemoval|null=null
    const success=await act(async()=>{outcome=await api.removeAccessIdentity(id,list,uuid)})
    return success?outcome:null
  }

  const removeBannedIp=async(id:string,ip:string):Promise<AccessRemoval|null>=>{
    let outcome:AccessRemoval|null=null
    const success=await act(async()=>{outcome=await api.removeBannedIp(id,ip)})
    return success?outcome:null
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
  const discardSettings=()=>{
    if(!selected||selectedRef.current!==selected.id||busy||propsLoading||!propsLoaded)return
    discardProperties()
    capacityDirtyRef.current=false;setCapacityDirty(false)
    setMemory({min:selected.memoryMinMb,max:selected.memoryMaxMb})
    setStorageTarget(selected.storageTargetGb)
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
    const server=servers.find(item=>item.id===id)
    const reportFailure=(cause:unknown)=>setError(server&&kind!=='install'?{...actionContext(server,'runtime'),detail:errorText(cause)}:errorText(cause))
    try {if(!await api.eulaConsentRecorded()){setConsentError(null);setDownloadConsent({id,kind});return;}}
    catch(cause){reportFailure(cause);return;}
    setError(null);setInstallingIds(current=>new Set(current).add(id))
    try {if(kind==='install')await api.install(id);else await api.prepareRuntime(id,kind==='runtime_reinstall')}
    catch(cause){if(progressRef.current[id]?.stage!=='failed' && errorText(cause)!=='Instalación cancelada'){if(kind==='install')setError(errorText(cause));else reportFailure(cause)}}
    finally {setInstallingIds(current=>{const next=new Set(current);next.delete(id);return next});await refresh().then(()=>setActionRefreshError(null)).catch(cause=>setError(current=>current??errorText(cause)))}
  }
  const acceptCreatedServer=(server:Server)=>{
    setServers(current=>current.some(item=>item.id===server.id)?current.map(item=>item.id===server.id?server:item):[...current,server])
    openServer(server)
  }
  const create = async(input:CreateInput,worldSource?:Parameters<typeof api.importWorld>[1])=>{
    let id:string|null=null
    const ready=await act(async()=>{const server=await api.create(input);acceptCreatedServer(server);if(worldSource)await api.importWorld(server.id,worldSource,'world');id=server.id})
    if(ready && id)await runInstallation(id)
  }
  const importExisting = async(input:ImportInput)=>{
    mainRef.current?.scrollTo({top:0,left:0})
    let id:string|null=null
    const ready=await act(async()=>{const server=await api.importServer(input);acceptCreatedServer(server);id=server.id})
    if(ready && id)await runInstallation(id)
  }

  const removeServer = async (server: Server): Promise<DeleteOutcome> => {
    setError(null); setNotice(null); setActionRefreshError(null); setBusy(true)
    try {
      const {value:outcome,refreshError}=await acknowledgedAction(()=>api.deleteServer(server.id),async()=>{
        setServers(current=>current.filter(item=>item.id!==server.id))
        setMetricsHistory(current=>{const next={...current};delete next[server.id];return next})
        setStartupPhases(current=>{const next={...current};delete next[server.id];return next})
        setMetrics(current=>current?.serverId===server.id?null:current)
        const next={...progressRef.current};delete next[server.id];progressRevision.current++;progressRef.current=next;setProgressByServer(next)
        mainRef.current?.scrollTo({top:0,left:0})
        if(selectedRef.current===server.id){setSelectedId(null);setView('overview');setLogs([]);setMetricsError(null)}
        await refresh()
      })
      setActionRefreshError(refreshError)
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

  useListMotion(railRef,servers.map(server=>server.id).join(','))
  useNavigationMotion(contentRef,`${selectedId}:${view}:${appearanceOpen}:${creating}:${importing}:${libraryOpen}`,navigationDirection)
  const selectedName = selected?.name ?? t("Servidores")
  const openLibrary=()=>{setAppearanceOpen(false);setLibraryOpen(true);setCreating(false);setImporting(false);hideSidebarTooltip()}
  const openServer=(server:Server)=>{setAppearanceOpen(false);setLibraryOpen(false);setCreating(false);setImporting(false);setSelectedId(server.id);setView('overview')}
  const openTask=(serverId:string,section:View)=>{const server=servers.find(item=>item.id===serverId);if(!server)return;openServer(server);setView(section)}
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
    if(current.action==='folder'){
      void api.openApplicationFolder('server',server.id).catch(cause=>setError(errorText(cause)))
      return
    }
    openServer(server)
    switch(current.action){
      case 'overview':return
      case 'console':case 'content':setView(current.action);return
      case 'start':case 'restart':void runServerAction(server,current.action);return
      case 'stop':void act(()=>api.stop(server.id));return
      case 'backup':setView('backups');void createCopy(server.id);return
      default:{const unhandled:never=current.action;return unhandled}
    }
  }
  return <div className={`shell auto-rail ${sidebarCollapsed ? 'sidebar-collapsed' : 'rail-expanded'}`}>
    <AppNotices notices={appNotices} servers={servers} onDismiss={id=>setAppNotices(current=>current.filter(item=>item.id!==id))} onOpen={item=>{openTask(item.serverId,item.type==='crash'?'console':taskSection(item.task.kind));setAppNotices(current=>current.filter(notice=>notice.id!==item.id))}}/>
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
      <BackgroundTasks tasks={progressByServer} servers={servers} error={taskSyncError} onOpenChange={setRailTasksOpen} onOpen={task=>openTask(task.serverId,taskSection(task.kind))} onDismiss={task=>{progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>
      <button className="rail-startup" onClick={()=>setStartupOpen(true)} aria-label={t("Inicio automático")} title={t("Inicio automático")}><SectionIcon name="settings"/><span>{t("Inicio automático")}</span></button>
      <button className={`rail-settings ${appearanceOpen?'active':''}`} onClick={()=>{setAppearanceOpen(true);setCreating(false);setImporting(false);hideSidebarTooltip();closeMenu()}} aria-current={appearanceOpen?'page':undefined} aria-label={t("Ajustes")} title={t("Ajustes")}><SectionIcon name="appearance"/><span>{t("Ajustes")}</span></button>
      <span className="rail-version">v{appVersion}</span>
    </aside>
    <main className="main" ref={mainRef}>
      {actionRefreshError&&<div className="panel-read-state failed" role="alert"><div><strong>{t("Acción completada · datos sin actualizar")}</strong><p>{translatedMessage(actionRefreshError)}</p></div><button disabled={operationBusy} onClick={async()=>{try{await refresh();setActionRefreshError(null)}catch(cause){setActionRefreshError(errorText(cause))}}}>{t("Actualizar datos")}</button><button className="text-button" onClick={()=>setActionRefreshError(null)}>{t("Cerrar aviso")}</button></div>}
      {error && typeof error!=='string' ? <ActionError failure={error} available={servers.some(server=>server.id===error.serverId)} onClose={()=>setError(null)} onReview={section=>openTask(error.serverId,section)}/> : error && <GeneralActionError key={error} message={error} onClose={()=>setError(null)} onOpenLogs={api.openApplicationLogs}/>}
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
              <button disabled={busy} onClick={() => runServerAction(selected,'restart')}>{t("Reiniciar")}</button>
              <button className="primary-button" disabled={busy} onClick={() => act(() => api.stop(selected.id))}>{t("Detener")}</button>
            </> : selected.state === 'starting' ? <button disabled={busy} onClick={() => act(() => api.stop(selected.id))}>{t("Detener inicio")}</button> : selected.state === 'stopped' || selected.state === 'crashed' || selected.state === 'error' ? <button className="primary-button" disabled={busy || !eula} onClick={() => runServerAction(selected,'start')}>{t("Iniciar servidor")} <Icon name="arrow" size={15}/></button> : null : <button className="primary-button" disabled={busy} onClick={() => runInstallation(selected.id)}>{selected.jarPath ? t("Preparar Java") : t("Instalar")} <Icon name="arrow" size={15}/></button>}
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
          {view === 'players' && <PlayersView key={selected.id} server={selected} players={players} loading={playersLoading} readError={playersError} busy={busy} onRefresh={() => refreshPlayers(selected.id)} onAction={(action,target,reason) => act(() => api.managePlayer(selected.id,action,target,reason))} onToggleWhitelist={async () => { const success=await act(async () => { if (selected.state === 'running') await api.managePlayer(selected.id,players?.whitelistEnabled ? 'whitelist_off' : 'whitelist_on'); else { const values = { ...await api.properties(selected.id),'white-list':String(!players?.whitelistEnabled) }; await api.saveProperties(selected.id,values) } });if(success&&selected.state!=='running')await refreshProperties(selected.id).catch(()=>{});return success }} onImportWhitelist={selection => importWhitelist(selected.id,selection)} onRemoveIdentity={(list,uuid) => removeIdentity(selected.id,list,uuid)} onRemoveIp={ip=>removeBannedIp(selected.id,ip)}/>}
          {view === 'worlds' && <WorldView server={selected} onBackupsChanged={()=>refreshBackups(selected.id)}/>}
          {view === 'content' && selected.platform!=='vanilla' && <ContentManager onImportPack={()=>{setPackResume(null);setPackOpen(true)}} key={selected.id} server={selected} progress={progress} logs={logs} onConsole={query=>{setConsoleQuery(query);setView('console')}} onInstalled={async()=>{await refresh();await refreshBackups(selected.id)}} onDismissProgress={()=>{const next={...progressRef.current};delete next[selected.id];progressRef.current=next;setProgressByServer(next)}}/>}
          {view === 'console'  && <ConsoleView initialQuery={consoleQuery} serverId={selected.id} state={selected.state} logs={logs} historyLoading={logLoading} historyError={logError} onReload={() => refreshLog(selected.id)} busy={busy} onSend={command => act(() => api.send(selected.id,command))}/> }
          {view === 'backups' && <BackupsView key={selected.id} server={selected} copies={backups} loaded={backupsLoaded} loading={backupsLoading} error={backupsError} busy={busy} reasons={backupReason} onCreate={()=>void createCopy(selected.id)} onRefresh={()=>refreshBackups(selected.id)} onRestore={backup=>{setError(null);setRestoreTarget({serverId:selected.id,backup})}}/>}
          {view === 'settings' && <ServerSettings key={selected.id} onConsole={()=>setView('console')} onRuntimeChanged={refresh} onPrepareRuntime={reinstall=>runInstallation(selected.id,reinstall?'runtime_reinstall':'runtime')} server={selected} props={props} savedProps={savedProps} setProps={setProps} propertiesLoaded={propsLoaded} propertiesLoading={propsLoading} propertiesError={propsError} propertiesDirty={propsDirty||capacityDirty} onReloadProperties={()=>void refreshProperties(selected.id).catch(()=>{})} memory={memory} setMemory={value=>{capacityDirtyRef.current=true;setCapacityDirty(true);setMemory(value)}} storageTarget={storageTarget} setStorageTarget={value=>{capacityDirtyRef.current=true;setCapacityDirty(true);setStorageTarget(value)}} systemMemory={systemMemory} busy={busy||!!restoreSync[selected.id]} progress={progress} onRename={name=>renameServer(selected.id,name)} onDiscard={discardSettings} onSave={()=>void saveSettings(selected.id)} onPickImage={() => act(async () => { const image = await api.pickServerImage(); if (image) await api.setServerImage(selected.id,image.path) })} onClearImage={() => act(() => api.clearServerImage(selected.id))} onDelete={() => setDeleteTarget(selected)}/>}
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
      <dl className="restore-preview-details"><dt>{t("Fecha")}</dt><dd>{formatDate(restoreTarget.backup.createdAt)}</dd><dt>{t("Tamaño")}</dt><dd>{fmtBytes(restoreTarget.backup.sizeBytes)}</dd><dt>{t("Motivo")}</dt><dd>{restoreTarget.backup.reason?backupReason[restoreTarget.backup.reason]??t("No disponible"):t("No registrado")}</dd><dt>{t("Servidor guardado")}</dt><dd>{restoreTarget.backup.minecraftVersion?`${restoreTarget.backup.platform??t("Motor no registrado")} · Minecraft ${restoreTarget.backup.minecraftVersion}`:t("Versión no registrada")}</dd></dl>
      {progressByServer[restoreTarget.serverId]?.kind==='restore'&&<InstallProgress key={progressByServer[restoreTarget.serverId].taskId} progress={progressByServer[restoreTarget.serverId]} busy={operationBusy} allowRetry={false} onRetry={()=>{}} onDismiss={()=>{const task=progressByServer[restoreTarget.serverId];progressRef.current=dismissTask(progressRef.current,task.serverId,task.taskId);setProgressByServer(progressRef.current)}}/>}
      {error && <p className="form-error" role="alert">{translatedMessage(typeof error==='string'?error:error.detail)}</p>}
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
  identity:'Identidad del jugador', warning:'Aviso', error:'Error', exception:'Excepción', plugin_error:'Error de plugin',
  mod_error:'Error de mod', crash:'Cierre inesperado', exit:'Proceso finalizado',
})

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
    <div className="overview-lower"><section className="recent-activity"><div className="section-heading"><h2>{t("Actividad reciente")}</h2><button className="text-button" onClick={onConsole}>{t("Ver consola")} <Icon name="arrow" size={15}/></button></div>{recentEvents.length ? <div className="event-list">{recentEvents.map((item,index) => <div className="event-row" key={`${item.timestamp ?? 'legacy'}-${index}`}><span className={`event-mark ${item.level}`} aria-hidden="true"/><div><strong>{eventLabels[item.event!]}</strong><span>{item.message}</span></div><time>{eventTime(item.timestamp)}</time></div>)}</div> : <p className="empty-inline">{t("Todavía no hay actividad.")}</p>}</section><section className="recent-backups"><div className="section-heading"><h2>{t("Copias recientes")}</h2></div>{backupsStatus&&<p className="field-help">{translatedMessage(backupsStatus)}</p>}{backups.length ? <div className="backup-mini-list">{backups.slice(0,3).map(backup => <div key={backup.id}><strong>{formatDate(backup.createdAt)}</strong><span>{fmtBytes(backup.sizeBytes)}</span></div>)}</div> : !backupsStatus?<p className="empty-inline">{t("Aún no hay copias guardadas.")}</p>:null}</section></div>
  </div>
}


export default App
