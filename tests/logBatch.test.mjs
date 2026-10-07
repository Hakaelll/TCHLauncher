import {test} from 'node:test'
import assert from 'node:assert/strict'
import {createLogBatch} from '../src/logBatch.ts'

function fixture(){
  const callbacks=[],published=[],cancelled=[]
  const batch=createLogBatch(1000,lines=>published.push(lines),callback=>{callbacks.push(callback);return callbacks.length},handle=>cancelled.push(handle))
  return {batch,callbacks,published,cancelled}
}
test('console bursts publish once and retain the same newest 1000 lines in order',()=>{
  const f=fixture()
  for(let i=0;i<20_000;i++)f.batch.push(i)
  assert.equal(f.callbacks.length,1);assert.equal(f.published.length,0)
  f.callbacks[0]()
  assert.deepEqual(f.published[0],Array.from({length:1000},(_,i)=>i+19_000))
  f.batch.push(20_000);f.batch.push(20_001);assert.equal(f.callbacks.length,2)
  f.callbacks[1]();assert.deepEqual(f.published[1],[20_000,20_001])
})
test('reselection and disposal release buffered entries and reject late timer callbacks',()=>{
  const f=fixture();f.batch.push('old server');f.batch.reset();f.batch.push('new server')
  f.callbacks[0]();assert.deepEqual(f.published,[])
  f.callbacks[1]();assert.deepEqual(f.published,[['new server']]);assert.deepEqual(f.cancelled,[1])
  f.batch.push('unmounted');f.batch.dispose();f.callbacks[2]();f.batch.push('later')
  assert.equal(f.callbacks.length,3);assert.deepEqual(f.published,[['new server']])
})
