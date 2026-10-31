import {localizedStatic,t} from './locale.ts'
export const playerActions = localizedStatic({
  whitelist_add: {label:t("Añadir a lista blanca"),effect:t("Autoriza a entrar cuando la lista blanca esté activa."),ip:false},
  op: {label:'Dar permisos OP',effect:t("Concede permisos de operador, incluidos comandos de administración."),ip:false},
  ban: {label:'Bloquear jugador',effect:t("Impide volver a entrar hasta que retires el bloqueo."),ip:false},
  kick: {label:'Expulsar jugador',effect:t("Cierra su conexión actual. Podrá volver a entrar."),ip:false},
  ban_ip: {label:'Bloquear IP',effect:t("Bloquea a quienes se conecten desde esta dirección, incluso si comparten red."),ip:true},
} as const)
export type PlayerAction=keyof typeof playerActions
export function playerActionErrors(action:PlayerAction,target:string,reason:string):{target?:string;reason?:string}{
  const errors:{target?:string;reason?:string}={}
  const name=target.trim()
  if(!name)errors.target=playerActions[action].ip?t("Introduce una dirección IPv4 o IPv6."):t("Introduce el nombre de Minecraft.")
  else if(!playerActions[action].ip&&!/^[A-Za-z0-9_]{3,16}$/.test(name))errors.target=t('Usa entre 3 y 16 letras, números o guiones bajos.')
  if(['ban','ban_ip','kick'].includes(action)){
    const value=reason.trim()
    if(Array.from(value).length>120||/[\u0000-\u001f\u007f-\u009f]/.test(value))errors.reason=t('Usa hasta 120 caracteres, sin saltos de línea ni caracteres de control.')
  }
  return errors
}
