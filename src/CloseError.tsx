import type {CloseFailure} from './api'
import {t,translatedMessage} from './locale'

function description(cause:CloseFailure['cause']):string {
  switch(cause.kind){
    case 'stop':return translatedMessage(cause.message)
    case 'backup':return t('Copia tras la parada: {0}',translatedMessage(cause.message))
    case 'timeout':return t('No se detuvo a tiempo. Revisa la consola; la aplicación permanece abierta.')
    default:{const exhaustive:never=cause;return exhaustive}
  }
}

export function CloseError({value}:{value:string|CloseFailure[]}){
  return <div className="form-error close-operation-error" role="alert" tabIndex={0}>
    {typeof value==='string'?translatedMessage(value):<ul className="close-failure-list">{value.map(failure=><li key={failure.serverId}>
      <strong>{failure.serverName}</strong><span>{description(failure.cause)}</span>
    </li>)}</ul>}
  </div>
}
