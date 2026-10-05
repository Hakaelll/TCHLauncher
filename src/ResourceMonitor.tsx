import {PixelIcon,PixelMeter} from './PixelIndicators'
import {ServerResponsePanel} from './ServerResponsePanel'
import {api,type Metrics,type Server} from './api'
import { useEffect,useState,useRef } from 'react'
import { AnimatedLine, AnimatedValue } from './motion'
import { diskRate, HISTORY_MS, seriesSegments, type MetricSample } from './telemetry'

const bytesLabel = (bytes: number) => bytes >= 1024 ** 3 ? `${(bytes / 1024 ** 3).toFixed(1)} GB` : bytes >= 1024 ** 2 ? `${(bytes / 1024 ** 2).toFixed(1)} MB` : `${(bytes / 1024).toFixed(1)} KB`

type MonitorProps={server:Server;metrics:Metrics|null;history:MetricSample[];error:string|null}
function ResourceChart({server,metrics,history,error,metric}:MonitorProps & {metric:'memoryBytes'|'cpuPercent'}) {
  const [hoverTime,setHoverTime]=useState<number|null>(null)
  const samples = history.filter(sample => sample.pid === metrics?.pid)
  const peakMemory = Math.max(0,...samples.map(sample => sample.memoryBytes ?? 0))
  const peakCpu = Math.max(0,...samples.map(sample => sample.cpuPercent ?? 0))
  const hasPeak = samples.some(sample => sample[metric] != null)
  const maximum = metric === 'memoryBytes' ? Math.max(server.memoryMaxMb * 1024 ** 2,peakMemory * 1.1) : 100
  const latestTime = samples.at(-1)?.sampledAt ?? 0
  const x = (sample: MetricSample) => 12 + (sample.sampledAt - latestTime + HISTORY_MS) / HISTORY_MS * 476
  const y = (sample: MetricSample) => 142 - Math.min(1,(sample[metric] ?? 0) / maximum) * 128
  const segments = seriesSegments(samples,metric)
  const heapSegments=metric==='memoryBytes'?seriesSegments(samples,'heapUsedBytes'):[]
  const format = metric === 'memoryBytes' ? bytesLabel : (value: number) => `${value.toFixed(1)} %`
  const latest = metrics?.[metric]
  const current = latest == null ? '—' : format(latest)
  const hovered=samples.find(sample=>sample.sampledAt===hoverTime && sample[metric]!=null)

  return <section className={`resource-chart ${metric === 'memoryBytes' ? 'memory' : 'cpu'}`} aria-label={metric === 'memoryBytes' ? 'Memoria del proceso Java' : 'CPU del proceso Java'}>
    <div className="chart-heading"><PixelIcon kind={metric==='memoryBytes'?'memory':'cpu'}/><h3>{metric === 'memoryBytes' ? 'Memoria' : 'CPU'}</h3><span>JAVA</span></div>
    <div className="monitor-reading"><strong><AnimatedValue key={`${server.id}:${metrics?.pid}`} value={latest} format={format}/></strong><span>{metric === 'memoryBytes' ? 'RAM del proceso' : 'CPU del proceso'}</span><small>Pico {hasPeak ? format(metric === 'memoryBytes' ? peakMemory : peakCpu) : '—'}</small></div>
    <PixelMeter label={metric==='memoryBytes'?'Heap usado respecto al máximo configurado':'CPU del proceso'} ratio={metric==='memoryBytes'?metrics?.heap?metrics.heap.usedBytes/(server.memoryMaxMb*1024**2):null:latest==null?null:latest/100}/>
    <div className="monitor-chart">
      <div className="chart-scale"><span>{format(maximum)}</span><span>0</span></div>
      <svg viewBox="0 0 500 154" preserveAspectRatio="none" role="img" aria-label={`${metric === 'memoryBytes' ? 'Memoria' : 'CPU'} durante los últimos dos minutos. Valor actual: ${current}.`} onPointerLeave={()=>setHoverTime(null)} onPointerMove={event=>{const bounds=event.currentTarget.getBoundingClientRect(),position=(event.clientX-bounds.left)/bounds.width*500;const nearest=samples.filter(sample=>sample[metric]!=null).reduce<MetricSample|null>((best,sample)=>!best || Math.abs(x(sample)-position)<Math.abs(x(best)-position)?sample:best,null);setHoverTime(nearest?.sampledAt ?? null)}}>
        {[0,100,200,300,400,500].map(column=><line key={`x:${column}`} x1={column} y1="14" x2={column} y2="142" className="chart-grid chart-grid-time"/>)}
        {[14,78,142].map(row => <line key={row} x1="0" y1={row} x2="500" y2={row} className="chart-grid"/>)}
        {segments.map((segment,index) => <AnimatedLine stepped key={index} identity={`${server.id}:${metrics?.pid}:${metric}:${index}`} times={segment.map(sample=>sample.sampledAt)} points={segment.map(sample => [x(sample),y(sample)])}/>)}
        <g className="heap-chart-line">{heapSegments.map((segment,index)=><AnimatedLine stepped key={index} identity={`${server.id}:${metrics?.pid}:heap:${index}`} times={segment.map(sample=>sample.sampledAt)} points={segment.map(sample=>[x(sample),142-Math.min(1,(sample.heapUsedBytes??0)/maximum)*128])}/>)}</g>
        {hovered && <rect className="chart-point" x={x(hovered)-3} y={y(hovered)-3} width="6" height="6"/>}
      </svg>
      {hovered && <div className="chart-tooltip" role="tooltip" style={{left:`${Math.max(16,Math.min(84,x(hovered)/5))}%`}}><strong>{format(hovered[metric]!)}</strong>{metric==='memoryBytes'&&hovered.heapUsedBytes!=null&&<span>Heap usado: {bytesLabel(hovered.heapUsedBytes)}</span>}<span>{new Date(hovered.sampledAt).toLocaleTimeString('es-ES')}</span></div>}
      {!samples.length && <p className="chart-empty">{error ? 'No se pudo actualizar el rendimiento.' : metrics?.pid ? 'Recogiendo muestras…' : 'Inicia el servidor para ver su rendimiento.'}</p>}
    </div>
    <div className="chart-times"><span>Hace 2 min</span><span>Ahora</span></div>
    {metric==='memoryBytes'&&<p className="heap-chart-legend"><span>━━ RAM del proceso</span><span>┄┄ Heap usado {metrics?.heap?bytesLabel(metrics.heap.usedBytes):'no disponible'}</span></p>}
    {error && <p className="network-error" role="status">{error}</p>}
  </section>
}

export function ResourceMonitor(props:MonitorProps) {
  const samples=props.history.filter(sample=>sample.pid===props.metrics?.pid)
  const readRate=props.metrics?.pid ? diskRate(samples,'diskReadBytes'):null
  const writeRate=props.metrics?.pid ? diskRate(samples,'diskWrittenBytes'):null
  return <section className="resource-monitor"><div className="section-heading"><div><h2>Rendimiento</h2><p>Proceso Java · últimos dos minutos</p></div></div>
    <div className="performance-charts"><ResourceChart {...props} metric="memoryBytes"/><ResourceChart {...props} metric="cpuPercent"/></div>
    <section className="heap-diagnostics" aria-label="Memoria interna de Java"><MemoryExport key={props.server.id} id={props.server.id}/><dl className="monitor-details"><div><dt>Heap usado</dt><dd>{props.metrics?.heap?bytesLabel(props.metrics.heap.usedBytes):'—'}</dd></div><div><dt>Heap reservado</dt><dd>{props.metrics?.heap?bytesLabel(props.metrics.heap.committedBytes):'—'}</dd></div><div><dt>Eventos del recolector</dt><dd>{props.metrics?.heap?.collections.toLocaleString('es-ES')??'—'}</dd></div></dl><p className="field-help">La gráfica mide la RAM residente del proceso, incluidos heap y memoria nativa. Java puede reutilizar heap reservado sin devolverlo inmediatamente a Windows o Linux. El heap usado incluye objetos aún no recogidos; no equivale a memoria retenida después de una recolección.</p>{props.metrics?.heapError&&<details><summary>Heap no disponible</summary><p className="field-help">{props.metrics.heapError}</p></details>}{props.metrics?.heap&&<p className="field-help">Contadores HotSpot leídos a las {new Date(props.metrics.heap.sampledAt).toLocaleTimeString('es-ES')}. Son muestras de Java; no se fuerza ninguna recolección.</p>}</section>
    <ServerResponsePanel key={props.server.id} server={props.server}/>
    <TickMonitor key={`${props.server.id}:${props.metrics?.pid}`} server={props.server} metrics={props.metrics}/>
    <dl className="monitor-details"><div><dt>Lectura de disco</dt><dd>{readRate==null?'—':`${bytesLabel(readRate)}/s`}</dd></div><div><dt>Escritura de disco</dt><dd>{writeRate==null?'—':`${bytesLabel(writeRate)}/s`}</dd></div><div><dt>Heap máximo de Java</dt><dd>{bytesLabel(props.server.memoryMaxMb*1024**2)}</dd></div><div><dt>Proceso</dt><dd>{props.metrics?.pid??'—'}</dd></div><div><dt>Tiempo de arranque</dt><dd>{props.metrics?.startupDurationMs==null?'—':`${(props.metrics.startupDurationMs/1000).toLocaleString('es-ES',{minimumFractionDigits:1,maximumFractionDigits:1})} s`}</dd></div></dl>
    <p className="field-help">Arranque: desde el lanzamiento de Java hasta que el servidor comunica que está listo. Se mide para el proceso actual; una salida antes de estar listo no produce un valor.</p>
  </section>
}

function TickMonitor({server,metrics}:{server:Server;metrics:Metrics|null}){
  const [waiting,setWaiting]=useState(false)
  const [requested,setRequested]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const reading=metrics?.tps?.reading
  useEffect(()=>{if(!waiting)return;const timer=setTimeout(()=>setWaiting(false),11_000);return()=>clearTimeout(timer)},[waiting])
  useEffect(()=>{if(reading)setWaiting(false)},[reading?.sampledAt])
  const query=async()=>{
    setError(null);setWaiting(true)
    try{await api.requestTps(server.id);setRequested(true)}catch(cause){setWaiting(false);setError(cause instanceof Error?cause.message:String(cause))}
  }
  const pending=waiting||metrics?.tps?.pending
  return <section className="tick-monitor" aria-label="TPS del servidor"><div className="section-heading"><div><h3>Ritmo del servidor</h3><p>{server.platform==='paper'?'Medias de TPS comunicadas por Paper. Objetivo habitual: 20.':'Este motor no ofrece una fuente de TPS integrada.'}</p></div>{server.platform==='paper'&&<button disabled={server.state!=='running'||!!pending} onClick={()=>void query()}>{pending?'Consultando…':'Consultar TPS'}</button>}</div>
    <dl className="monitor-details">{[['Último minuto',reading?.oneMinute],['Últimos 5 minutos',reading?.fiveMinutes],['Últimos 15 minutos',reading?.fifteenMinutes]].map(([label,value])=><div key={label}><dt>{label}</dt><dd className={typeof value==='number'&&value<16?'tick-low':typeof value==='number'&&value<18?'tick-warning':undefined}>{typeof value==='number'?value.toFixed(1):'—'}</dd></div>)}</dl>
    <p className="field-help" role="status">{server.platform!=='paper'?'TPS no disponibles para este motor.':pending?'Esperando respuesta del proceso Java…':reading?`Consulta recibida a las ${new Date(reading.sampledAt).toLocaleTimeString()}. La lectura caduca tras un minuto.`:server.state!=='running'?'Inicia el servidor para consultar su ritmo.':requested?'No hay una lectura reciente. El comando puede no estar disponible en este servidor.':'Sin lectura: pulsa Consultar TPS para solicitarla.'}</p>
    {error&&<p className="form-error" role="alert">{error}</p>}
  </section>
}

function MemoryExport({id}:{id:string}) {
  const [busy,setBusy]=useState(false),[error,setError]=useState<string|null>(null),[saved,setSaved]=useState<string|null>(null)
  const active=useRef(true),inFlight=useRef(false)
  useEffect(()=>{active.current=true;return()=>{active.current=false}},[])
  const save=async()=>{
    if(inFlight.current)return
    inFlight.current=true;setBusy(true);setError(null);setSaved(null)
    try{const path=await api.exportMemoryReport(id);if(active.current)setSaved(path)}
    catch(cause){if(active.current)setError(cause instanceof Error?cause.message:String(cause))}
    finally{inFlight.current=false;if(active.current)setBusy(false)}
  }
  return <><div className="section-heading"><h3>Memoria de Java</h3><button disabled={busy} onClick={()=>void save()}>{busy?'Guardando…':'Exportar diagnóstico'}</button></div>
    <p className="field-help">Hasta 30 minutos de muestras recogidas con este servidor seleccionado. Sin logs ni comandos. Guarda el archivo en el equipo que aloja el servidor.</p>
    {error&&<p className="form-error" role="alert">{error}</p>}{saved&&<p className="field-help" role="status">Diagnóstico guardado: {saved}</p>}</>
}
