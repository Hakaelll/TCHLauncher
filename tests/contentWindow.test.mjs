import {test} from 'node:test'
import assert from 'node:assert/strict'
import {contentWindow} from '../src/contentWindow.ts'
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
