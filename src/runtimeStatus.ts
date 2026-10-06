import type {RuntimeInfo} from './api'
export function runtimeStatus(info:RuntimeInfo|null,loading:boolean,error:string|null){
  if(loading)return {kind:'checking',label:'Comprobando',description:'Comprobando el runtime…'}
  if(error)return {kind:'unavailable',label:'Sin comprobar',description:info?'Última lectura conservada. Comprueba Java de nuevo.':'No se pudo consultar el runtime.'}
  if(!info)return {kind:'unavailable',label:'No disponible',description:'Consulta Java para ver su estado.'}
  if(info.compatibility==='incompatible')return {kind:'incompatible',label:'Incompatible',description:`Java ${info.version??'sin versión'} no cumple el requisito de este servidor.`}
  if(info.status==='invalid')return {kind:'invalid',label:'No válido',description:'El ejecutable de Java no se pudo verificar.'}
  if(info.status==='missing')return {kind:'missing',label:'Sin instalar',description:'Prepara Java para este servidor.'}
  return {kind:'ready',label:info.compatibility==='compatible'?'Listo':'Versión comprobada',description:`Java ${info.version??'sin versión'} · ${info.managed?'Gestionado por TCHLauncher':'Runtime externo'}`}
}
