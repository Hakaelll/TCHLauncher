import {test} from 'node:test'
import assert from 'node:assert/strict'
import {connectedIdentity,isOfflineUuid} from '../src/playerIdentity.ts'

const local='6f9fc5b5-88af-3f15-ada4-df541a7d348e'
const account='7cdcf761-3838-4071-8f63-95c57f238877'
const records=[{name:'yagofdezz',uuid:local},{name:'yagofdezz',uuid:account}]

test('authenticated connections do not use the first offline history entry',()=>{
  assert.equal(connectedIdentity('yagofdezz',records,true),account)
  assert.equal(connectedIdentity('Yagofdezz',[...records].reverse(),true),account)
  assert.equal(connectedIdentity('yagofdezz',[records[0]],true),null)
})
test('ambiguous proxy/offline identities remain unresolved rather than borrowing a skin',()=>{
  assert.equal(connectedIdentity('yagofdezz',records,false),null)
  assert.equal(connectedIdentity('yagofdezz',[records[0]],false),local)
  assert.equal(connectedIdentity('other',records,true),null)
})
test('repeated identities collapse while malformed UUIDs never become profile requests',()=>{
  assert.equal(connectedIdentity('yagofdezz',[records[1],records[1]],true),account)
  assert.equal(connectedIdentity('yagofdezz',[{name:'yagofdezz',uuid:'invalid'}],true),null)
  assert.equal(isOfflineUuid(local),true)
  assert.equal(isOfflineUuid(account),false)
})
