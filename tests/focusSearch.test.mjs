import {test} from 'node:test'
import assert from 'node:assert/strict'
import {focusSearch} from '../src/focusSearch.ts'
const input=({disabled=false,hidden=false,rects=1}={})=>({
  matches:()=>disabled,closest:()=>hidden?{}:null,getClientRects:()=>Array(rects),
  focused:false,selected:false,focus(){this.focused=true},select(){this.selected=true},
})
test('current-view search skips hidden, inert, disabled and unrendered inputs',()=>{
  const candidates=[input({hidden:true}),input({disabled:true}),input({rects:0}),input()]
  assert.equal(focusSearch({querySelectorAll:()=>candidates}),true)
  assert.equal(candidates[3].focused,true);assert.equal(candidates[3].selected,true)
  assert.equal(candidates.slice(0,3).some(value=>value.focused||value.selected),false)
})
test('views without an eligible search retain the native find action',()=>{
  assert.equal(focusSearch(null),false)
  assert.equal(focusSearch({querySelectorAll:()=>[]}),false)
  assert.equal(focusSearch({querySelectorAll:()=>[input({hidden:true}),input({rects:0})]}),false)
})
