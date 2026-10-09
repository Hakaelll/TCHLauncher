import {test} from 'node:test'
import assert from 'node:assert/strict'
import {contentOperation} from '../src/contentOperation.ts'

const deferred=()=>{let resolve;const promise=new Promise(done=>{resolve=done});return {promise,resolve}}

test('concurrent content actions perform one write and retain the busy lock through follow-up reading',async()=>{
  const owner={active:true,working:false},busy=[],write=deferred(),read=deferred()
  let writes=0,reads=0
  const options={owner,isCurrent:()=>owner.active,setBusy:value=>busy.push(value)}
  const first=contentOperation({...options,work:async()=>{writes++;await write.promise;reads++;await read.promise}})
  assert.equal(owner.working,true)
  await contentOperation({...options,work:async()=>{writes++}})
  assert.equal(writes,1)
  write.resolve();await Promise.resolve()
  assert.equal(reads,1)
  assert.equal(owner.working,true)
  assert.deepEqual(busy,[true])
  read.resolve();await first
  assert.equal(owner.working,false)
  assert.deepEqual(busy,[true,false])
})

test('failed actions release the content lock and preserve the original rejection for their handler',async()=>{
  const owner={active:true,working:false},busy=[],failure=new Error('write denied')
  const options={owner,isCurrent:()=>owner.active,setBusy:value=>busy.push(value)}
  await assert.rejects(contentOperation({...options,work:async()=>{throw failure}}),error=>error===failure)
  assert.equal(owner.working,false)
  let writes=0
  await contentOperation({...options,work:async()=>{writes++}})
  assert.equal(writes,1)
  assert.deepEqual(busy,[true,false,true,false])
})

test('old content completions cannot clear a replacement owner lock or start an obsolete action',async()=>{
  const old={active:true,working:false},fresh={active:true,working:false},oldBusy=[],freshBusy=[]
  let current=old,writes=0
  const oldRead=deferred(),newRead=deferred()
  const previous=contentOperation({owner:old,isCurrent:()=>current===old&&old.active,setBusy:value=>oldBusy.push(value),work:()=>oldRead.promise})
  current=fresh
  const next=contentOperation({owner:fresh,isCurrent:()=>current===fresh&&fresh.active,setBusy:value=>freshBusy.push(value),work:()=>newRead.promise})
  await contentOperation({owner:old,isCurrent:()=>current===old&&old.active,setBusy:value=>oldBusy.push(value),work:async()=>{writes++}})
  oldRead.resolve();await previous
  assert.equal(writes,0)
  assert.equal(old.working,false)
  assert.equal(fresh.working,true)
  assert.deepEqual(oldBusy,[true])
  assert.deepEqual(freshBusy,[true])
  newRead.resolve();await next
  assert.deepEqual(freshBusy,[true,false])
  fresh.active=false
  await contentOperation({owner:fresh,isCurrent:()=>fresh.active,setBusy:value=>freshBusy.push(value),work:async()=>{writes++}})
  assert.equal(writes,0)
})
