import {test} from 'node:test'
import assert from 'node:assert/strict'
import {acknowledgedAction} from '../src/acknowledgedAction.ts'
test('acknowledged mutation remains completed after a failed read and is never repeated',async()=>{
  let writes=0,reads=0
  const result=await acknowledgedAction(async()=>{writes++},async()=>{reads++;throw new Error('database busy')})
  assert.equal(writes,1);assert.equal(reads,1);assert.equal(result.refreshError,'database busy')
})
test('mutation failure rejects before any follow-up read',async()=>{
  let reads=0
  await assert.rejects(acknowledgedAction(async()=>{throw new Error('write denied')},async()=>{reads++}),/write denied/)
  assert.equal(reads,0)
  assert.deepEqual(await acknowledgedAction(async()=>{},async()=>{}),{value:undefined,refreshError:null})
})

test('native update warnings survive a failed refresh and recovery reads do not repeat the update',async()=>{
  const warning='Backup retention could not finish'
  let updates=0,reads=0
  const refresh=async()=>{if(++reads===1)throw new Error('read unavailable')}
  const result=await acknowledgedAction(async()=>{updates++;return warning},refresh)
  assert.deepEqual(result,{value:warning,refreshError:'read unavailable'})
  await refresh()
  assert.equal(updates,1)
  assert.equal(reads,2)
  assert.deepEqual(await acknowledgedAction(async()=>warning,refresh),{value:warning,refreshError:null})
})
