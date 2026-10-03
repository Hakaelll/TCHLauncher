import {useEffect,useState} from 'react'
import {api,type CrashReport,type Server} from './api'

export function CrashPanel({server,onConsole,onSettings,onContent}:{server:Server;onConsole:()=>void;onSettings:()=>void;onContent:()=>void}){
  const [report,setReport]=useState<CrashReport|null>(null)
  const [error,setError]=useState('')
  const [loading,setLoading]=useState(server.state==='crashed')
  useEffect(()=>{
    let alive=true;setReport(null);setError('');setLoading(server.state==='crashed')
    if(server.state==='crashed')void api.crashReport(server.id).then(value=>{if(alive)setReport(value)},reason=>{if(alive)setError(String(reason))}).finally(()=>{if(alive)setLoading(false)})
    return()=>{alive=false}
  },[server.id,server.state])
  if(server.state!=='crashed')return null
  const review=(action:string)=>{
    if(action==='settings')onSettings()
    else if(action==='content'&&server.platform!=='vanilla')onContent()
    else if(action==='network'){const heading=document.getElementById('network-title');heading?.scrollIntoView({block:'start',behavior:'instant'});heading?.focus({preventScroll:true})}
    else onConsole()
  }
  return <section className="crash-diagnostics" aria-labelledby="crash-title">
    <div className="section-heading"><div><h2 id="crash-title">Revisar el cierre</h2>{report&&<p>{new Date(report.endedAt).toLocaleString()} · {report.exitCode===null?'Código de salida no disponible':`Código de salida ${report.exitCode}`}</p>}</div><button onClick={onConsole}>Ver consola</button></div>
    {loading?<p className="field-help" role="status">Leyendo el diagnóstico de este arranque…</p>:error?<p role="alert">No se pudo leer el diagnóstico: {error}</p>:!report?<p className="field-help">No hay un diagnóstico guardado para este arranque. Revisa la consola.</p>:<>
      {report.stopRequested&&<p className="inline-note">Habías solicitado detener el servidor; el código indica que Java no terminó correctamente.</p>}
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
