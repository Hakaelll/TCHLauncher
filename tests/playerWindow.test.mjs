import {test} from 'node:test'
import assert from 'node:assert/strict'
import {playerWindow} from '../src/playerWindow.ts'

test('search and paging keep separate UUIDs for the same Minecraft name',()=>{
  const rows=Array.from({length:125},(_,i)=>({name:i<2?'Alex':`Player_${i}`,uuid:`uuid-${i}`}))
  const before=JSON.stringify(rows)
  const label=row=>`${row.name} ${row.uuid}`
  assert.equal(playerWindow(rows,'',label,0).rows.length,50)
  assert.equal(playerWindow(rows,'',label,2).rows.length,25)
  assert.deepEqual(playerWindow(rows,'  ALEX ',label,9).rows,rows.slice(0,2))
  assert.equal(playerWindow(rows,'uuid-124',label,5).rows[0].uuid,'uuid-124')
  assert.equal(JSON.stringify(rows),before)
})
test('shrinking lists and empty searches clamp pages without inventing players',()=>{
  const label=row=>row
  assert.equal(playerWindow(['Alex'],'',label,20).page,0)
  assert.deepEqual(playerWindow(['Alex'],'nobody',label,20).rows,[])
  assert.equal(playerWindow([], '',label,Infinity).total,0)
  assert.equal(playerWindow(['Alex'],'',label,-1).page,0)
})
