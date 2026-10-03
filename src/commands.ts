import type {Server} from './api'

export type LauncherCommand={id:string;label:string;detail:string;action:'library'|'create'|'import'|'appearance'|'overview'|'console'|'content'|'start'|'stop'|'backup';serverId?:string;disabledReason?:string}
export function launcherCommands(servers:Server[],busy:boolean,installing:ReadonlySet<string>):LauncherCommand[] {
  const commands:LauncherCommand[]=[
    {id:'library',label:'Mis servidores',detail:'Abrir la biblioteca',action:'library'},
    {id:'create',label:'Nuevo servidor',detail:'Asistente de creación',action:'create',disabledReason:busy?'Espera a que termine la operación actual':undefined},
    {id:'import',label:'Importar servidor',detail:'Elegir una carpeta existente',action:'import',disabledReason:busy?'Espera a que termine la operación actual':undefined},
    {id:'appearance',label:'Ajustes del launcher',detail:'Tema, tipografía y diagnóstico',action:'appearance'},
  ]
  for(const server of servers) {
    const context=`${server.name} · ${server.platform} · ${server.version}`
    const blocked=busy||installing.has(server.id)?'Hay una operación en curso':undefined
    const stopped=['stopped','crashed','error'].includes(server.state)
    const entry=(action:LauncherCommand['action'],label:string,disabledReason?:string):LauncherCommand=>({id:`${server.id}:${action}`,label:`${label} · ${server.name}`,detail:context,action,serverId:server.id,disabledReason})
    commands.push(entry('overview','Abrir servidor'),entry('console','Ver consola'))
    if(server.platform!=='vanilla')commands.push(entry('content',server.platform==='paper'?'Buscar plugins':'Buscar mods'))
    if(server.state==='running'||server.state==='starting')commands.push(entry('stop',server.state==='starting'?'Detener inicio':'Detener servidor',blocked))
    else commands.push(entry('start','Iniciar servidor',blocked||(!stopped?'El servidor está cambiando de estado':!server.jarPath||!server.javaPath?'Prepara el servidor y Java desde Resumen':undefined)))
    commands.push(entry('backup','Crear copia',blocked||(!stopped?'Detén el servidor para crear una copia consistente':undefined)))
  }
  return commands
}
const normalized=(value:string)=>value.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase().trim()
export function filterCommands(commands:LauncherCommand[],query:string):LauncherCommand[] {
  const terms=normalized(query).split(/\s+/).filter(Boolean)
  return commands.filter(command=>terms.every(term=>normalized(`${command.label} ${command.detail}`).includes(term)))
}
