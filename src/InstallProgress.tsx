import {translatedMessage} from './locale.ts'
import {localizedStatic,t} from './locale.ts'
import { useState } from 'react'
import { api, type Progress } from './api'
import { AnimatedValue } from './motion'

const bytes = (value: number) => value >= 1024 ** 3 ? `${(value / 1024 ** 3).toFixed(1)} GB` : `${(value / 1024 ** 2).toFixed(1)} MB`
const stages: Record<Progress['stage'],string> = localizedStatic({preparing:'Preparando',connecting:'Conectando',downloading:'Descargando',verifying:'Verificando',extracting:'Extrayendo',copying:'Copiando',publishing:'Guardando',retrying:'Reintentando',completed:'Completado',cancelled:'Cancelado',failed:'Error'})

export function InstallProgress({progress,busy,onRetry,onDismiss,retryLabel=t("Reintentar"),allowRetry=true}: {progress:Progress;busy:boolean;onRetry:()=>void;onDismiss:()=>void;retryLabel?:string;allowRetry?:boolean}) {
  const [cancelling,setCancelling] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const terminal = ['completed','cancelled','failed'].includes(progress.stage)
  const cancellable=progress.stage!=='publishing'
  const measured=progress.stage==='downloading'||progress.stage==='copying'
  const percentage = progress.total ? Math.min(100,Math.floor(progress.received / progress.total * 100)) : null
  const cancel = async () => {
    setCancelling(true);setError(null)
    try { const accepted = await api.cancelInstall(progress.serverId,progress.taskId); if (!accepted) setCancelling(false) }
    catch (cause) {setError(String(cause));setCancelling(false)}
  }
  return <section className={`install-progress ${terminal ? 'terminal' : ''}`} aria-label={t("Estado de la tarea")}>
    <div className="install-progress-heading"><div><strong>{translatedMessage(progress.label)}</strong><span role="status">{terminal ? stages[progress.stage] : cancelling ? t("Cancelando…") : `${stages[progress.stage]}${progress.attempt > 1 ? ` · Intento ${progress.attempt} de 3` : ''}`}</span></div>
      {!terminal ? cancellable&&<button disabled={cancelling} onClick={cancel}>{t("Cancelar")}</button> : <div className="install-progress-actions">{allowRetry&&progress.stage !== 'completed' && <button disabled={busy} onClick={onRetry}>{translatedMessage(retryLabel)}</button>}<button className="text-button" onClick={onDismiss}>{t("Cerrar")}</button></div>}
    </div>
    {!terminal && <><progress aria-label={`${stages[progress.stage]}: ${progress.label}`} value={measured && progress.total ? progress.received : undefined} max={progress.total ?? undefined}/><div className="install-transfer"><span>{progress.received ? <><AnimatedValue key={progress.label} value={progress.received} format={bytes}/>{progress.total ? ` / ${bytes(progress.total)}` : ''}</> : measured?t("Esperando datos"):stages[progress.stage]}{percentage != null && measured ? ` · ${percentage} %` : ''}</span><span>{progress.bytesPerSecond != null && measured ? <><AnimatedValue key={progress.label} value={progress.bytesPerSecond} format={bytes}/>/s</> : '—'}</span></div></>}
    {progress.error && progress.stage === 'failed' && <p className="form-error">{translatedMessage(progress.error)}</p>}
    {progress.stage === 'cancelled' && <p>{progress.kind==='content_toggle'?t("Se canceló el cambio. El archivo y su configuración se conservan."):progress.kind==='content_remove'?t("Se canceló la retirada. El archivo y su configuración se conservan."):progress.kind==='server_import'?t("Se canceló la importación. La carpeta original se conserva."):progress.kind==='world_import'?t("Se canceló la importación. El mundo original se conserva."):(progress.kind==='world_duplicate'||progress.kind==='world_export')?t("Se canceló la copia. El mundo original se conserva."):(progress.kind==='world_activate'||progress.kind==='world_delete')?t("Se canceló el cambio. Los mundos se conservan junto a cualquier copia de seguridad ya completada."):progress.kind==='restore'?t("Se canceló la restauración. El servidor mantiene su estado anterior."):progress.kind==='backup'?t("Se canceló la copia. Los archivos del servidor se conservan."):t("Los archivos incompletos se han retirado. Puedes volver a intentarlo.")}</p>}
    {error && <p className="form-error" role="alert">{t("No se pudo solicitar la cancelación:")} {translatedMessage(error)}</p>}
  </section>
}
