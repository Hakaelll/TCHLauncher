import {t} from './locale.ts'
import type {ScheduledTask} from './api'

export function scheduledResult(result:string|null){
  if(!result)return {label:t("Sin ejecución anterior"),tone:'neutral'}
  if(result.startsWith('Error:'))return {label:t("Falló"),tone:'error'}
  if(result.startsWith('Interrumpida'))return {label:t("Interrumpida"),tone:'warning'}
  if(result.startsWith('Copia cancelada'))return {label:t("Cancelada"),tone:'warning'}
  if(result==='Inicio solicitado'||result==='Reinicio solicitado')return {label:t("Solicitud enviada"),tone:'neutral'}
  return {label:t("Último resultado"),tone:'neutral'}
}
export function scheduledState(task:ScheduledTask){
  if(task.invalid)return t("Datos inválidos")
  if(task.running)return t("En ejecución")
  if(task.enabled)return t("Programada")
  if(task.lastAt!==null&&task.intervalMinutes===null)return t("Finalizada")
  return t("Pausada")
}
