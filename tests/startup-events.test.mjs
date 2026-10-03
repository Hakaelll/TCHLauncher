import {test} from 'node:test'
import assert from 'node:assert/strict'
import {acceptStartupEvent} from '../src/startupEvents.ts'
const event=(sessionId,phase,serverId='one')=>({serverId,sessionId,phase})
test('startup events require a launch and remain monotonic within the same session',()=>{
  const empty={}
  assert.equal(acceptStartupEvent(empty,event('first','ready')),empty)
  let current=acceptStartupEvent(empty,event('first','launching'))
  current=acceptStartupEvent(current,event('first','preparing_spawn'))
  assert.equal(acceptStartupEvent(current,event('first','loading_world')),current)
  assert.equal(acceptStartupEvent(current,event('first','preparing_spawn')),current)
  current=acceptStartupEvent(current,event('first','ready'))
  assert.equal(current.one.phase,'ready')
  assert.equal(acceptStartupEvent(current,event('first','launching')),current)
})
test('a restart rejects later updates from the old session without changing another server',()=>{
  let current=acceptStartupEvent({},event('first','launching'))
  current=acceptStartupEvent(current,event('other','launching','two'))
  current=acceptStartupEvent(current,event('second','launching'))
  assert.equal(current.one.sessionId,'second')
  assert.equal(acceptStartupEvent(current,event('first','ready')),current)
  current=acceptStartupEvent(current,event('second','loading_world'))
  assert.equal(current.one.phase,'loading_world')
  assert.equal(current.two.sessionId,'other')
})
