import {test} from 'node:test'
import assert from 'node:assert/strict'
import {contentWindow} from '../src/contentWindow.ts'
import {listWindow} from '../src/listWindow.ts'
const rows=[
  {title:'Mod 10',status:'error',file:{filename:'ten.jar',projectId:'managed'},check:{descriptor:{id:'primary',ids:['primary','secondary']}}},
  {title:'Mod 2',status:'warning',file:{filename:'two.jar',projectId:null}},
  {title:'Mod 2',status:'observed',file:{filename:'other.jar',projectId:'other'}},
  {title:'Local',status:'unknown',file:{filename:'local.jar',projectId:null}},
]
test('inventory search includes secondary identities and composes state with origin',()=>{
  assert.deepEqual(contentWindow(rows,' SECONDARY ','attention','managed','name').map(r=>r.file.filename),['ten.jar'])
  assert.deepEqual(contentWindow(rows,'','attention','local','name').map(r=>r.file.filename),['two.jar'])
  assert.deepEqual(contentWindow(rows,'','observed','managed','name').map(r=>r.file.filename),['other.jar'])
  assert.equal(contentWindow(rows,'','unknown','managed','name').length,0)
  assert.deepEqual(contentWindow(rows,'   ','all','local','name').map(r=>r.file.filename),['local.jar','two.jar'])
})
test('attention ordering preserves separate equal-name files and native input order',()=>{
  const before=JSON.stringify(rows)
  assert.deepEqual(contentWindow(rows,'','all','all','attention').map(r=>r.file.filename),['ten.jar','two.jar','local.jar','other.jar'])
  assert.deepEqual(contentWindow(rows,'mod','all','all','name').map(r=>r.file.filename),['two.jar','other.jar','ten.jar'])
  assert.equal(JSON.stringify(rows),before)
})

test('installed file pages retain distinct equal names, full search and source counts',()=>{
  const inventory=Array.from({length:49},(_,i)=>({title:'Same name',status:'unknown',file:{filename:`file-${i}.jar`,projectId:i===48?'managed':null}}))
  const before=JSON.stringify(inventory)
  const matching=contentWindow(inventory,'','all','all','name')
  const first=listWindow(matching,0,24),second=listWindow(matching,1,24),last=listWindow(matching,2,24)
  assert.equal(first.total,49)
  assert.equal(first.pages,3)
  assert.equal(first.rows.length,24)
  assert.equal(second.rows.length,24)
  assert.equal(last.rows.length,1)
  assert.deepEqual([...first.rows,...second.rows,...last.rows].map(row=>row.file.filename),inventory.map(row=>row.file.filename))
  assert.deepEqual(listWindow(contentWindow(inventory,'file-48','all','managed','name'),2,24).rows,[inventory[48]])
  assert.equal(JSON.stringify(inventory),before)
})

test('inventory removal and empty filters clamp paging without restoring old page numbers',()=>{
  assert.deepEqual(listWindow(rows,90,24),{rows,total:4,pages:1,page:0})
  assert.deepEqual(listWindow([],90,24),{rows:[],total:0,pages:1,page:0})
  for(const requested of [-1,NaN,Infinity])assert.equal(listWindow(rows,requested,24).page,0)
  const inventory=Array.from({length:49},(_,i)=>i)
  assert.deepEqual(listWindow(inventory,1.9,24).rows,inventory.slice(24,48))
})
