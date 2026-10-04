import type { Metrics } from './api'

export const HISTORY_MS = 120_000
export type MetricSample = Pick<Metrics,'serverId' | 'sampledAt' | 'pid' | 'memoryBytes' | 'cpuPercent' | 'diskReadBytes' | 'diskWrittenBytes'> & {heapUsedBytes?:number|null}

export function recordSample(history: MetricSample[],metrics: Metrics): MetricSample[] {
  if (metrics.pid == null || metrics.memoryBytes == null || !Number.isFinite(metrics.sampledAt)) return history
  const last = history.at(-1)
  const sameProcess = last?.serverId === metrics.serverId && last.pid === metrics.pid
  if (sameProcess && metrics.sampledAt <= last.sampledAt) return history
  const retained = sameProcess ? history.filter(sample => sample.sampledAt >= metrics.sampledAt - HISTORY_MS) : []
  return [...retained,{...metrics,heapUsedBytes:metrics.heap?.usedBytes??null}].slice(-65)
}

export function diskRate(history: MetricSample[],key: 'diskReadBytes' | 'diskWrittenBytes'): number | null {
  const current = history.at(-1)
  const previous = history.at(-2)
  if (!current || !previous || current.pid !== previous.pid || current.serverId !== previous.serverId) return null
  const bytes = current[key]
  const before = previous[key]
  const seconds = (current.sampledAt - previous.sampledAt) / 1000
  if (bytes == null || before == null || bytes < before || seconds <= 0 || seconds > 15) return null
  return (bytes - before) / seconds
}

export function seriesSegments(history: MetricSample[],key: 'memoryBytes' | 'cpuPercent' | 'heapUsedBytes'): MetricSample[][] {
  const segments: MetricSample[][] = []
  let current: MetricSample[] = []
  for (const sample of history) {
    const previous = current.at(-1)
    const value = sample[key]
    if (value == null || !Number.isFinite(value) || previous && sample.sampledAt - previous.sampledAt > 15_000) {
      if (current.length) segments.push(current)
      current = []
    }
    if (value != null && Number.isFinite(value)) current.push(sample)
  }
  if (current.length) segments.push(current)
  return segments
}
