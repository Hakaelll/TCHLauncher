import {test} from 'node:test'
import assert from 'node:assert/strict'
import {verifyStartup} from '../src/startupVerification.ts'

function observation(states,timeoutMs=6000) {
  let time=0,index=0
  return {readState:async()=>states[Math.min(index++,states.length-1)],wait:async()=>{time+=1500},now:()=>time,timeoutMs}
}
test('process creation and starting do not establish Minecraft readiness',async()=>{
  assert.deepEqual(await verifyStartup(observation(['starting','starting','running'])),{kind:'ready',state:'running'})
})
test('a stopped or crashed process never becomes a successful upgrade check',async()=>{
  for(const state of ['stopped','stopping','crashed','error']) {
    assert.deepEqual(await verifyStartup(observation(['starting',state,'running'])),{kind:'halted',state})
  }
})
test('slow startup, missing server and failed native observation remain unverified',async()=>{
  assert.deepEqual(await verifyStartup(observation(['starting'])),{kind:'unverified'})
  assert.equal((await verifyStartup(observation([null]))).kind,'unverified')
  const options=observation(['running']);options.readState=async()=>{throw new Error('native unavailable')}
  assert.equal((await verifyStartup(options)).kind,'unverified')
})
test('leaving the panel stops observation without stopping the server',async()=>{
  const controller=new AbortController();controller.abort()
  let reads=0
  const options=observation(['running']);options.signal=controller.signal;options.readState=async()=>{reads++;return 'running'}
  assert.deepEqual(await verifyStartup(options),{kind:'unverified'});assert.equal(reads,0)
})
