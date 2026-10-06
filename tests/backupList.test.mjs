import {test} from 'node:test'
import assert from 'node:assert/strict'
import {createBackupList} from '../src/backupList.ts'
const deferred=()=>{let resolve,reject;const promise=new Promise((yes,no)=>{resolve=yes;reject=no});return {promise,resolve,reject}}
const copy=id=>({id,serverId:'a',createdAt:'2026-10-05',sizeBytes:1,path:id})

test('backup read failure stays unavailable and retry distinguishes a valid empty list',async()=>{
  const replies=[Promise.reject(new Error('SQLite busy')),Promise.resolve([])]
  let state
  const list=createBackupList(()=>replies.shift(),value=>state=value)
  list.select('a')
  await assert.rejects(list.refresh(),/SQLite busy/)
  assert.equal(state.loaded,false);assert.equal(state.loading,false);assert.match(state.error,/SQLite busy/)
  await list.refresh();assert.equal(state.loaded,true);assert.deepEqual(state.copies,[]);assert.equal(state.error,null)
})
test('newest read owns the list across overlapping requests and server reselection',async()=>{
  const pending=[];let state
  const list=createBackupList(()=>{const task=deferred();pending.push(task);return task.promise},value=>state=value)
  list.select('a');const old=list.refresh();const latest=list.refresh()
  pending[1].resolve([copy('new')]);await latest
  pending[0].resolve([copy('old')]);await old;assert.equal(state.copies[0].id,'new')
  const beforeSwitch=list.refresh();list.select('b');list.select('a');const afterSwitch=list.refresh()
  pending[2].reject(new Error('old failure'));await beforeSwitch;assert.equal(state.error,null)
  pending[3].resolve([copy('returned')]);await afterSwitch;assert.equal(state.copies[0].id,'returned')
  const closing=list.refresh();list.dispose();const before=state
  pending[4].resolve([copy('closed')]);await closing;assert.equal(state,before)
})
test('failed refresh preserves last known copies without presenting them as current',async()=>{
  let state;let fail=false
  const list=createBackupList(async()=>{if(fail)throw new Error('unavailable');return [copy('known')]},value=>state=value)
  list.select('a');await list.refresh();fail=true
  await assert.rejects(list.refresh(),/unavailable/)
  assert.equal(state.loaded,true);assert.equal(state.copies[0].id,'known');assert.equal(state.error,'Error: unavailable')
  await list.refresh('b');assert.equal(state.copies[0].id,'known')
})
