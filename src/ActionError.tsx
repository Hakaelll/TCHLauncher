import type {Server} from './api'
import {localizedStatic,t,translatedMessage} from './locale'
import './actionError.css'

export type ActionContext={operation:'start'|'restart'|'runtime';serverId:string;serverName:string}
export type ActionFailure=ActionContext&{detail:string}
export function actionContext(server:Server,operation:ActionContext['operation']):ActionContext {
  return {operation,serverId:server.id,serverName:server.name}
}
const titles=localizedStatic({start:'No se pudo iniciar «{0}»',restart:'No se pudo reiniciar «{0}»',runtime:'No se pudo preparar Java para «{0}»'})
const advice=localizedStatic({
  start:'Revisa Java, la memoria y el puerto en Configuración. La consola puede aportar información del intento de arranque.',
  restart:'Comprueba el estado actual y la consola antes de volver a iniciar. El reinicio puede haberse detenido después de cerrar el proceso anterior.',
  runtime:'Comprueba el runtime en Configuración. Allí puedes verificar Java, reinstalar Temurin o elegir un ejecutable compatible.',
})

export function ActionError({failure,available,onReview,onClose}:{
  failure:ActionFailure;available:boolean;onReview:(section:'settings'|'console')=>void;onClose:()=>void
}) {
  return <section className="action-error" role="alert" aria-label={t('Error de la acción')}>
    <div className="action-error-heading"><strong>{t(titles[failure.operation],failure.serverName)}</strong><button className="action-error-close" onClick={onClose} aria-label={t('Cerrar error')}>×</button></div>
    <p>{translatedMessage(failure.detail)}</p>
    <p className="action-error-advice">{advice[failure.operation]}</p>
    {!available&&<p>{t('Este servidor ya no está en la biblioteca. El detalle del fallo se conserva aquí.')}</p>}
    <div className="action-error-actions"><button disabled={!available} onClick={()=>onReview(failure.operation==='restart'?'console':'settings')}>{failure.operation==='restart'?t('Ver consola'):t('Revisar configuración')}</button>
      {failure.operation==='start'&&<button disabled={!available} onClick={()=>onReview('console')}>{t('Ver consola')}</button>}
    </div>
    <details><summary>{t('Detalle técnico')}</summary><pre tabIndex={0}>{JSON.stringify({serverId:failure.serverId,operation:failure.operation,message:failure.detail},null,2)}</pre></details>
  </section>
}
