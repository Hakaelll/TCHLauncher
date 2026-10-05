import {test} from 'node:test'
import assert from 'node:assert/strict'
import {saveState} from '../src/saveState.ts'

const entry=(event,at=0)=>({serverId:'real',event,message:event,line:event,level:event==='save_failed'?'error':'system',timestamp:`2026-10-05T12:00:${String(at).padStart(2,'0')}Z`})

test('starting and per-dimension output never become a completed whole save',()=>{
  const value=saveState([entry('save_started'),entry('save_dimension',1)])
  assert.equal(value.phase,'saving');assert.equal(value.lastConfirmed,null)
  assert.equal(saveState([entry('save_dimension')]).phase,'unknown')
})

test('save completion is retained through a failed later attempt with visible evidence',()=>{
  const complete=entry('save',1),failure=entry('save_failed',3)
  const value=saveState([entry('save_started'),complete,entry('save_started',2),failure])
  assert.equal(value.phase,'failed');assert.equal(value.latest,failure);assert.equal(value.lastConfirmed,complete)
  assert.equal(saveState([failure,entry('save_started',4),entry('save',5)]).phase,'confirmed')
})

test('exit never implies saving succeeded and a new startup clears old-session evidence',()=>{
  assert.equal(saveState([entry('save_started'),entry('exit')]).phase,'unconfirmed')
  assert.equal(saveState([entry('save_started'),entry('crash')]).lastConfirmed,null)
  for(const boundary of ['startup','world_load']) {
    assert.deepEqual(saveState([entry('save_failed'),entry('save'),entry(boundary)]),{phase:'unknown',lastConfirmed:null,latest:null})
  }
  assert.equal(saveState([entry('save_failed'),entry('crash')]).phase,'failed')
  assert.equal(saveState([entry('save_started')],'stopped').phase,'unconfirmed')
  assert.equal(saveState([entry('save_started')],'running').phase,'saving')
})
