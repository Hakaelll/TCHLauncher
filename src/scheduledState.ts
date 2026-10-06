import type {ScheduledTask} from './api'

export function scheduledResult(result:string|null){
  if(!result)return {label:'Sin ejecución anterior',tone:'neutral'}
  if(result.startsWith('Error:'))return {label:'Falló',tone:'error'}
  if(result.startsWith('Interrumpida'))return {label:'Interrumpida',tone:'warning'}
  if(result.startsWith('Copia cancelada'))return {label:'Cancelada',tone:'warning'}
  if(result==='Inicio solicitado'||result==='Reinicio solicitado')return {label:'Solicitud enviada',tone:'neutral'}
  return {label:'Último resultado',tone:'neutral'}
}
export function scheduledState(task:ScheduledTask){
  if(task.invalid)return 'Datos inválidos'
  if(task.running)return 'En ejecución'
  if(task.enabled)return 'Programada'
  if(task.lastAt!==null&&task.intervalMinutes===null)return 'Finalizada'
  return 'Pausada'
}
