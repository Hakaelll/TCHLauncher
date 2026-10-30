import {errorText} from './format.ts'
import {translatedMessage,t,formatNumber,formatTime} from './locale.ts'
import {PixelIcon} from './PixelIndicators'
import {ServerResponsePanel} from './ServerResponsePanel'
import {api,type Metrics,type Server} from './api'
import { useEffect,useState,useRef } from 'react'
import { AnimatedLine, AnimatedValue } from './motion'
import { diskRate, HISTORY_MS, seriesSegments, type MetricSample } from './telemetry'
import {chartSamples,inspectionSamples,stepSample,memoryDomain,cpuDomain,measured,type ChartMetric} from './chartData'

const bytesLabel = (bytes: number) => bytes >= 1024 ** 3 ? `${(bytes / 1024 ** 3).toFixed(1)} GB` : bytes >= 1024 ** 2 ? `${(bytes / 1024 ** 2).toFixed(1)} MB` : `${(bytes / 1024).toFixed(1)} KB`

type MonitorProps={server:Server;metrics:Metrics|null;history:MetricSample[];error:string|null}
function ResourceChart({server,metrics,history,error,metric,windowMs}:MonitorProps & {metric:'memoryBytes'|'cpuPercent';windowMs:number}) {
  const [hoverTime,setHoverTime]=useState<number|null>(null)
  const [showProcess,setShowProcess]=useState(true),[showHeap,setShowHeap]=useState(true)
  const [autoCpu,setAutoCpu]=useState(true)
  const memory=metric==='memoryBytes'
  const samples=chartSamples(history,server.id,metrics?.pid,windowMs)
  const peakValues=samples.map(sample=>sample[metric]).filter(measured)
  const peak=peakValues.length?Math.max(...peakValues):null
  const maximum=memory?memoryDomain(samples,server.memoryMaxMb*1024**2):cpuDomain(peak,autoCpu)
  const latestTime = samples.at(-1)?.sampledAt ?? 0
  const x = (sample: MetricSample) => 12 + (sample.sampledAt - latestTime + windowMs) / windowMs * 476
  const y = (sample: MetricSample,key:ChartMetric=metric) => 142 - Math.min(1,(sample[key] ?? 0) / maximum) * 128
  const keys:ChartMetric[]=memory?[...(showProcess?['memoryBytes' as const]:[]),...(showHeap?['heapUsedBytes' as const]:[])]:['cpuPercent']
  const inspectable=inspectionSamples(samples,keys)
  const segments = showProcess?seriesSegments(samples,metric):[]
  const heapSegments=memory&&showHeap?seriesSegments(samples,'heapUsedBytes'):[]
  const hasHeap=samples.some(sample=>measured(sample.heapUsedBytes))
  const format = memory ? bytesLabel : (value: number) => `${value.toFixed(1)} %`
  const latest = metrics?.[metric]
  const current = latest == null ? '—' : format(latest)
  const hovered=inspectable.find(sample=>sample.sampledAt===hoverTime)
  const selected=hovered??samples.at(-1)
  const pointKey=hovered?keys.find(key=>measured(hovered[key])):undefined
  const area=(segment:MetricSample[],key:ChartMetric)=>[`${x(segment[0])},142`,...segment.flatMap((sample,i)=>i===0?[`${x(sample)},${y(sample,key)}`]:[`${x(sample)},${y(segment[i-1],key)}`,`${x(sample)},${y(sample,key)}`]),`${x(segment[segment.length-1])},142`].join(' ')

  return <section className={`resource-chart ${metric === 'memoryBytes' ? 'memory' : 'cpu'}`} aria-label={metric === 'memoryBytes' ? t("Memoria del proceso Java") : t("CPU del proceso Java")}>
    <div className="chart-heading"><PixelIcon kind={memory?'memory':'cpu'}/><div><h3>{memory?t("Memoria"):'CPU'}</h3><p>{memory?t("RAM residente del proceso Java"):t("Uso del proceso Java")}</p></div>{(memory||autoCpu)&&<span className="chart-axis-note">{t("Escala automática")}</span>}</div>
    <div className="monitor-reading"><strong><AnimatedValue key={`${server.id}:${metrics?.pid}`} value={latest} format={format}/></strong><small><span>{t("Pico del intervalo")}</span><b>{peak===null?'—':format(peak)}</b></small></div>
    <div className="chart-sample-reading" aria-live={hovered?'polite':'off'}><time>{selected?formatTime(selected.sampledAt):'—'}</time><span>{hovered?t("Muestra seleccionada"):t("Última muestra")}</span>{memory&&showProcess&&<strong className="process-sample">{t("RAM")} {measured(selected?.memoryBytes)?bytesLabel(selected!.memoryBytes!):'—'}</strong>}{memory&&showHeap&&<strong className="heap-sample">{t("Heap")} {measured(selected?.heapUsedBytes)?bytesLabel(selected!.heapUsedBytes!):'—'}</strong>}{!memory&&<strong className="cpu-sample">{measured(selected?.cpuPercent)?format(selected!.cpuPercent!):'—'}</strong>}</div>

    <div className="monitor-chart">
      <div className="chart-scale"><span>{format(maximum)}</span><span>{format(maximum/2)}</span><span>0</span></div>
      <svg viewBox="0 0 500 154" preserveAspectRatio="none" role="img" tabIndex={inspectable.length?0:undefined} onKeyDown={event=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;event.preventDefault();setHoverTime(stepSample(inspectable,hoverTime,event.key as 'ArrowLeft'|'ArrowRight'|'Home'|'End'))}} onBlur={()=>setHoverTime(null)} aria-label={t("{0} durante los últimos {1} segundos. Valor actual: {2}. Usa las flechas para explorar las muestras.",memory?t("Memoria"):'CPU',windowMs/1000,current)} onPointerLeave={()=>setHoverTime(null)} onPointerMove={event=>{const bounds=event.currentTarget.getBoundingClientRect(),position=(event.clientX-bounds.left)/bounds.width*500;const nearest=inspectable.reduce<MetricSample|null>((best,sample)=>!best || Math.abs(x(sample)-position)<Math.abs(x(best)-position)?sample:best,null);setHoverTime(nearest?.sampledAt ?? null)}}>
        {segments.map((segment,index)=><polygon key={`area:${index}`} className="chart-area" points={area(segment,metric)}/>)}
        {!showProcess&&heapSegments.map((segment,index)=><polygon key={`heap-area:${index}`} className="chart-area heap-area" points={area(segment,'heapUsedBytes')}/>)}
        {[14,46,78,110,142].map(row => <line key={row} x1="12" y1={row} x2="488" y2={row} className={`chart-grid ${row===14||row===78||row===142?'major':'minor'}`}/>)}
        {segments.map((segment,index) => <AnimatedLine stepped key={index} identity={`${server.id}:${metrics?.pid}:${metric}:${windowMs}:${index}`} times={segment.map(sample=>sample.sampledAt)} points={segment.map(sample => [x(sample),y(sample)])}/>)}
        <g className="heap-chart-line">{heapSegments.map((segment,index)=><AnimatedLine stepped key={index} identity={`${server.id}:${metrics?.pid}:heap:${windowMs}:${index}`} times={segment.map(sample=>sample.sampledAt)} points={segment.map(sample=>[x(sample),y(sample,'heapUsedBytes')])}/>)}</g>
        {segments.map((segment,index)=>{const sample=segment.at(-1)!;return <rect key={`point:${index}`} className="chart-point" x={x(sample)-2} y={y(sample)-2} width="4" height="4"/>})}
        {heapSegments.map((segment,index)=>{const sample=segment.at(-1)!;return <rect key={`heap-point:${index}`} className="heap-point" x={x(sample)-2} y={y(sample,'heapUsedBytes')-2} width="4" height="4"/>})}
        {hovered&&pointKey&&<><line className="chart-cursor" x1={x(hovered)} y1="14" x2={x(hovered)} y2="142"/><rect className={pointKey==='heapUsedBytes'?'heap-point':'chart-point'} x={x(hovered)-3} y={y(hovered,pointKey)-3} width="6" height="6"/></>}
      </svg>
      {!inspectable.length&&<p className="chart-empty"><strong>{error?t("Lectura no disponible"):memory&&!showProcess?t("Heap no disponible"):metrics?.pid?t("Recogiendo muestras…"):t("Esperando al servidor")}</strong><span>{metrics?.pid?t("Las lecturas aparecerán aquí."):t("Inicia el servidor para medir su rendimiento.")}</span></p>}
    </div>
    <div className="chart-times"><span>−{windowMs/1000} s</span><span>−{windowMs/2000} s</span><span>{t("Última muestra")}</span></div>
    <div className="chart-legend" role={memory?'group':undefined} aria-label={memory?t("Series de memoria visibles"):undefined}>{memory?<><button className="process-legend" aria-pressed={showProcess} disabled={showProcess&&(!showHeap||!hasHeap)} onClick={()=>setShowProcess(value=>!value)}>{t("RAM del proceso")}</button><button className="heap-legend" aria-pressed={showHeap} disabled={!showProcess||!hasHeap} onClick={()=>setShowHeap(value=>!value)}>{t("Heap usado")}{!hasHeap&&<span>{t("no disponible")}</span>}</button></>:<><span className="process-legend">{t("CPU del proceso")}</span><button aria-label={t("Escala automática de CPU")} aria-pressed={autoCpu} onClick={()=>setAutoCpu(value=>!value)}>{autoCpu?t("Escala automática"):t("Escala 0–100 %")}</button></>}<small>{t("Flechas para explorar")}</small></div>
    {error && <p className="network-error" role="status">{translatedMessage(error)}</p>}
  </section>
}

export function ResourceMonitor(props:MonitorProps) {
  const [windowMs,setWindowMs]=useState(HISTORY_MS)
  const samples=chartSamples(props.history,props.server.id,props.metrics?.pid,HISTORY_MS)
  const readRate=props.metrics?.pid ? diskRate(samples,'diskReadBytes'):null
  const writeRate=props.metrics?.pid ? diskRate(samples,'diskWrittenBytes'):null
  return <section className="resource-monitor"><div className="section-heading performance-heading"><div><h2>{t("Rendimiento")}</h2><p>{t("Proceso Java · lecturas reales")}</p></div><div className="chart-range" role="group" aria-label={t("Intervalo de las gráficas")}>{[[30_000,'30 s'],[60_000,'1 min'],[HISTORY_MS,'2 min']].map(([value,label])=><button key={value} aria-pressed={windowMs===value} onClick={()=>setWindowMs(Number(value))}>{label}</button>)}</div></div>
    <div className="performance-charts"><ResourceChart {...props} metric="memoryBytes" windowMs={windowMs}/><ResourceChart {...props} metric="cpuPercent" windowMs={windowMs}/></div>
    <section className="heap-diagnostics" aria-label={t("Memoria interna de Java")}><MemoryExport key={props.server.id} id={props.server.id}/><dl className="monitor-details"><div><dt>{t("Heap usado")}</dt><dd>{props.metrics?.heap?bytesLabel(props.metrics.heap.usedBytes):'—'}</dd></div><div><dt>{t("Heap reservado")}</dt><dd>{props.metrics?.heap?bytesLabel(props.metrics.heap.committedBytes):'—'}</dd></div><div><dt>{t("Eventos del recolector")}</dt><dd>{props.metrics?.heap?formatNumber(props.metrics.heap.collections):'—'}</dd></div></dl><details className="performance-help"><summary>{t("Cómo interpretar la memoria")}</summary><p className="field-help">{t("La gráfica mide la RAM residente del proceso, incluidos heap y memoria nativa. Java puede reutilizar heap reservado sin devolverlo inmediatamente a Windows o Linux. El heap usado incluye objetos aún no recogidos; no equivale a memoria retenida después de una recolección.")}</p></details>{props.metrics?.heapError&&<details><summary>{t("Heap no disponible")}</summary><p className="field-help">{translatedMessage(props.metrics.heapError)}</p></details>}{props.metrics?.heap&&<p className="field-help">{t("Contadores HotSpot leídos a las")} {formatTime(props.metrics.heap.sampledAt)}{t(". Son muestras de Java; no se fuerza ninguna recolección.")}</p>}</section>
    <div className="performance-checks"><ServerResponsePanel key={props.server.id} server={props.server}/><TickMonitor key={`${props.server.id}:${props.metrics?.pid}`} server={props.server} metrics={props.metrics}/></div>
    <section className="performance-process"><h3>{t("Proceso y disco")}</h3><dl className="monitor-details"><div><dt>{t("Lectura de disco")}</dt><dd>{readRate==null?'—':`${bytesLabel(readRate)}/s`}</dd></div><div><dt>{t("Escritura de disco")}</dt><dd>{writeRate==null?'—':`${bytesLabel(writeRate)}/s`}</dd></div><div><dt>{t("Heap máximo de Java")}</dt><dd>{bytesLabel(props.server.memoryMaxMb*1024**2)}</dd></div><div><dt>{t("Proceso")}</dt><dd>{props.metrics?.pid??'—'}</dd></div><div><dt>{t("Tiempo de arranque")}</dt><dd>{props.metrics?.startupDurationMs==null?'—':`${formatNumber(props.metrics.startupDurationMs/1000,{minimumFractionDigits:1,maximumFractionDigits:1})} s`}</dd></div></dl><details className="performance-help"><summary>{t("Cómo se mide el arranque")}</summary><p className="field-help">{t("Arranque: desde el lanzamiento de Java hasta que el servidor comunica que está listo. Se mide para el proceso actual; una salida antes de estar listo no produce un valor.")}</p></details></section>
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
    try{await api.requestTps(server.id);setRequested(true)}catch(cause){setWaiting(false);setError(errorText(cause))}
  }
  const pending=waiting||metrics?.tps?.pending
  return <section className="tick-monitor" aria-label={t("TPS del servidor")}><div className="section-heading"><div><h3>{t("Ritmo del servidor")}</h3><p>{server.platform==='paper'?t("Medias de TPS comunicadas por Paper. Objetivo habitual: 20."):t("Este motor no ofrece una fuente de TPS integrada.")}</p></div>{server.platform==='paper'&&<button disabled={server.state!=='running'||!!pending} onClick={()=>void query()}>{pending?t("Consultando…"):t("Consultar TPS")}</button>}</div>
    <dl className="monitor-details">{[[t("Último minuto"),reading?.oneMinute],[t("Últimos 5 minutos"),reading?.fiveMinutes],[t("Últimos 15 minutos"),reading?.fifteenMinutes]].map(([label,value])=><div key={label}><dt>{translatedMessage(label)}</dt><dd className={typeof value==='number'&&value<16?'tick-low':typeof value==='number'&&value<18?'tick-warning':undefined}>{typeof value==='number'?value.toFixed(1):'—'}</dd></div>)}</dl>
    <p className="field-help" role="status">{server.platform!=='paper'?t("TPS no disponibles para este motor."):pending?t("Esperando respuesta del proceso Java…"):reading?t("Consulta recibida a las {0}. La lectura caduca tras un minuto.", formatTime(reading.sampledAt)):server.state!=='running'?t("Inicia el servidor para consultar su ritmo."):requested?t("No hay una lectura reciente. El comando puede no estar disponible en este servidor."):t("Sin lectura: pulsa Consultar TPS para solicitarla.")}</p>
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}
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
    catch(cause){if(active.current)setError(errorText(cause))}
    finally{inFlight.current=false;if(active.current)setBusy(false)}
  }
  return <><div className="section-heading"><h3>{t("Memoria de Java")}</h3><button disabled={busy} onClick={()=>void save()}>{busy?t("Guardando…"):t("Exportar diagnóstico")}</button></div>
    <details className="performance-help"><summary>{t("Qué incluye el diagnóstico")}</summary><p className="field-help">{t("Hasta 30 minutos de muestras recogidas con este servidor seleccionado. Sin logs ni comandos. Guarda el archivo en el equipo que aloja el servidor.")}</p></details>
    {error&&<p className="form-error" role="alert">{translatedMessage(error)}</p>}{saved&&<p className="field-help" role="status">{t("Diagnóstico guardado:")} {saved}</p>}</>
}
