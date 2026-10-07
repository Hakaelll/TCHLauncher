import {test} from 'node:test'
import assert from 'node:assert/strict'
import {chartSamples,inspectionSamples,stepSample,memoryDomain} from '../src/chartData.ts'

const sample=(time,overrides={})=>({serverId:'one',pid:42,sampledAt:time,memoryBytes:100,cpuPercent:3,heapUsedBytes:null,...overrides})

test('chart windows isolate server and process, retain the boundary, and do not mutate history',()=>{
  const history=[sample(60_000),sample(30_000),sample(29_999),sample(59_000,{serverId:'other'}),sample(61_000,{pid:43}),sample(NaN)]
  const original=[...history]
  assert.deepEqual(chartSamples(history,'one',42,30_000).map(row=>row.sampledAt),[30_000,60_000])
  assert.deepEqual(history,original)
  assert.deepEqual(chartSamples(history,'one',null,120_000),[])
})

test('heap-only inspection excludes unavailable readings but keeps a real zero',()=>{
  const history=[sample(1),sample(2,{heapUsedBytes:0}),sample(3,{heapUsedBytes:NaN}),sample(4,{heapUsedBytes:Infinity}),sample(5,{heapUsedBytes:50})]
  assert.deepEqual(inspectionSamples(history,['heapUsedBytes']).map(row=>row.sampledAt),[2,5])
  assert.equal(inspectionSamples(history,['memoryBytes','heapUsedBytes']).length,5)
})

test('keyboard chart inspection enters at the newest sample and clamps at both ends',()=>{
  const rows=[sample(10),sample(20),sample(30)]
  assert.equal(stepSample(rows,null,'ArrowLeft'),30)
  assert.equal(stepSample(rows,null,'ArrowRight'),30)
  assert.equal(stepSample(rows,20,'ArrowLeft'),10)
  assert.equal(stepSample(rows,10,'ArrowLeft'),10)
  assert.equal(stepSample(rows,30,'ArrowRight'),30)
  assert.equal(stepSample(rows,30,'Home'),10)
  assert.equal(stepSample(rows,10,'End'),30)
  assert.equal(stepSample([],null,'End'),null)
})

test('memory axes include native process overhead and larger measured heap without treating gaps as zero',()=>{
  assert.equal(memoryDomain([sample(1,{memoryBytes:200,heapUsedBytes:300})],100),330)
  assert.equal(memoryDomain([sample(1,{memoryBytes:200,heapUsedBytes:300})],4000),330)
  assert.equal(memoryDomain([sample(1,{memoryBytes:null,heapUsedBytes:NaN})],100),100)
  assert.equal(memoryDomain([],0),1)
})
