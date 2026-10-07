import {test} from 'node:test'
import assert from 'node:assert/strict'
import {startVisibilityPolling} from '../src/visibilityPolling.ts'
const settle=()=>new Promise(resolve=>setImmediate(resolve))
function fixture(){
  let visible=true,listener=null,next=0
  const timers=new Map()
  return {timers,env:{visible:()=>visible,subscribe:callback=>{listener=callback;return()=>{listener=null}},schedule:(callback,delay)=>{timers.set(++next,{callback,delay});return next},cancel:id=>timers.delete(id)},
    change(value){visible=value;listener?.()},fire(){const [id,item]=timers.entries().next().value;timers.delete(id);item.callback()},subscribed:()=>listener!==null}
}
test('hidden metrics slow down and foreground return immediately refreshes without overlap',async()=>{
  const f=fixture();let resolve,reads=0
  const stop=startVisibilityPolling(()=>{reads++;return new Promise(r=>{resolve=r})},3000,{hiddenMs:15000,immediate:true},f.env)
  assert.equal(reads,1);f.change(false);f.change(true);assert.equal(reads,1)
  resolve();await settle();assert.equal([...f.timers.values()][0].delay,3000)
  f.change(false);assert.equal([...f.timers.values()][0].delay,15000)
  f.change(true);assert.equal(reads,2);assert.equal(f.timers.size,0)
  stop();resolve();await settle();assert.equal(f.timers.size,0);assert.equal(f.subscribed(),false)
})
test('hidden player polling pauses, resumes once and rejects late callbacks after disposal',async()=>{
  const f=fixture();let reads=0
  const stop=startVisibilityPolling(async()=>{reads++},3000,{},f.env)
  const late=[...f.timers.values()][0].callback
  f.change(false);assert.equal(f.timers.size,0);assert.equal(reads,0)
  f.change(true);await settle();assert.equal(reads,1);assert.equal(f.timers.size,1)
  f.fire();await settle();assert.equal(reads,2)
  stop();late();await settle();assert.equal(reads,2);assert.equal(f.timers.size,0)
})
