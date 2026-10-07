import {fmtBytes as bytes} from './format'
import {translatedMessage,t,getLanguage} from './locale.ts'
import type {Metrics} from './api'
export function StorageMeter({metrics,targetGb,onSettings}:{metrics:Metrics|null;targetGb:number;onSettings:()=>void}){
  const size=metrics?.sizeBytes
  const known=size!=null
  const percent=known?Math.round(size/(targetGb*1024**3)*100):null
  const warning=known&&size>=targetGb*1024**3*(metrics?.storageWarningPercent??90)/100
  return <div className="resource-meter">
    <div><span>{t("Espacio del servidor")}</span><strong>{known?`${bytes(size)} / ${targetGb} GB`:metrics?t("Medición no disponible"):t("Calculando…")}</strong></div>
    {percent!=null&&<progress aria-label={t("Espacio ocupado respecto al objetivo")} value={Math.min(percent,100)} max={100}/>}
    <p>{warning?t("Ha alcanzado el {0} % del objetivo. Revisa mundos y logs.", metrics?.storageWarningPercent??90):known?t("Objetivo de aviso; no limita el disco."):metrics?t("Revisa que la carpeta del servidor siga disponible. Volveremos a medirla automáticamente."):t("Esperando la lectura del almacenamiento.")}</p>
    {known&&metrics?.sizeSampledAt!=null&&<p className="field-help">{t("Medido a las")} {new Date(metrics.sizeSampledAt).toLocaleTimeString(getLanguage()==='en'?'en-US':'es-ES')}  {t("· se actualiza cada minuto.")}</p>}
    {known&&!!metrics?.sizeSkippedLinks&&<p className="field-help">{metrics.sizeSkippedLinks}  {t("enlaces simbólicos no incluidos.")}</p>}
    {metrics?.sizeError&&<><details className="storage-measure-error"><summary>{t("No se pudo medir el almacenamiento")}</summary><p className="form-error">{translatedMessage(metrics.sizeError)}</p></details><button className="text-button" onClick={onSettings}>{t("Revisar almacenamiento")}</button></>}
  </div>
}
