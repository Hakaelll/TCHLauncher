import { invoke } from '@tauri-apps/api/core'

export type ServerState = 'stopped' | 'installing' | 'starting' | 'running' | 'stopping' | 'crashed' | 'error'
export type Server = {
  id: string
  name: string
  version: string
  platform: 'vanilla' | 'paper'
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
export type Progress = { serverId: string; label: string; received: number; total: number | null }
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
export type CreateInput = { name: string; version: string; platform: 'vanilla' | 'paper'; memoryMinMb: number; memoryMaxMb: number; maxPlayers: number; port: number; storageTargetGb: number; imagePath?: string | null }
export type ImportInspection = { path: string; suggestedName: string; jarFiles: string[]; detectedVersion: string | null }
export type ImportInput = { path: string; name: string; version: string; platform: 'vanilla' | 'paper'; jarFile: string; copy: boolean; storageTargetGb: number; imagePath?: string | null }
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

export const api = {
  listServers: () => invoke<Server[]>('list_servers'),
  versions: () => invoke<Version[]>('get_versions'),
  create: (input: CreateInput) => invoke<Server>('create_server', { input }),
  pickImportFolder: () => invoke<string | null>('pick_import_folder'),
  pickServerImage: () => invoke<ImageSelection | null>('pick_server_image'),
  setServerImage: (id: string, path: string) => invoke<Server>('set_server_image', { id, path }),
  clearServerImage: (id: string) => invoke<Server>('clear_server_image', { id }),
  previewDeleteServer: (id: string) => invoke<DeletePreview>('preview_delete_server', { id }),
  deleteServer: (id: string) => invoke<DeleteOutcome>('delete_server', { id }),
  inspectImport: (path: string) => invoke<ImportInspection>('inspect_import', { path }),
  importServer: (input: ImportInput) => invoke<Server>('import_server', { input }),
  install: (id: string) => invoke<void>('install_server', { id }),
  eulaAccepted: (id: string) => invoke<boolean>('eula_accepted', { id }),
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
