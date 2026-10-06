import type {JvmSettings,Server} from './api'
export function jvmPreviewKey(server:Server,settings:JvmSettings){
  return JSON.stringify([server.id,server.platform,server.version,server.directory,server.jarPath,server.javaPath,server.memoryMinMb,server.memoryMaxMb,settings.mode,settings.mode==='custom'?settings.arguments:''])
}
