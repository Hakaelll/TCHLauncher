import {test} from 'node:test'
import assert from 'node:assert/strict'
import {createPropertiesDraft} from '../src/propertiesDraft.ts'
const deferred=()=>{let resolve,reject;const promise=new Promise((yes,no)=>{resolve=yes;reject=no});return {promise,resolve,reject}}

test('failed initial properties reads stay unavailable until a successful retry',async()=>{
  let state,fail=true
  const owner=createPropertiesDraft(async()=>{if(fail)throw new Error('read denied');return {}},value=>state=value)
  owner.select('a');await assert.rejects(owner.refresh(),/read denied/)
  assert.equal(state.loaded,false);assert.equal(state.loading,false)
  owner.edit({motd:'unsafe'});assert.deepEqual(state.values,{})
  fail=false;await owner.refresh();assert.equal(state.loaded,true);assert.deepEqual(state.values,{})
})
test('an unsaved edit invalidates pending reads and background refresh preserves the draft',async()=>{
  let state;const pending=[]
  const owner=createPropertiesDraft(()=>{const request=deferred();pending.push(request);return request.promise},value=>state=value)
  owner.select('a');const first=owner.refresh();pending[0].resolve({motd:'native'});await first
  const older=owner.refresh();const draft={motd:'local'};owner.edit(draft);draft.motd='outside'
  pending[1].resolve({motd:'older'});await older;await owner.refresh()
  assert.equal(state.values.motd,'local');assert.equal(state.dirty,true);assert.equal(pending.length,2)
  assert.equal(state.loading,false)
})
test('write acknowledgment invalidates old replies and explicit restore replacement drops obsolete data',async()=>{
  let state;const pending=[]
  const owner=createPropertiesDraft(()=>{const request=deferred();pending.push(request);return request.promise},value=>state=value)
  owner.select('a');const old=owner.refresh();owner.acknowledge('a',{motd:'saved'})
  pending[0].reject(new Error('old failure'));await old;assert.equal(state.error,null);assert.equal(state.values.motd,'saved')
  owner.edit({motd:'unsaved'});const restore=owner.refresh('a',true)
  assert.equal(state.loaded,false);assert.equal(state.dirty,false);assert.deepEqual(state.values,{})
  pending[1].resolve({motd:'restored'});await restore;assert.equal(state.values.motd,'restored')
})
test('server reselection, superseded responses and disposal isolate properties',async()=>{
  let state;const pending=[]
  const owner=createPropertiesDraft(()=>{const request=deferred();pending.push(request);return request.promise},value=>state=value)
  owner.select('a');const old=owner.refresh();owner.select('b');owner.select('a');const fresh=owner.refresh()
  pending[1].resolve({motd:'fresh'});await fresh;pending[0].resolve({motd:'old'});await old
  assert.equal(state.values.motd,'fresh');owner.acknowledge('b',{motd:'foreign'});assert.equal(state.values.motd,'fresh')
  const last=owner.refresh();owner.dispose();const before=state;pending[2].resolve({motd:'closed'});await last;assert.equal(state,before)
})
