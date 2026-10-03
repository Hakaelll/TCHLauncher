import type {Metrics} from './api'
const bytes=(value:number)=>value>=1024**3?`${(value/1024**3).toFixed(1)} GB`:value>=1024**2?`${(value/1024**2).toFixed(1)} MB`:`${Math.round(value/1024)} KB`
export function StorageMeter({metrics,targetGb,onSettings}:{metrics:Metrics|null;targetGb:number;onSettings:()=>void}){
  const size=metrics?.sizeBytes
  const known=size!=null
  const percent=known?Math.round(size/(targetGb*1024**3)*100):null
  const warning=known&&size>=targetGb*1024**3*(metrics?.storageWarningPercent??90)/100
  return <div className="resource-meter">
    <div><span>Espacio del servidor</span><strong>{known?`${bytes(size)} / ${targetGb} GB`:metrics?'Medición no disponible':'Calculando…'}</strong></div>
    {percent!=null&&<progress aria-label="Espacio ocupado respecto al objetivo" value={Math.min(percent,100)} max={100}/>}
    <p>{warning?`Ha alcanzado el ${metrics?.storageWarningPercent??90} % del objetivo. Revisa mundos y logs.`:known?'Objetivo de aviso; no limita el disco.':metrics?'Revisa que la carpeta del servidor siga disponible. Volveremos a medirla automáticamente.':'Esperando la lectura del almacenamiento.'}</p>
    {known&&metrics?.sizeSampledAt!=null&&<p className="field-help">Medido a las {new Date(metrics.sizeSampledAt).toLocaleTimeString('es-ES')} · se actualiza cada minuto.</p>}
    {known&&!!metrics?.sizeSkippedLinks&&<p className="field-help">{metrics.sizeSkippedLinks} enlaces simbólicos no incluidos.</p>}
    {metrics?.sizeError&&<><details className="storage-measure-error"><summary>No se pudo medir el almacenamiento</summary><p className="form-error">{metrics.sizeError}</p></details><button className="text-button" onClick={onSettings}>Revisar almacenamiento</button></>}
  </div>
}
