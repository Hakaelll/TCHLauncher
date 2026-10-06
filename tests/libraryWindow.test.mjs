import {test} from 'node:test'
import assert from 'node:assert/strict'
import {libraryWindow} from '../src/libraryWindow.ts'
const rows=[
  {id:'a',name:'Mundo 10',platform:'fabric',version:'1.21.11',state:'running',pinned:false,createdAt:'2026-01-02'},
  {id:'b',name:'Mundo 2',platform:'paper',version:'1.20.4',state:'crashed',pinned:true,createdAt:'2026-01-03'},
  {id:'c',name:'Mundo 2',platform:'neoforge',version:'1.20.1',state:'starting',pinned:false,createdAt:'invalid'},
]
test('library filters combine real status and trimmed search without merging names',()=>{
  assert.deepEqual(libraryWindow(rows,'  MUNDO 2 ','all','existing').map(s=>s.id),['b','c'])
  assert.deepEqual(libraryWindow(rows,'','running','existing').map(s=>s.id),['a'])
  assert.deepEqual(libraryWindow(rows,'','attention','existing').map(s=>s.id),['b'])
  assert.equal(libraryWindow(rows,'fabric','pinned','existing').length,0)
  assert.equal(libraryWindow(rows,'1.20.1','all','existing')[0].id,'c')
})
test('library sorting is stable, numeric and does not mutate native ordering',()=>{
  const before=JSON.stringify(rows)
  assert.deepEqual(libraryWindow(rows,'','all','name').map(s=>s.id),['b','c','a'])
  assert.deepEqual(libraryWindow(rows,'','all','recent').map(s=>s.id),['b','a','c'])
  assert.deepEqual(libraryWindow(rows,'','all','existing').map(s=>s.id),['a','b','c'])
  assert.equal(JSON.stringify(rows),before)
})
