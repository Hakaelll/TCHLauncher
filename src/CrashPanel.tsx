import {useEffect,useState} from 'react'
import {api,type CrashReport,type Server} from './api'

type Props={server:Server;onConsole:()=>void;onSettings:()=>void;onContent:()=>void}
export function CrashPanel(props:Props){
  if(props.server.state==='crashed')return <CrashReview key={`${props.server.id}:current`} {...props} historical={false}/>
  if(!['stopped','running'].includes(props.server.state))return null
  return <CrashHistory key={`${props.server.id}:${props.server.state}`} {...props}/>
}
function CrashHistory(props:Props){
  const [expanded,setExpanded]=useState(false)
  return <details className="crash-history" onToggle={event=>setExpanded(event.currentTarget.open)}>
    <summary>Consultar el último fallo guardado</summary>
    {expanded&&<CrashReview {...props} historical/>}
  </details>
}
function CrashReview({server,onConsole,onSettings,onContent,historical}:Props&{historical:boolean}){
  const [report,setReport]=useState<CrashReport|null>(null)
  const [error,setError]=useState('')
  const [loading,setLoading]=useState(true)
  useEffect(()=>{
    let alive=true;setReport(null);setError('');setLoading(true)
    void (historical?api.lastCrashReport(server.id):api.crashReport(server.id)).then(value=>{if(alive)setReport(value)},reason=>{if(alive)setError(String(reason))}).finally(()=>{if(alive)setLoading(false)})
    return()=>{alive=false}
  },[server.id,server.state,historical])
  const review=(action:string)=>{
    if(action==='settings')onSettings()
    else if(action==='content'&&server.platform!=='vanilla')onContent()
    else if(action==='network'){const heading=document.getElementById('network-title');heading?.scrollIntoView({block:'start',behavior:'instant'});heading?.focus({preventScroll:true})}
    else onConsole()
  }
  return <section className="crash-diagnostics" aria-labelledby="crash-title">
    <div className="section-heading"><div><h2 id="crash-title">{historical?'Último fallo guardado':'Revisar el cierre'}</h2>{report&&<p>{new Date(report.endedAt).toLocaleString()} · {report.exitCode===null?'Código de salida no disponible':`Código de salida ${report.exitCode}`}</p>}</div><button onClick={onConsole}>{historical?'Ver consola actual':'Ver consola'}</button></div>
    {historical&&<p className="field-help">Este informe pertenece a una ejecución anterior. No describe el estado actual del servidor.</p>}
    {loading?<p className="field-help" role="status">{historical?'Leyendo el último fallo guardado…':'Leyendo el diagnóstico de este arranque…'}</p>:error?<p role="alert">No se pudo leer el diagnóstico: {error}</p>:!report?<p className="field-help">{historical?'No hay un fallo guardado para este servidor.':'No hay un diagnóstico guardado para este arranque. Revisa la consola.'}</p>:<>
      {report.stopRequested&&<p className="inline-note">Habías solicitado detener el servidor, pero Java no confirmó un cierre correcto.</p>}
      {!report.outputComplete&&<p className="inline-note">Parte de la salida seguía abierta al registrar el cierre. El diagnóstico puede estar incompleto.</p>}
      <div className="crash-findings">{report.diagnoses.map(diagnosis=><div className="crash-finding" key={diagnosis.kind}>
        <h3>{diagnosis.title}</h3><p>{diagnosis.explanation}</p><p className="crash-recommendation">{diagnosis.recommendation}</p>
        {diagnosis.action!=='console'&&<button onClick={()=>review(diagnosis.action)}>{diagnosis.action==='settings'?'Revisar configuración':diagnosis.action==='network'?'Revisar conexión':server.platform==='vanilla'?'Ver consola':server.platform==='paper'?'Revisar plugins':'Revisar mods'}</button>}
        <details><summary>Detalle técnico</summary><pre>{diagnosis.evidence.length?diagnosis.evidence.join('\n'):'No se conservó salida del proceso.'}</pre></details>
      </div>)}</div>
      {report.recentFiles.length>0&&<details className="crash-recent"><summary>Archivos de mods/plugins modificados en las últimas 24 horas ({report.recentFiles.length})</summary><p className="field-help">La fecha del archivo no demuestra que causara el fallo. Los archivos copiados también pueden conservar su fecha original.</p><ul>{report.recentFiles.map(file=><li key={`${file.folder}/${file.name}`}><code>{file.folder}/{file.name}</code><time>{new Date(file.modifiedAt).toLocaleString()}</time></li>)}</ul></details>}
    </>}
  </section>
}
