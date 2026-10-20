import { invoke } from '@tauri-apps/api/core'

export type ServerState = 'stopped' | 'installing' | 'starting' | 'running' | 'stopping' | 'crashed' | 'error'
export type RouterStatus = {
  router:{name:string;ip:string;externalIp:string|null;publicAddress:boolean|null;checkedAt:number}|null
  mapping:{phase:'pending'|'active'|'closing'|'attention';port:number;localAddress:string;externalAddress:string|null;expiresAt:number|null;verifiedAt:number|null;lastError:string|null}|null
  leaseSeconds:number
  remoteReachability:'not_checked'
  discoveryError:string|null
}
export type Server = {
  id: string
  name: string
  version: string
  platform: 'vanilla' | 'paper' | 'fabric' | 'neoforge'
  pinned: boolean
  directory: string
  jarPath: string | null
  javaPath: string | null
  memoryMinMb: number
  memoryMaxMb: number
  storageTargetGb: number
  createdAt: string
  state: ServerState
  imageData: string | null
}
export type ApplicationInfo = {version:string;os:string;architecture:string;locations:{key:'library'|'servers'|'runtimes'|'backups';path:string;exists:boolean}[];eulaRecorded:boolean}
export type Version = { id: string; kind: string }
export type Progress = { serverId: string; taskId: string; kind: 'install' | 'runtime' | 'runtime_reinstall' | 'content_install' | 'pack_install' | 'loader_update' | 'minecraft_update' | 'backup' | 'restore' | 'world_import' | 'world_duplicate' | 'world_export' | 'world_activate' | 'world_delete' | 'server_import' | 'content_remove' | 'content_toggle'; label: string; stage: 'preparing' | 'connecting' | 'downloading' | 'verifying' | 'extracting' | 'copying' | 'publishing' | 'retrying' | 'completed' | 'cancelled' | 'failed'; received: number; total: number | null; bytesPerSecond: number | null; attempt: number; error: string | null }
export type RuntimeInfo = {path:string|null;version:string|null;distribution:string|null;managed:boolean;status:'missing'|'invalid'|'ready';error:string|null;requiredMajor:number|null;compatibility:'unknown'|'compatible'|'incompatible'}
export type JvmSettings = {mode:'recommended'|'custom';arguments:string}
export type JvmPreview = {arguments:string[];installed:boolean}
export type LogLine = { serverId: string; level: string; line: string; message: string; timestamp: string | null; event: string | null }
export type StorageUsage = {sampledAt:number;serverBytes:number;worldsBytes:number;contentBytes:number;logsBytes:number;otherBytes:number;backupBytes:number;skippedLinks:number;warningPercent:number;serverVolume:{mount:string;availableBytes:number;totalBytes:number}|null;backupVolume:{mount:string;availableBytes:number;totalBytes:number}|null}
export type StartupPhase='launching'|'loading_world'|'preparing_spawn'|'ready'
export type ServerResponse={address:string;sampledAt:number;latencyMs:number;version:string|null;online:number|null;maximum:number|null}
export type Metrics = { serverId: string; sampledAt: number; pid: number | null; uptimeSeconds: number | null; startupDurationMs: number | null; startupPhase: StartupPhase|null; memoryBytes: number | null; heap:{usedBytes:number;committedBytes:number;collections:number;sampledAt:number}|null; heapError:string|null; cpuPercent: number | null; diskReadBytes: number | null; diskWrittenBytes: number | null; sizeBytes: number|null; sizeError:string|null; sizeSampledAt:number|null; sizeSkippedLinks:number|null; storageWarningPercent:number; tps:{reading:{oneMinute:number;fiveMinutes:number;fifteenMinutes:number;sampledAt:number}|null;pending:boolean} }
export type Backup = { id: string; serverId: string; createdAt: string; sizeBytes: number; path: string; retentionWarning?: string|null;reason?:string|null;minecraftVersion?:string|null;platform?:string|null;metadataError?:string|null }
export type BackupPolicy = {enabled:boolean;maxCount:number|null;maxBytes:number|null}
export type BackupRetentionPlan = {fingerprint:string;remove:Backup[];beforeBytes:number;afterBytes:number;warnings:string[]}
export type TaskAction = 'backup'|'backup_restart'|'start'|'restart'|'stop'
export type CrashPolicy = {enabled:boolean;delaySeconds:number;maxAttempts:number}
export type CrashStatus = {policy:CrashPolicy;attempts:number;nextAt:number|null;lastResult:string|null}
export type OsAutostart = {supported:boolean;enabled:boolean;currentPath:boolean;note:string|null}
export type StartupServer = {id:string;name:string;enabled:boolean;lastResult:string|null}
export type ScheduledInput = {action:TaskAction;nextAt:number;intervalMinutes:number|null}
export type ScheduledTask = Omit<ScheduledInput,'action'> & {action:TaskAction|'invalid';invalid:boolean;id:string;serverId:string;enabled:boolean;running:boolean;lastAt:number|null;lastResult:string|null}
export type PlayerIdentity = { name: string; uuid: string | null }
export type KnownPlayer = PlayerIdentity & { expiresOn: string | null }
export type Operator = PlayerIdentity & { level: number; bypassesPlayerLimit: boolean }
export type BannedPlayer = PlayerIdentity & { created: string | null; source: string | null; expires: string | null; reason: string | null }
export type BannedIp = { ip: string; created: string | null; source: string | null; expires: string | null; reason: string | null }
export type AuthenticationCheck = {file:string;key:string;enabled:boolean|null;problem:string|null}
export type PlayersSnapshot = { online: string[]; known: KnownPlayer[]; whitelisted: PlayerIdentity[]; operators: Operator[]; bannedPlayers: BannedPlayer[]; bannedIps: BannedIp[]; whitelistEnabled: boolean; enforceWhitelist: boolean; maxPlayers: number; onlineMode: boolean; authenticationChecks:AuthenticationCheck[] }
export type WhitelistImport = { fileName: string; names: string[] }
export type CreateInput = { seed?:string; name: string; version: string; platform: 'vanilla' | 'paper' | 'fabric' | 'neoforge'; memoryMinMb: number; memoryMaxMb: number; maxPlayers: number; port: number; storageTargetGb: number; imagePath?: string | null }
export type ImportInspection = { path: string; suggestedName: string; jarFiles: string[]; detectedVersion: string | null; launchers: {filename:string;platform:Server['platform']|null;version:string|null;note:string|null}[] }
export type ImportInput = { path: string; name: string; version: string; platform: 'vanilla' | 'paper' | 'fabric' | 'neoforge'; jarFile: string; copy: boolean; storageTargetGb: number; imagePath?: string | null }
export type ImageSelection = { path: string; data: string }
export type DeletePreview = { managedFiles: boolean; directory: string; backupCount: number }
export type DeleteOutcome = { retainedDirectory: string | null; cleanupWarnings: string[] }
export type World = { name: string; sizeBytes: number; active: boolean; iconData: string | null; dimensions: string[] }
export type WorldInspection = { path: string; suggestedName: string; sizeBytes: number }
export type NetworkSnapshot = {
  localIp: string | null
  configuredIp: string | null
  port: number
  lanAddress: string | null
  localAddress: string
  portState: 'listening' | 'starting' | 'not_responding' | 'available' | 'occupied' | 'unknown'
}

export type ContentDependency = { project_id: string | null; version_id: string | null; file_name: string | null; dependency_type: 'required' | 'optional' | 'incompatible' | 'embedded' }
export type ContentFile = { filename: string; url: string; hashes: Record<string,string>; size: number; primary: boolean }
export type ContentVersion = { id: string; project_id: string; name: string; version_number: string; version_type: string; date_published: string; game_versions: string[]; loaders: string[]; environment: string | null; changelog: string | null; dependencies: ContentDependency[]; files: ContentFile[] }
export type ContentProject = { id: string; title: string; description: string; icon_url: string | null; environment: string[]; server_side: string | null }
export type ContentPlan = { minecraftVersion: string; loader: string; entries: {project: ContentProject; version: ContentVersion; file: ContentFile; required: boolean}[]; optional: ContentDependency[]; incompatible: ContentDependency[] }
export type ContentSearch = { hits: {project_id: string; title: string; description: string; author: string; icon_url: string | null; categories: string[]; versions: string[]; downloads?:number; environment?: string[]; server_side?: string}[]; offset: number; limit: number; total_hits: number }

export type InstalledContent = {filename: string; title: string; projectId: string | null; versionId: string | null; versionNumber: string | null; sha512: string; size: number; modified: boolean}
export type ContentKind = 'mod'|'plugin'|'modpack'|'resourcepack'|'shader'
export type ContentSearchOptions = {category:string;sort:string;allVersions:boolean;projectKind?:ContentKind}
export type ContentAvailability = {versionId:string|null;versionNumber:string|null;channel:string|null;environment:string|null}
export type ContentCheck = {filename:string;descriptor:{id:string|null;ids?:string[];apiVersion?:string|null;dependencies?:{id:string;kind:'depends'|'recommends'|'suggests'|'conflicts'|'breaks';alternatives:string[]}[];name:string|null;version:string|null;environment:string|null;minecraft:string|null;note:string|null};issues:string[];listedInLog:boolean|null}
export type ContentAudit = {files:InstalledContent[];disabled:InstalledContent[];checks:ContentCheck[];logModifiedAt:number|null;logNote:string}
export type ContentReview = {token: string; plan: ContentPlan; changes: {title: string; filename: string; action: 'install' | 'update' | 'present'; previousVersion: string | null; version: string}[]; warnings: string[]}
export type PackEntry = {path: string; size: number; source: 'download' | 'overrides' | 'server-overrides'; server: 'required' | 'optional'; selected: boolean; replacesCommon: boolean; protected: boolean}
export type PackInspection = {path: string; archiveSha512: string; name: string; versionId: string; summary: string | null; minecraft: string; loader: string; loaderVersion: string | null; entries: PackEntry[]; excludedClientFiles: number; selectedBytes: number; blockers: string[]; warnings: string[]}
export type LoaderReview = {current:string;available:string[];fingerprint:string}
export type MinecraftReview = {currentMinecraft:string;targetMinecraft:string;platform:string;currentLoader:string|null;targetLoader:string|null;javaMajor:number;javaAction:'retain_configured'|'prepare_managed';content:{title:string;filename:string;status:'compatible'|'incompatible'|'unknown'|'update';reason:string}[];changes:ContentReview['changes'];migration:ContentPlan|null;blockers:string[];warnings:string[];fingerprint:string}
export type NatPmpRecord={serverId:string;localIp:string;gateway:string;internalPort:number;externalPort:number|null;externalIp:string|null;phase:'pending'|'active'|'closing'|'attention';expiresAt:number;confirmedAt:number|null;renewAt:number;retryAt:number;lastError:string|null}
export type NatPmpDetection={gateway:string;externalIp:string;publicAddress:boolean}
export const api = {
  serverResponse:(id:string)=>invoke<ServerResponse>('check_server_response',{id}),
  natpmpStatus:(id:string)=>invoke<NatPmpRecord|null>('get_natpmp_status',{id}),
  detectNatpmp:(id:string,gateway:string|null)=>invoke<NatPmpDetection>('detect_natpmp',{id,gateway}),
  openNatpmp:(id:string,gateway:string,expectedIp:string,expectedPort:number)=>invoke<NatPmpRecord>('open_natpmp',{id,gateway,expectedIp,expectedPort,confirmed:true}),
  closeNatpmp:(id:string)=>invoke<void>('close_natpmp',{id}),
  requestTps:(id:string)=>invoke<void>('request_tps',{id}),
  storageUsage:(id:string)=>invoke<StorageUsage>('get_storage_usage',{id}),
  saveStorageWarning:(id:string,percent:number)=>invoke<void>('save_storage_warning',{id,percent}),
  applicationInfo:()=>invoke<ApplicationInfo>('get_application_info'),
  openApplicationFolder:(key:ApplicationInfo['locations'][number]['key']|'server',id:string|null=null)=>invoke<void>('open_application_folder',{key,id}),
  installationProgress: () => invoke<Progress[]>('get_installation_progress'),
  minecraftReview: (id:string,target:string) => invoke<MinecraftReview>('review_minecraft_update',{id,target}),
  updateMinecraft: (id:string,target:string,fingerprint:string,confirmedWarnings:boolean) => invoke<string|null>('update_minecraft',{id,target,fingerprint,confirmedWarnings}),
  loaderReview: (id:string) => invoke<LoaderReview>('review_loader_update',{id}),
  updateLoader: (id:string,target:string,fingerprint:string) => invoke<string|null>('update_loader',{id,target,fingerprint}),
  osAutostart: () => invoke<OsAutostart>('get_os_autostart'),
  setOsAutostart: (enabled:boolean) => invoke<OsAutostart>('set_os_autostart',{enabled}),
  startupServers: () => invoke<StartupServer[]>('get_startup_servers'),
  setStartupServer: (id:string,enabled:boolean) => invoke<void>('set_startup_server',{id,enabled}),
  crashRestart: (id:string) => invoke<CrashStatus>('get_crash_restart',{id}),
  saveCrashRestart: (id:string,policy:CrashPolicy) => invoke<void>('save_crash_restart',{id,policy}),
  scheduledTasks: (id:string) => invoke<ScheduledTask[]>('list_scheduled_tasks',{id}),
  addScheduledTask: (id:string,input:ScheduledInput) => invoke<ScheduledTask>('add_scheduled_task',{id,input}),
  setScheduledTaskEnabled: (id:string,taskId:string,enabled:boolean) => invoke<void>('set_scheduled_task_enabled',{id,taskId,enabled}),
  removeScheduledTask: (id:string,taskId:string) => invoke<void>('remove_scheduled_task',{id,taskId}),
  stopBackup: (id:string) => invoke<StopBackupStatus>('get_stop_backup',{id}),
  setStopBackup: (id:string,enabled:boolean) => invoke<void>('set_stop_backup',{id,enabled}),
  backupPolicy: (id:string) => invoke<BackupPolicy>('get_backup_policy',{id}),
  saveBackupPolicy: (id:string,policy:BackupPolicy) => invoke<void>('save_backup_policy',{id,policy}),
  previewBackupRetention: (id:string) => invoke<BackupRetentionPlan>('preview_backup_retention',{id}),
  applyBackupRetention: (id:string,fingerprint:string) => invoke<BackupRetentionPlan>('apply_backup_retention',{id,fingerprint}),
  installModpack: (id:string,pack:PackInspection,selected:string[]) => invoke<void>('install_modpack',{id,path:pack.path,archiveSha512:pack.archiveSha512,selected}),
  pickModpack: () => invoke<PackInspection|null>('pick_modpack'),
  inspectModpack: (path:string) => invoke<PackInspection>('inspect_modpack',{path}),
  removeContent: (id:string,filename:string,sha512:string) => invoke<void>('remove_content',{id,filename,sha512}),
  planContentUpdates: (id:string) => invoke<ContentReview>('plan_content_updates',{id}),
  installedContentIcon: (id:string,filename:string) => invoke<string|null>('installed_content_icon',{id,filename}),
  contentIcon: (url:string) => invoke<string>('content_icon',{url}),
  setContentEnabled: (id:string,filename:string,sha512:string,enabled:boolean) => invoke<void>('set_content_enabled',{id,filename,sha512,enabled}),
  listContent: (id:string) => invoke<InstalledContent[]>('list_content',{id}),
  installContent: (id:string,token:string) => invoke<void>('install_content',{id,token}),
  contentVersions: (id:string,projectId:string) => invoke<ContentVersion[]>('content_versions',{id,projectId}),
  searchContent: (id:string,query:string,offset=0,options?:ContentSearchOptions) => invoke<ContentSearch>('search_content',{id,query,offset,options}),
  openContentProject: (projectId:string) => invoke<void>('open_content_project',{projectId}),
  contentAvailability: (id:string,projectId:string) => invoke<ContentAvailability>('content_availability',{id,projectId}),
  auditContent: (id:string) => invoke<ContentAudit>('audit_content',{id}),
  planContent: (id:string,projectId:string,versionId:string|null=null) => invoke<ContentReview>('plan_content',{id,projectId,versionId}),
  listServers: () => invoke<Server[]>('list_servers'),
  versions: (platform:Server['platform']='vanilla',experimental=false) => invoke<Version[]>('get_versions',{platform,experimental}),
  create: (input: CreateInput) => invoke<Server>('create_server', { input,seed:input.seed??null }),
  setPinned: (id:string,pinned:boolean) => invoke<void>('set_server_pinned',{id,pinned}),
  pickImportFolder: () => invoke<string | null>('pick_import_folder'),
  pickServerImage: () => invoke<ImageSelection | null>('pick_server_image'),
  setServerImage: (id: string, path: string) => invoke<Server>('set_server_image', { id, path }),
  clearServerImage: (id: string) => invoke<Server>('clear_server_image', { id }),
  previewDeleteServer: (id: string) => invoke<DeletePreview>('preview_delete_server', { id }),
  deleteServer: (id: string) => invoke<DeleteOutcome>('delete_server', { id }),
  inspectImport: (path: string) => invoke<ImportInspection>('inspect_import', { path }),
  importServer: (input: ImportInput) => invoke<Server>('import_server', { input }),
  install: (id: string) => invoke<void>('install_server', { id }),
  cancelInstall: (id: string, taskId: string) => invoke<boolean>('cancel_install', { id, taskId }),
  pickCustomRuntime: (id:string) => invoke<RuntimeInfo|null>('pick_custom_runtime',{id}),
  setCustomRuntime: (id:string,path:string) => invoke<void>('set_custom_runtime',{id,path}),
  runtimeInfo: (id:string) => invoke<RuntimeInfo>('get_runtime_info',{id}),
  jvmSettings: (id:string) => invoke<JvmSettings>('get_jvm_settings',{id}),
  previewJvm: (id:string,settings:JvmSettings) => invoke<JvmPreview>('preview_jvm',{id,settings}),
  saveJvm: (id:string,settings:JvmSettings) => invoke<void>('save_jvm_settings',{id,settings}),
  playerHead: (uuid:string) => invoke<string|null>('get_player_head',{uuid}),
  serverCover: (id:string) => invoke<string|null>('get_server_cover',{id}),
  prepareRuntime: (id:string,reinstall:boolean) => invoke<void>('prepare_runtime',{id,reinstall}),
  eulaAccepted: (id: string) => invoke<boolean>('eula_accepted', { id }),
  eulaConsentRecorded: () => invoke<boolean>('eula_consent_recorded'),
  acceptEula: (id: string) => invoke<void>('accept_eula', { id }),
  start: (id: string) => invoke<void>('start_server', { id }),
  stop: (id: string) => invoke<void>('stop_server', { id }),
  previewForceStop: (id:string) => invoke<{sessionId:string;pid:number}>('preview_force_stop',{id}),
  forceStop: (id:string,sessionId:string) => invoke<void>('force_stop_server',{id,sessionId}),
  restart: (id: string) => invoke<void>('restart_server', { id }),
  send: (id: string, command: string) => invoke<void>('send_command', { id, command }),
  players: (id: string) => invoke<PlayersSnapshot>('get_players', { id }),
  saveOperator: (id:string,edit:{uuid:string;level:number;bypassesPlayerLimit:boolean;expectedLevel:number;expectedBypassesPlayerLimit:boolean}) => invoke<string>('save_operator',{id,edit}),
  managePlayer: (id: string, action: string, target?: string, reason?: string) => invoke<void>('manage_player', { id, action, target: target || null, reason: reason || null }),
  pickWhitelistFile: () => invoke<WhitelistImport | null>('pick_whitelist_file'),
  importWhitelist: (id: string, names: string[]) => invoke<number>('import_whitelist', { id, names }),
  log: (id: string) => invoke<LogLine[]>('read_log', { id }),
  exportMemoryReport: (id: string) => invoke<string | null>('export_memory_report', { id }),
  exportLog: (id: string) => invoke<string | null>('export_log', { id }),
  properties: (id: string) => invoke<Record<string, string>>('get_properties', { id }),
  saveProperties: (id: string, values: Record<string, string>) => invoke<void>('save_properties', { id, values }),
  saveMemory: (id: string, min: number, max: number) => invoke<void>('save_memory', { id, min, max }),
  saveServerSettings: (id:string,values:Record<string,string>,min:number,max:number,gb:number)=>invoke<void>('save_server_settings',{id,values,min,max,gb}),
  saveStorageTarget: (id: string, gb: number) => invoke<void>('save_storage_target', { id, gb }),
  createBackup: (id: string) => invoke<Backup>('create_backup', { id }),
  backups: (id: string) => invoke<Backup[]>('list_backups', { id }),
  restoreBackup: (id: string, backupId: string) => invoke<void>('restore_backup', { id, backupId }),
  worlds: (id: string) => invoke<World[]>('list_worlds', { id }),
  pickWorldFolder: () => invoke<WorldInspection | null>('pick_world_folder'),
  importWorld: (id: string, path: string, name: string) => invoke<World>('import_world', { id, path, name }),
  duplicateWorld: (id: string, name: string, newName: string) => invoke<World>('duplicate_world', { id, name, newName }),
  exportWorld: (id: string, name: string, includeContent=false, includeConfig=true) => invoke<string | null>('export_world', { id, name, includeContent, includeConfig }),
  setActiveWorld: (id: string, name: string) => invoke<Backup | null>('set_active_world', { id, name }),
  deleteWorld: (id: string, name: string) => invoke<Backup>('delete_world', { id, name }),
  openWorldFolder: (id: string, name: string) => invoke<void>('open_world_folder', { id, name }),
  metrics: (id: string) => invoke<Metrics>('get_metrics', { id }),
  systemMemory: () => invoke<number>('system_memory'),
  networkStatus: (id: string) => invoke<NetworkSnapshot>('get_network_status', { id }),
  publicIp: () => invoke<string>('get_public_ip'),
  openApplicationLogs: () => invoke<void>('open_application_logs'),
  routerStatus: (id:string) => invoke<RouterStatus>('get_router_status',{id}),
  detectRouter: (id:string) => invoke<RouterStatus>('detect_router',{id}),
  openRouterPort: (id:string,expectedIp:string,expectedPort:number) => invoke<RouterStatus>('open_router_port',{id,confirmed:true,expectedIp,expectedPort}),
  closeRouterPort: (id:string) => invoke<RouterStatus>('close_router_port',{id}),
  lastCrashReport: (id:string) => invoke<CrashReport|null>('get_last_crash_report',{id}),
  crashReport: (id:string) => invoke<CrashReport|null>('get_crash_report',{id}),
  hideWindow: () => invoke<void>('hide_window'),
  stopAllAndExit: () => invoke<void>('stop_all_and_exit'),
}


export type CrashReport={serverId:string;sessionId:string;endedAt:number;exitCode:number|null;stopRequested:boolean;forced:boolean;outputComplete:boolean;diagnoses:{kind:string;title:string;explanation:string;recommendation:string;action:string;evidence:string[]}[];recentFiles:{name:string;folder:string;modifiedAt:number}[]}

export type StopBackupStatus={enabled:boolean;phase:'idle'|'copying'|'completed'|'cancelled'|'error';lastAt:number|null;lastResult:string|null}
