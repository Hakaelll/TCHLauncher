import {localizedStatic,t,translatedMessage} from './locale'

const labels=localizedStatic({properties:'Propiedades',backups:'Copias',eula:'EULA',servers:'Servidores'})
export type RestoreSyncState={kind:'loading'}|{kind:'failed';failures:{section:keyof typeof labels;message:string}[]}

export function RestoreSyncNotice({state,busy,onRetry}:{state:RestoreSyncState;busy:boolean;onRetry:()=>void}){
  const failed=state.kind==='failed'
  return <div className="restore-sync-notice">
    <div className="restore-sync-content" role={failed?'alert':'status'}>
      <p className={failed?'form-error':'field-help'}>{failed?t('La copia se restauró, pero faltan datos por actualizar.'):t('Copia restaurada. Actualizando datos…')}</p>
      {state.kind==='failed'&&<ul className="restore-sync-errors">{state.failures.map(failure=><li key={failure.section}><strong>{labels[failure.section]}</strong> {translatedMessage(failure.message)}</li>)}</ul>}
    </div>
    <button disabled={busy||state.kind==='loading'} onClick={onRetry}>{t('Actualizar datos')}</button>
  </div>
}
