import { test } from 'node:test'
import assert from 'node:assert/strict'
import { recordSample, diskRate, seriesSegments } from '../src/telemetry.ts'

const sample = (overrides = {}) => ({serverId:'one',sampledAt:200_000,pid:42,uptimeSeconds:1,memoryBytes:1024,cpuPercent:2,diskReadBytes:100,diskWrittenBytes:200,sizeBytes:1024,...overrides})

test('a restart or another server starts a new process history', () => {
  const history = [sample()]
  assert.equal(recordSample(history,sample({pid:43,sampledAt:203_000})).length,1)
  assert.equal(recordSample(history,sample({serverId:'two',sampledAt:203_000})).length,1)
})
test('late and duplicate replies cannot reorder a history', () => {
  const history = [sample()]
  assert.equal(recordSample(history,sample({sampledAt:199_000})),history)
  assert.equal(recordSample(history,sample()),history)
})
test('only measured samples in the two-minute window are retained', () => {
  const history = [sample({sampledAt:10_000}),sample({sampledAt:85_000}),sample()]
  assert.deepEqual(recordSample(history,sample({sampledAt:203_000})).map(item => item.sampledAt),[85_000,200_000,203_000])
  assert.equal(recordSample(history,sample({pid:null,memoryBytes:null})),history)
})
test('disk throughput uses bytes and actual elapsed time', () => {
  const history = [sample(),sample({sampledAt:203_000,diskReadBytes:400,diskWrittenBytes:800})]
  assert.equal(diskRate(history,'diskReadBytes'),100)
  assert.equal(diskRate(history,'diskWrittenBytes'),200)
})
test('unknown counters, resets and long sampling gaps are not shown as throughput', () => {
  for (const override of [{diskReadBytes:null},{diskReadBytes:20},{sampledAt:200_000},{sampledAt:220_000},{pid:50}]) {
    assert.equal(diskRate([sample(),sample({sampledAt:203_000,...override})],'diskReadBytes'),null)
  }
})
test('unavailable CPU samples and gaps do not draw invented connecting lines', () => {
  const history = [sample(),sample({sampledAt:203_000,cpuPercent:null}),sample({sampledAt:206_000}),sample({sampledAt:240_000})]
  assert.deepEqual(seriesSegments(history,'cpuPercent').map(segment => segment.length),[1,1,1])
  assert.deepEqual(seriesSegments(history,'memoryBytes').map(segment => segment.length),[3,1])
})
