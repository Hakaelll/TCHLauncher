import {translatedMessage,t,formatDate} from './locale.ts'
import {useState} from 'react'
import {api,type CrashReport,type Server} from './api'
import {useServerRead} from './useServerRead'

const readHistory=async(id:string)=>({report:await api.lastCrashReport(id)})
const readCurrent=async(id:string)=>({report:await api.crashReport(id)})

type Props={server:Server;onConsole:()=>void;onSettings:()=>void;onContent:()=>void}
export function CrashPanel(props:Props){
  if(props.server.state==='crashed')return <CrashReview key={`${props.server.id}:current`} {...props} historical={false}/>
  if(!['stopped','running'].includes(props.server.state))return null
  return <CrashHistory key={`${props.server.id}:${props.server.state}`} {...props}/>
}
function CrashHistory(props:Props){
  const snapshot=useServerRead(props.server.id,readHistory)
  const report=snapshot.data?.report
  const [expanded,setExpanded]=useState(false)
  if(snapshot.error)return <div className="panel-read-state failed"><p role="alert">{t("No se pudo consultar el historial de fallos:")} {translatedMessage(snapshot.error)}</p><button disabled={snapshot.loading} onClick={()=>void snapshot.refresh(props.server.id)}>{snapshot.loading?t("Consultando…"):t("Reintentar lectura")}</button></div>
  if(!report)return null
  return <details className="crash-history" onToggle={event=>setExpanded(event.currentTarget.open)}>
    <summary>{report.forced?t("Último cierre forzado"):t("Último cierre con error")} · {formatDate(report.endedAt)}</summary>
    {expanded&&<CrashReview {...props} historical initial={report}/>}
  </details>
}
function CrashReview({server,onConsole,onSettings,onContent,historical,initial}:Props&{historical:boolean;initial?:CrashReport}){
  const snapshot=useServerRead(initial?null:server.id,historical?readHistory:readCurrent)
  const report=initial??snapshot.data?.report
  const error=snapshot.error
  const loading=!initial&&(snapshot.loading||snapshot.data===null&&!error)
  const review=(action:string)=>{
    if(action==='settings')onSettings()
    else if(action==='content'&&server.platform!=='vanilla')onContent()
    else if(action==='network'){const heading=document.getElementById('network-title');heading?.scrollIntoView({block:'start',behavior:'instant'});heading?.focus({preventScroll:true})}
    else onConsole()
  }
  return <section className="crash-diagnostics" aria-labelledby="crash-title">
    <div className="section-heading"><div><h2 id="crash-title">{report?.forced?t("Cierre forzado"):historical?t("Último fallo guardado"):t("Revisar el cierre")}</h2>{report&&<p>{formatDate(report.endedAt)} · {report.exitCode===null?t("Código de salida no disponible"):t("Código de salida {0}", report.exitCode)}</p>}</div><button onClick={onConsole}>{historical?t("Ver consola actual"):t("Ver consola")}</button></div>
    {historical&&<p className="field-help">{t("Este informe pertenece a una ejecución anterior. No describe el estado actual del servidor.")}</p>}
    {loading?<p className="field-help" role="status">{historical?t("Leyendo el último fallo guardado…"):t("Leyendo el diagnóstico de este arranque…")}</p>:error?<div className="panel-read-state failed"><p role="alert">{t("No se pudo leer el diagnóstico:")} {translatedMessage(error)}</p><button onClick={()=>void snapshot.refresh(server.id)}>{t("Reintentar lectura")}</button></div>:!report?<p className="field-help">{historical?t("No hay un fallo guardado para este servidor."):t("No hay un diagnóstico guardado para este arranque. Revisa la consola.")}</p>:<>
      {report.stopRequested&&!report.forced&&<p className="inline-note">{t("Habías solicitado detener el servidor, pero Java no confirmó un cierre correcto.")}</p>}
      {!report.outputComplete&&<p className="inline-note">{t("No se pudo recoger toda la salida de Java. El diagnóstico puede estar incompleto.")}</p>}
      <div className="crash-findings">{report.diagnoses.map(diagnosis=><div className="crash-finding" key={diagnosis.kind}>
        <h3>{translatedMessage(diagnosis.title)}</h3><p>{translatedMessage(diagnosis.explanation)}</p><p className="crash-recommendation">{translatedMessage(diagnosis.recommendation)}</p>
        {diagnosis.action!=='console'&&<button onClick={()=>review(diagnosis.action)}>{diagnosis.action==='settings'?t("Revisar configuración"):diagnosis.action==='network'?t("Revisar conexión"):server.platform==='vanilla'?t("Ver consola"):server.platform==='paper'?t("Revisar plugins"):t("Revisar mods")}</button>}
        <details><summary>{t("Detalle técnico")}</summary><pre>{diagnosis.evidence.length?diagnosis.evidence.join('\n'):t("No se conservó salida del proceso.")}</pre></details>
      </div>)}</div>
      {report.recentFiles.length>0&&<details className="crash-recent"><summary>{t("Archivos de mods/plugins modificados en las últimas 24 horas (")}{report.recentFiles.length})</summary><p className="field-help">{t("La fecha del archivo no demuestra que causara el fallo. Los archivos copiados también pueden conservar su fecha original.")}</p><ul>{report.recentFiles.map(file=><li key={`${file.folder}/${file.name}`}><code>{file.folder}/{file.name}</code><time>{formatDate(file.modifiedAt)}</time></li>)}</ul></details>}
    </>}
  </section>
}
