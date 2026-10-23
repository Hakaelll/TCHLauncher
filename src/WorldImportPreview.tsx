import type {WorldInspection} from './api'
import {t} from './locale.ts'
import './worldImport.css'

export function WorldImportPreview({source,name,platform}: {source:WorldInspection;name:string;platform:string}) {
  const target=name.trim()||source.suggestedName
  const incompatible=source.dimensions.length>0&&platform!=='paper'
  return <div className="world-import-preview">
    <h3>{t('Carpetas que se copiarán')}</h3>
    <ul>{[source.suggestedName,...source.dimensions].map((part,index)=>{
      const suffix=part.slice(source.suggestedName.length)
      const dimension=index===0?'overworld':suffix==='_nether'?'nether':'end'
      return <li key={part}><span className={`world-dimension-mark ${dimension}`} aria-hidden="true"/>
        <div><strong>{index===0?t('Mundo principal'):suffix==='_nether'?'Nether':'End'}</strong><small>{part}</small></div>
        <code title={t('Nombre en este servidor')}>{target}{suffix}</code>
      </li>
    })}</ul>
    <p className={incompatible?'form-error':'field-help'} role={incompatible?'alert':undefined}>{incompatible
      ?t('Este mundo tiene dimensiones separadas de Paper. Elige un servidor Paper; no se convierten automáticamente a otros tipos')
      :t('Detén el servidor de origen antes de copiar. Las carpetas originales se conservan.')}</p>
  </div>
}
