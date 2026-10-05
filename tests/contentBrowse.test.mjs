import {test} from 'node:test'
import assert from 'node:assert/strict'
import {checkPage} from '../src/contentBrowse.ts'

test('compatibility checks are bounded and stop scheduling after leaving a page',async()=>{
  let active=true,running=0,peak=0
  const started=[],completed=[],release=[]
  const work=checkPage(Array.from({length:20},(_,i)=>i),async item=>{
    started.push(item);running++;peak=Math.max(peak,running)
    await new Promise(resolve=>release.push(resolve))
    completed.push(item);running--
  },()=>active)
  assert.equal(started.length,4)
  active=false;release.forEach(resolve=>resolve());await work
  assert.equal(peak,4);assert.equal(completed.length,4);assert.equal(started.length,4)
})

test('each result is checked once as worker slots become free',async()=>{
  const checked=[]
  await checkPage([1,2,3,4,5,6],async item=>{await Promise.resolve();checked.push(item)},()=>true)
  assert.deepEqual(checked.sort(),[1,2,3,4,5,6])
})
