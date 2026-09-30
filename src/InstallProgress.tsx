import { useState } from 'react'
import { api, type Progress } from './api'
import { AnimatedValue } from './motion'

const bytes = (value: number) => value >= 1024 ** 3 ? `${(value / 1024 ** 3).toFixed(1)} GB` : `${(value / 1024 ** 2).toFixed(1)} MB`
const stages: Record<Progress['stage'],string> = {preparing:'Preparando',connecting:'Conectando',downloading:'Descargando',verifying:'Verificando',extracting:'Extrayendo',retrying:'Reintentando',completed:'Completado',cancelled:'Cancelado',failed:'Error'}

export function InstallProgress({progress,busy,onRetry,onDismiss}: {progress:Progress;busy:boolean;onRetry:()=>void;onDismiss:()=>void}) {
  const [cancelling,setCancelling] = useState(false)
  const [error,setError] = useState<string|null>(null)
  const terminal = ['completed','cancelled','failed'].includes(progress.stage)
  const percentage = progress.total ? Math.min(100,Math.floor(progress.received / progress.total * 100)) : null
  const cancel = async () => {
    setCancelling(true);setError(null)
    try { const accepted = await api.cancelInstall(progress.serverId,progress.taskId); if (!accepted) setCancelling(false) }
    catch (cause) {setError(String(cause));setCancelling(false)}
  }
  return <section className={`install-progress ${terminal ? 'terminal' : ''}`} aria-label="Estado de la instalación">
    <div className="install-progress-heading"><div><strong>{progress.label}</strong><span role="status">{terminal ? stages[progress.stage] : cancelling ? 'Cancelando…' : `${stages[progress.stage]}${progress.attempt > 1 ? ` · Intento ${progress.attempt} de 3` : ''}`}</span></div>
      {!terminal ? <button disabled={cancelling} onClick={cancel}>Cancelar</button> : <div className="install-progress-actions">{progress.stage !== 'completed' && <button disabled={busy} onClick={onRetry}>Reintentar</button>}<button className="text-button" onClick={onDismiss}>Cerrar</button></div>}
    </div>
    {!terminal && <><progress aria-label={`${stages[progress.stage]}: ${progress.label}`} value={progress.stage === 'downloading' && progress.total ? progress.received : undefined} max={progress.total ?? undefined}/><div className="install-transfer"><span>{progress.received ? <><AnimatedValue key={progress.label} value={progress.received} format={bytes}/>{progress.total ? ` / ${bytes(progress.total)}` : ''}</> : 'Esperando datos'}{percentage != null && progress.stage === 'downloading' ? ` · ${percentage} %` : ''}</span><span>{progress.bytesPerSecond != null && progress.stage === 'downloading' ? <><AnimatedValue key={progress.label} value={progress.bytesPerSecond} format={bytes}/>/s</> : '—'}</span></div></>}
    {progress.error && progress.stage === 'failed' && <p className="form-error">{progress.error}</p>}
    {progress.stage === 'cancelled' && <p>Los archivos incompletos se han retirado. Puedes volver a intentarlo.</p>}
    {error && <p className="form-error" role="alert">No se pudo solicitar la cancelación: {error}</p>}
  </section>
}
