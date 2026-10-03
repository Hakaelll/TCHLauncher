import { useState } from 'react'
import { api, type Progress } from './api'
import { AnimatedValue } from './motion'

const bytes = (value: number) => value >= 1024 ** 3 ? `${(value / 1024 ** 3).toFixed(1)} GB` : `${(value / 1024 ** 2).toFixed(1)} MB`
const stages: Record<Progress['stage'],string> = {preparing:'Preparando',connecting:'Conectando',downloading:'Descargando',verifying:'Verificando',extracting:'Extrayendo',copying:'Copiando',publishing:'Guardando',retrying:'Reintentando',completed:'Completado',cancelled:'Cancelado',failed:'Error'}

export function InstallProgress({progress,busy,onRetry,onDismiss,retryLabel='Reintentar',allowRetry=true}: {progress:Progress;busy:boolean;onRetry:()=>void;onDismiss:()=>void;retryLabel?:string;allowRetry?:boolean}) {
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
  return <section className={`install-progress ${terminal ? 'terminal' : ''}`} aria-label="Estado de la tarea">
    <div className="install-progress-heading"><div><strong>{progress.label}</strong><span role="status">{terminal ? stages[progress.stage] : cancelling ? 'Cancelando…' : `${stages[progress.stage]}${progress.attempt > 1 ? ` · Intento ${progress.attempt} de 3` : ''}`}</span></div>
      {!terminal ? cancellable&&<button disabled={cancelling} onClick={cancel}>Cancelar</button> : <div className="install-progress-actions">{allowRetry&&progress.stage !== 'completed' && <button disabled={busy} onClick={onRetry}>{retryLabel}</button>}<button className="text-button" onClick={onDismiss}>Cerrar</button></div>}
    </div>
    {!terminal && <><progress aria-label={`${stages[progress.stage]}: ${progress.label}`} value={measured && progress.total ? progress.received : undefined} max={progress.total ?? undefined}/><div className="install-transfer"><span>{progress.received ? <><AnimatedValue key={progress.label} value={progress.received} format={bytes}/>{progress.total ? ` / ${bytes(progress.total)}` : ''}</> : measured?'Esperando datos':stages[progress.stage]}{percentage != null && measured ? ` · ${percentage} %` : ''}</span><span>{progress.bytesPerSecond != null && measured ? <><AnimatedValue key={progress.label} value={progress.bytesPerSecond} format={bytes}/>/s</> : '—'}</span></div></>}
    {progress.error && progress.stage === 'failed' && <p className="form-error">{progress.error}</p>}
    {progress.stage === 'cancelled' && <p>{progress.kind==='content_remove'?'Se canceló la retirada. El archivo y su configuración se conservan.':progress.kind==='server_import'?'Se canceló la importación. La carpeta original se conserva.':progress.kind==='world_import'?'Se canceló la importación. El mundo original se conserva.':progress.kind==='restore'?'Se canceló la restauración. El servidor mantiene su estado anterior.':progress.kind==='backup'?'Se canceló la copia. Los archivos del servidor se conservan.':'Los archivos incompletos se han retirado. Puedes volver a intentarlo.'}</p>}
    {error && <p className="form-error" role="alert">No se pudo solicitar la cancelación: {error}</p>}
  </section>
}
