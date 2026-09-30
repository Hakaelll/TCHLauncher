import type { Metrics, Server } from './api'
import { diskRate, HISTORY_MS, seriesSegments, type MetricSample } from './telemetry'

const bytesLabel = (bytes: number) => bytes >= 1024 ** 3 ? `${(bytes / 1024 ** 3).toFixed(1)} GB` : bytes >= 1024 ** 2 ? `${(bytes / 1024 ** 2).toFixed(1)} MB` : `${(bytes / 1024).toFixed(1)} KB`

type MonitorProps={server:Server;metrics:Metrics|null;history:MetricSample[];error:string|null}
function ResourceChart({server,metrics,history,error,metric}:MonitorProps & {metric:'memoryBytes'|'cpuPercent'}) {
  const samples = history.filter(sample => sample.pid === metrics?.pid)
  const peakMemory = Math.max(0,...samples.map(sample => sample.memoryBytes ?? 0))
  const peakCpu = Math.max(0,...samples.map(sample => sample.cpuPercent ?? 0))
  const hasPeak = samples.some(sample => sample[metric] != null)
  const maximum = metric === 'memoryBytes' ? Math.max(server.memoryMaxMb * 1024 ** 2,peakMemory * 1.1) : 100
  const latestTime = samples.at(-1)?.sampledAt ?? 0
  const x = (sample: MetricSample) => 12 + (sample.sampledAt - latestTime + HISTORY_MS) / HISTORY_MS * 476
  const y = (sample: MetricSample) => 142 - Math.min(1,(sample[metric] ?? 0) / maximum) * 128
  const segments = seriesSegments(samples,metric)
  const format = metric === 'memoryBytes' ? bytesLabel : (value: number) => `${value.toFixed(1)} %`
  const latest = metrics?.[metric]
  const current = latest == null ? '—' : format(latest)

  return <section className={`resource-chart ${metric === 'memoryBytes' ? 'memory' : 'cpu'}`} aria-label={metric === 'memoryBytes' ? 'Memoria del proceso Java' : 'CPU del proceso Java'}>
    <h3>{metric === 'memoryBytes' ? 'Memoria' : 'CPU'}</h3>
    <div className="monitor-reading"><strong>{current}</strong><span>{metric === 'memoryBytes' ? 'RAM del proceso' : 'CPU del proceso'}</span><small>Pico {hasPeak ? format(metric === 'memoryBytes' ? peakMemory : peakCpu) : '—'}</small></div>
    <div className="monitor-chart">
      <div className="chart-scale"><span>{format(maximum)}</span><span>0</span></div>
      <svg viewBox="0 0 500 154" preserveAspectRatio="none" role="img" aria-label={`${metric === 'memoryBytes' ? 'Memoria' : 'CPU'} durante los últimos dos minutos. Valor actual: ${current}.`}>
        {[14,78,142].map(row => <line key={row} x1="0" y1={row} x2="500" y2={row} className="chart-grid"/>)}
        {segments.map((segment,index) => <polyline key={index} className="chart-line" points={segment.map(sample => `${x(sample).toFixed(1)},${y(sample).toFixed(1)}`).join(' ')}/>)}
        {segments.at(-1)?.at(-1) && <circle className="chart-point" cx={x(segments.at(-1)!.at(-1)!)} cy={y(segments.at(-1)!.at(-1)!)} r="3"/>}
      </svg>
      {!samples.length && <p className="chart-empty">{error ? 'No se pudo actualizar el rendimiento.' : metrics?.pid ? 'Recogiendo muestras…' : 'Inicia el servidor para ver su rendimiento.'}</p>}
    </div>
    <div className="chart-times"><span>Hace 2 min</span><span>Ahora</span></div>
    {error && <p className="network-error" role="status">{error}</p>}
  </section>
}

export function ResourceMonitor(props:MonitorProps) {
  const samples=props.history.filter(sample=>sample.pid===props.metrics?.pid)
  const readRate=props.metrics?.pid ? diskRate(samples,'diskReadBytes'):null
  const writeRate=props.metrics?.pid ? diskRate(samples,'diskWrittenBytes'):null
  return <section className="resource-monitor"><div className="section-heading"><div><h2>Rendimiento</h2><p>Proceso Java · últimos dos minutos</p></div></div>
    <div className="performance-charts"><ResourceChart {...props} metric="memoryBytes"/><ResourceChart {...props} metric="cpuPercent"/></div>
    <dl className="monitor-details"><div><dt>Lectura de disco</dt><dd>{readRate==null?'—':`${bytesLabel(readRate)}/s`}</dd></div><div><dt>Escritura de disco</dt><dd>{writeRate==null?'—':`${bytesLabel(writeRate)}/s`}</dd></div><div><dt>Heap máximo de Java</dt><dd>{bytesLabel(props.server.memoryMaxMb*1024**2)}</dd></div><div><dt>Proceso</dt><dd>{props.metrics?.pid??'—'}</dd></div></dl>
  </section>
}
