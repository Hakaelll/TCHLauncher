import {test} from 'node:test'
import assert from 'node:assert/strict'
import {operatorDraft,operatorDirty,operatorConflict,reconcileOperator} from '../src/operatorDraft.ts'
const original={level:4,bypassesPlayerLimit:false}
test('operator polling preserves unsaved changes and flags changed native permission values',()=>{
  let state=operatorDraft('a:uuid',original)
  state={...state,draft:{level:2,bypassesPlayerLimit:true}}
  assert.equal(reconcileOperator(state,'a:uuid',{...original}),state)
  assert.equal(operatorConflict(state,'a:uuid',original),false)
  const external={level:3,bypassesPlayerLimit:false}
  assert.equal(reconcileOperator(state,'a:uuid',external),state)
  assert.equal(operatorConflict(state,'a:uuid',external),true)
  assert.deepEqual(state.baseline,original)
  state=operatorDraft('a:uuid',external);assert.equal(operatorDirty(state),false);assert.equal(operatorConflict(state,'a:uuid',external),false)
})
test('clean drafts follow native changes and another UUID never inherits a permission draft',()=>{
  const clean=operatorDraft('a:uuid',original),changed={level:1,bypassesPlayerLimit:true}
  assert.deepEqual(reconcileOperator(clean,'a:uuid',changed).draft,changed)
  const dirty={...clean,draft:changed}
  assert.equal(operatorConflict(dirty,'b:uuid',original),true)
  const other=reconcileOperator(dirty,'b:uuid',original)
  assert.equal(other.identity,'b:uuid');assert.deepEqual(other.draft,original)
  original.level=3;assert.equal(clean.baseline.level,4);original.level=4
})
