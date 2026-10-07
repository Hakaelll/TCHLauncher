import {t} from './locale.ts'
import type {RuntimeInfo} from './api'
export function runtimeStatus(info:RuntimeInfo|null,loading:boolean,error:string|null){
  if(loading)return {kind:'checking',label:'Comprobando',description:t("Comprobando el runtime…")}
  if(error)return {kind:'unavailable',label:t("Sin comprobar"),description:info?t("Última lectura conservada. Comprueba Java de nuevo."):t("No se pudo consultar el runtime.")}
  if(!info)return {kind:'unavailable',label:t("No disponible"),description:t("Consulta Java para ver su estado.")}
  if(info.compatibility==='incompatible')return {kind:'incompatible',label:'Incompatible',description:t("Java {0} no cumple el requisito de este servidor.", info.version??t("sin versión"))}
  if(info.status==='invalid')return {kind:'invalid',label:t("No válido"),description:t("El ejecutable de Java no se pudo verificar.")}
  if(info.status==='missing')return {kind:'missing',label:'Sin instalar',description:t("Prepara Java para este servidor.")}
  return {kind:'ready',label:info.compatibility==='compatible'?t("Listo"):t("Versión comprobada"),description:`Java ${info.version??t("sin versión")} · ${info.managed?'Gestionado por TCHLauncher':'Runtime externo'}`}
}
