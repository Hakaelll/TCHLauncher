import { invoke } from '@tauri-apps/api/core'

export type ServerState = 'stopped' | 'installing' | 'starting' | 'running' | 'stopping' | 'crashed' | 'error'
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
export type Version = { id: string; kind: string }
export type Progress = { serverId: string; taskId: string; kind: 'install' | 'runtime' | 'runtime_reinstall' | 'content_install'; label: string; stage: 'preparing' | 'connecting' | 'downloading' | 'verifying' | 'extracting' | 'retrying' | 'completed' | 'cancelled' | 'failed'; received: number; total: number | null; bytesPerSecond: number | null; attempt: number; error: string | null }
export type RuntimeInfo = {path:string|null;version:string|null;distribution:string|null;managed:boolean;status:'missing'|'invalid'|'ready';error:string|null;requiredMajor:number|null;compatibility:'unknown'|'compatible'|'incompatible'}
export type JvmSettings = {mode:'recommended'|'custom';arguments:string}
export type JvmPreview = {arguments:string[];installed:boolean}
export type LogLine = { serverId: string; level: string; line: string; message: string; timestamp: string | null; event: string | null }
export type Metrics = { serverId: string; sampledAt: number; pid: number | null; uptimeSeconds: number | null; memoryBytes: number | null; cpuPercent: number | null; diskReadBytes: number | null; diskWrittenBytes: number | null; sizeBytes: number }
export type Backup = { id: string; serverId: string; createdAt: string; sizeBytes: number; path: string }
export type PlayerIdentity = { name: string; uuid: string | null }
export type KnownPlayer = PlayerIdentity & { expiresOn: string | null }
export type Operator = PlayerIdentity & { level: number; bypassesPlayerLimit: boolean }
export type BannedPlayer = PlayerIdentity & { created: string | null; source: string | null; expires: string | null; reason: string | null }
export type BannedIp = { ip: string; created: string | null; source: string | null; expires: string | null; reason: string | null }
export type PlayersSnapshot = { online: string[]; known: KnownPlayer[]; whitelisted: PlayerIdentity[]; operators: Operator[]; bannedPlayers: BannedPlayer[]; bannedIps: BannedIp[]; whitelistEnabled: boolean; enforceWhitelist: boolean; maxPlayers: number }
export type WhitelistImport = { fileName: string; names: string[] }
export type CreateInput = { name: string; version: string; platform: 'vanilla' | 'paper' | 'fabric' | 'neoforge'; memoryMinMb: number; memoryMaxMb: number; maxPlayers: number; port: number; storageTargetGb: number; imagePath?: string | null }
export type ImportInspection = { path: string; suggestedName: string; jarFiles: string[]; detectedVersion: string | null }
export type ImportInput = { path: string; name: string; version: string; platform: 'vanilla' | 'paper' | 'fabric' | 'neoforge'; jarFile: string; copy: boolean; storageTargetGb: number; imagePath?: string | null }
export type ImageSelection = { path: string; data: string }
export type DeletePreview = { managedFiles: boolean; directory: string; backupCount: number }
export type DeleteOutcome = { retainedDirectory: string | null; cleanupWarnings: string[] }
export type World = { name: string; sizeBytes: number; active: boolean; iconData: string | null }
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
export type ContentSearch = { hits: {project_id: string; title: string; description: string; author: string; icon_url: string | null; categories: string[]; versions: string[]; environment?: string[]; server_side?: string}[]; offset: number; limit: number; total_hits: number }

export type InstalledContent = {filename: string; title: string; projectId: string | null; versionId: string | null; versionNumber: string | null; sha512: string; size: number; modified: boolean}
export type ContentReview = {token: string; plan: ContentPlan; changes: {title: string; filename: string; action: 'install' | 'update' | 'present'; previousVersion: string | null; version: string}[]; warnings: string[]}
export type PackEntry = {path: string; size: number; source: 'download' | 'overrides' | 'server-overrides'; server: 'required' | 'optional'; selected: boolean; replacesCommon: boolean}
export type PackInspection = {path: string; archiveSha512: string; name: string; versionId: string; summary: string | null; minecraft: string; loader: string; loaderVersion: string | null; entries: PackEntry[]; excludedClientFiles: number; selectedBytes: number; blockers: string[]; warnings: string[]}
export const api = {
  pickModpack: () => invoke<PackInspection|null>('pick_modpack'),
  inspectModpack: (path:string) => invoke<PackInspection>('inspect_modpack',{path}),
  removeContent: (id:string,filename:string,sha512:string) => invoke<void>('remove_content',{id,filename,sha512}),
  planContentUpdates: (id:string) => invoke<ContentReview>('plan_content_updates',{id}),
  contentIcon: (url:string) => invoke<string>('content_icon',{url}),
  listContent: (id:string) => invoke<InstalledContent[]>('list_content',{id}),
  installContent: (id:string,token:string) => invoke<void>('install_content',{id,token}),
  contentVersions: (id:string,projectId:string) => invoke<ContentVersion[]>('content_versions',{id,projectId}),
  searchContent: (id:string,query:string,offset=0) => invoke<ContentSearch>('search_content',{id,query,offset}),
  planContent: (id:string,projectId:string,versionId:string|null=null) => invoke<ContentReview>('plan_content',{id,projectId,versionId}),
  listServers: () => invoke<Server[]>('list_servers'),
  versions: (platform:Server['platform']='vanilla') => invoke<Version[]>('get_versions',{platform}),
  create: (input: CreateInput) => invoke<Server>('create_server', { input }),
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
  restart: (id: string) => invoke<void>('restart_server', { id }),
  send: (id: string, command: string) => invoke<void>('send_command', { id, command }),
  players: (id: string) => invoke<PlayersSnapshot>('get_players', { id }),
  managePlayer: (id: string, action: string, target?: string, reason?: string) => invoke<void>('manage_player', { id, action, target: target || null, reason: reason || null }),
  pickWhitelistFile: () => invoke<WhitelistImport | null>('pick_whitelist_file'),
  importWhitelist: (id: string, names: string[]) => invoke<number>('import_whitelist', { id, names }),
  log: (id: string) => invoke<LogLine[]>('read_log', { id }),
  exportLog: (id: string) => invoke<string | null>('export_log', { id }),
  properties: (id: string) => invoke<Record<string, string>>('get_properties', { id }),
  saveProperties: (id: string, values: Record<string, string>) => invoke<void>('save_properties', { id, values }),
  saveMemory: (id: string, min: number, max: number) => invoke<void>('save_memory', { id, min, max }),
  saveStorageTarget: (id: string, gb: number) => invoke<void>('save_storage_target', { id, gb }),
  createBackup: (id: string) => invoke<Backup>('create_backup', { id }),
  backups: (id: string) => invoke<Backup[]>('list_backups', { id }),
  restoreBackup: (id: string, backupId: string) => invoke<void>('restore_backup', { id, backupId }),
  worlds: (id: string) => invoke<World[]>('list_worlds', { id }),
  pickWorldFolder: () => invoke<WorldInspection | null>('pick_world_folder'),
  importWorld: (id: string, path: string, name: string) => invoke<World>('import_world', { id, path, name }),
  duplicateWorld: (id: string, name: string, newName: string) => invoke<World>('duplicate_world', { id, name, newName }),
  exportWorld: (id: string, name: string) => invoke<string | null>('export_world', { id, name }),
  setActiveWorld: (id: string, name: string) => invoke<Backup | null>('set_active_world', { id, name }),
  deleteWorld: (id: string, name: string) => invoke<Backup>('delete_world', { id, name }),
  openWorldFolder: (id: string, name: string) => invoke<void>('open_world_folder', { id, name }),
  metrics: (id: string) => invoke<Metrics>('get_metrics', { id }),
  systemMemory: () => invoke<number>('system_memory'),
  networkStatus: (id: string) => invoke<NetworkSnapshot>('get_network_status', { id }),
  publicIp: () => invoke<string>('get_public_ip'),
  hideWindow: () => invoke<void>('hide_window'),
  stopAllAndExit: () => invoke<void>('stop_all_and_exit'),
}
