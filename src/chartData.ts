import type {MetricSample} from './telemetry'

export type ChartMetric='memoryBytes'|'heapUsedBytes'|'cpuPercent'
export const measured=(value:unknown):value is number=>typeof value==='number'&&Number.isFinite(value)&&value>=0

export function chartSamples(history:MetricSample[],serverId:string,pid:number|null|undefined,windowMs:number):MetricSample[] {
  if(pid==null)return []
  const matching=history.filter(sample=>sample.serverId===serverId&&sample.pid===pid&&Number.isFinite(sample.sampledAt))
  const latest=matching.reduce((latest,sample)=>Math.max(latest,sample.sampledAt),-Infinity)
  return matching.filter(sample=>sample.sampledAt>=latest-windowMs).sort((a,b)=>a.sampledAt-b.sampledAt)
}

export function inspectionSamples(samples:MetricSample[],keys:ChartMetric[]):MetricSample[] {
  return samples.filter(sample=>keys.some(key=>measured(sample[key])))
}

export function stepSample(samples:MetricSample[],time:number|null,key:'ArrowLeft'|'ArrowRight'|'Home'|'End'):number|null {
  if(!samples.length)return null
  const index=samples.findIndex(sample=>sample.sampledAt===time)
  const target=key==='Home'?0:key==='End'?samples.length-1:index<0?samples.length-1:key==='ArrowLeft'?Math.max(0,index-1):Math.min(samples.length-1,index+1)
  return samples[target].sampledAt
}

/** Fit measured memory to the viewport; Xmx is a fallback, not a resident RAM limit. */
export function memoryDomain(samples:MetricSample[],configuredBytes:number):number {
  let peak=0
  for(const sample of samples)peak=Math.max(peak,measured(sample.memoryBytes)?sample.memoryBytes:0,measured(sample.heapUsedBytes)?sample.heapUsedBytes:0)
  return peak>0?peak*1.1:Math.max(configuredBytes,1)
}
