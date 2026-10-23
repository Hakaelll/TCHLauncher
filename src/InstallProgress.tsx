import {translatedMessage,localizedStatic,t} from './locale.ts'
import { useState } from 'react'
import { api, type Progress } from './api'
import { AnimatedValue } from './motion'

const bytes = (value: number) => value >= 1024 ** 3 ? `${(value / 1024 ** 3).toFixed(1)} GB` : `${(value / 1024 ** 2).toFixed(1)} MB`
const stages: Record<Progress['stage'],string> = localizedStatic({preparing:'Preparando',connecting:'Conectando',downloading:'Descargando',verifying:'Verificando',extracting:'Extrayendo',copying:'Copiando',publishing:'Guardando',retrying:'Reintentando',completed:'Completado',cancelled:'Cancelado',failed:'Error'})

function failureAdvice(kind:Progress['kind']):string {
  switch(kind){
    case 'runtime':case 'runtime_reinstall':return t('Comprueba el runtime en Configuración antes de volver a prepararlo. Puedes verificar Java o elegir otro ejecutable compatible.')
    case 'install':return t('Revisa la conexión, el espacio disponible y el detalle antes de reintentar la instalación del servidor.')
    case 'content_install':return t('Revisa las versiones y dependencias del contenido antes de preparar otra instalación.')
    case 'pack_install':return t('Revisa el archivo .mrpack y las dependencias indicadas en el detalle antes de reintentar.')
    case 'minecraft_update':case 'loader_update':return t('Revisa la versión actual y las copias guardadas. Prepara una nueva revisión de compatibilidad antes de actualizar otra vez.')
    case 'backup':return t('Revisa las copias existentes, el espacio disponible y el acceso a la carpeta de copias antes de repetir.')
    case 'restore':return t('Revisa el estado del servidor y las copias disponibles antes de volver a restaurar o iniciar.')
    case 'server_import':return t('Revisa la carpeta original, el destino y el espacio disponible antes de importar otra vez.')
    case 'world_import':case 'world_duplicate':case 'world_export':return t('Revisa el origen, el destino y el espacio disponible antes de repetir la copia del mundo.')
    case 'world_activate':case 'world_delete':return t('Actualiza la lista de mundos y revisa las copias disponibles antes de repetir la acción.')
    case 'content_remove':case 'content_toggle':return t('Actualiza el inventario y comprueba el estado del archivo antes de repetir la acción.')
  }
  const unhandled:never=kind
  return unhandled
}

export function InstallProgress({progress,busy,onRetry,onDismiss,retryLabel=t("Reintentar"),allowRetry=true}: {progress:Progress;busy:boolean;onRetry:()=>void;onDismiss:()=>void;retryLabel?:string;allowRetry?:boolean}) {
  const [cancelling,setCancelling] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const terminal = ['completed','cancelled','failed'].includes(progress.stage)
  const cancellable=progress.stage!=='publishing'
  const measured=progress.stage==='downloading'||progress.stage==='copying'
  const received=Number.isFinite(progress.received)?Math.max(0,progress.received):0
  const total=progress.total!=null&&Number.isFinite(progress.total)&&progress.total>0?progress.total:null
  const percentage = measured&&total!=null ? Math.min(100,Math.floor(received / total * 100)) : null
  const cancel = async () => {
    setCancelling(true);setError(null)
    try { const accepted = await api.cancelInstall(progress.serverId,progress.taskId); if (!accepted) setCancelling(false) }
    catch (cause) {setError(String(cause));setCancelling(false)}
  }
  return <section className={`install-progress ${terminal ? 'terminal' : ''}`} data-stage={progress.stage} aria-label={t("Estado de la tarea")}>
    <div className="install-progress-heading"><div><strong>{translatedMessage(progress.label)}</strong><span className="install-stage" role="status">{terminal ? stages[progress.stage] : cancelling ? t("Cancelando…") : `${stages[progress.stage]}${progress.attempt > 1 ? ` ${t("· Intento {0} de 3",progress.attempt)}` : ''}`}</span></div>
      {!terminal ? cancellable&&<button disabled={cancelling} onClick={cancel}>{t("Cancelar")}</button> : <div className="install-progress-actions">{allowRetry&&progress.stage !== 'completed' && <button disabled={busy} onClick={onRetry}>{translatedMessage(retryLabel)}</button>}<button className="text-button" onClick={onDismiss}>{t("Cerrar")}</button></div>}
    </div>
    {!terminal && <><div className={`install-meter ${percentage==null?'unmeasured':'measured'}`}><progress aria-label={`${stages[progress.stage]}: ${translatedMessage(progress.label)}`} value={percentage!=null?Math.min(received,total!):undefined} max={percentage!=null?total!:undefined}/>{percentage!=null&&<strong className="install-percentage">{percentage}<small> %</small></strong>}</div><div className="install-transfer"><span>{received ? <><AnimatedValue key={progress.taskId} value={received} format={bytes}/>{total!=null ? ` / ${bytes(total)}` : ''}</> : measured?t("Esperando datos"):stages[progress.stage]}</span><span>{progress.bytesPerSecond != null && Number.isFinite(progress.bytesPerSecond)&&progress.bytesPerSecond>=0&&measured ? <><AnimatedValue key={progress.taskId} value={progress.bytesPerSecond} format={bytes}/>/s</> : '—'}</span></div></>}
    {progress.error && progress.stage === 'failed' && <div className="task-failure">
      <p className="form-error">{translatedMessage(progress.error)}</p>
      <p className="field-help">{failureAdvice(progress.kind)}</p>
      <details><summary>{t('Detalle técnico')}</summary><pre tabIndex={0}>{JSON.stringify({serverId:progress.serverId,taskId:progress.taskId,kind:progress.kind,attempt:progress.attempt,message:progress.error},null,2)}</pre></details>
    </div>}
    {progress.stage === 'cancelled' && <p>{progress.kind==='content_toggle'?t("Se canceló el cambio. El archivo y su configuración se conservan."):progress.kind==='content_remove'?t("Se canceló la retirada. El archivo y su configuración se conservan."):progress.kind==='server_import'?t("Se canceló la importación. La carpeta original se conserva."):progress.kind==='world_import'?t("Se canceló la importación. El mundo original se conserva."):(progress.kind==='world_duplicate'||progress.kind==='world_export')?t("Se canceló la copia. El mundo original se conserva."):(progress.kind==='world_activate'||progress.kind==='world_delete')?t("Se canceló el cambio. Los mundos se conservan junto a cualquier copia de seguridad ya completada."):progress.kind==='restore'?t("Se canceló la restauración. El servidor mantiene su estado anterior."):progress.kind==='backup'?t("Se canceló la copia. Los archivos del servidor se conservan."):t("Los archivos incompletos se han retirado. Puedes volver a intentarlo.")}</p>}
    {error && <p className="form-error" role="alert">{t("No se pudo solicitar la cancelación:")} {translatedMessage(error)}</p>}
  </section>
}
