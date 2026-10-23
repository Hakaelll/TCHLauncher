import {test} from 'node:test'
import assert from 'node:assert/strict'
import {createServerRead} from '../src/serverRead.ts'
import {acknowledgedAction} from '../src/acknowledgedAction.ts'
const deferred=()=>{let resolve,reject;const promise=new Promise((yes,no)=>{resolve=yes;reject=no});return {promise,resolve,reject}}

test('nullable crash reports distinguish an unread inventory, absent history and failed observation',async()=>{
  const pending=[];let state
  const reader=createServerRead(()=>{const request=deferred();pending.push(request);return request.promise},value=>state=value)
  reader.select('paper')
  assert.equal(state.data,null)
  const first=reader.refresh();assert.equal(state.loading,true)
  pending[0].resolve({report:null});await first
  assert.deepEqual(state.data,{report:null});assert.equal(state.loading,false)
  const retry=reader.refresh();pending[1].reject(new Error('report locked'))
  await assert.rejects(retry,/report locked/)
  assert.deepEqual(state.data,{report:null});assert.equal(state.error,'report locked')
  const report={serverId:'paper',sessionId:'crashed-session',diagnoses:[{kind:'bind'}]}
  const recovered=reader.refresh();pending[2].resolve({report});await recovered
  assert.equal(state.error,null);assert.equal(state.data.report,report)
})

test('failed reads retain known data and retry distinguishes unavailable from valid empty',async()=>{
  const pending=[];let state
  const reader=createServerRead(()=>{const task=deferred();pending.push(task);return task.promise},value=>state=value)
  reader.select('a');const first=reader.refresh();pending[0].reject(new Error('busy'))
  await assert.rejects(first,/busy/);assert.equal(state.data,null);assert.equal(state.loading,false)
  const retry=reader.refresh();assert.equal(state.error,'busy');pending[1].resolve([]);await retry
  assert.deepEqual(state.data,[]);assert.equal(state.error,null)
  const failed=reader.refresh();pending[2].reject(new Error('offline'));await assert.rejects(failed)
  assert.deepEqual(state.data,[]);assert.equal(state.error,'offline')
})

test('polls, manual retries and A-B-A selection only accept their newest read',async()=>{
  const pending=[];let state
  const reader=createServerRead(()=>{const task=deferred();pending.push(task);return task.promise},value=>state=value)
  reader.select('a');const poll=reader.refresh();const manual=reader.refresh()
  pending[1].resolve('fresh');await manual;pending[0].resolve('old');await poll;assert.equal(state.data,'fresh')
  const old=reader.refresh();reader.select('b');reader.select('a');const next=reader.refresh()
  pending[3].resolve('reselected');await next;pending[2].reject(new Error('stale'));await old
  assert.equal(state.data,'reselected');assert.equal(state.error,null)
  const count=pending.length;await reader.refresh('b');assert.equal(pending.length,count)
  const disposed=reader.refresh();reader.dispose();pending[4].resolve('late');await disposed
  assert.equal(state.data,'reselected')
})

test('native acknowledgement invalidates an older read and survives failed synchronization',async()=>{
  const pending=[];let state
  const reader=createServerRead(()=>{const task=deferred();pending.push(task);return task.promise},value=>state=value)
  reader.select('startup-servers')
  const oldRead=reader.refresh()
  assert.equal(reader.acknowledge('startup-servers',[{id:'one',enabled:true}]),true)
  pending[0].resolve([{id:'one',enabled:false}]);await oldRead
  assert.deepEqual(state.data,[{id:'one',enabled:true}])
  const followUp=reader.refresh();pending[1].reject(new Error('read failed'))
  await assert.rejects(followUp,/read failed/)
  assert.deepEqual(state.data,[{id:'one',enabled:true}]);assert.equal(state.error,'read failed')
  const retry=reader.refresh();pending[2].resolve([{id:'one',enabled:true,lastResult:'Started'}]);await retry
  assert.equal(state.error,null);assert.equal(state.data[0].lastResult,'Started')
  let sent=0,read=0
  const result=await acknowledgedAction(async()=>{sent++;return true},async success=>{
    assert.equal(success,true);read++;throw new Error('players unavailable')
  })
  assert.deepEqual(result,{value:true,refreshError:'players unavailable'})
  assert.equal(sent,1);assert.equal(read,1)
  const readRetry=await acknowledgedAction(async()=>{read++;return []},async()=>{})
  assert.deepEqual(readRetry,{value:[],refreshError:null});assert.equal(sent,1)

})
test('acknowledgements cannot populate a different selection or a disposed panel',()=>{
  let state
  const reader=createServerRead(async()=>null,value=>state=value)
  reader.select('a');assert.equal(reader.acknowledge('b','other server'),false)
  assert.equal(state.data,null)
  reader.select('b');assert.equal(reader.acknowledge('a','old save'),false)
  reader.dispose();assert.equal(reader.acknowledge('b','late save'),false)
  assert.equal(state.data,null)
})
