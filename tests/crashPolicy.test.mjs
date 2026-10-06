import {test} from 'node:test'
import assert from 'node:assert/strict'
import {crashPolicyErrors,defaultCrashPolicy,reconcileCrashDraft,sameCrashPolicy} from '../src/crashPolicy.ts'

test('restart policies reject fractional and nonfinite values at native boundaries',()=>{
  for(const delaySeconds of [5,3600])for(const maxAttempts of [1,10]){
    assert.deepEqual(crashPolicyErrors({enabled:true,delaySeconds,maxAttempts}),{delaySeconds:null,maxAttempts:null})
  }
  for(const delaySeconds of [0,4,3601,5.5,NaN,Infinity])assert.ok(crashPolicyErrors({...defaultCrashPolicy,delaySeconds}).delaySeconds)
  for(const maxAttempts of [0,11,1.5,NaN,Infinity])assert.ok(crashPolicyErrors({...defaultCrashPolicy,maxAttempts}).maxAttempts)
  assert.ok(crashPolicyErrors({enabled:false,delaySeconds:0,maxAttempts:0}).delaySeconds)
})
test('native refresh reconciles a clean policy but preserves local unsaved edits',()=>{
  const previous={...defaultCrashPolicy}
  const observed={enabled:true,delaySeconds:120,maxAttempts:5}
  assert.deepEqual(reconcileCrashDraft(previous,previous,observed),observed)
  assert.notEqual(reconcileCrashDraft(previous,previous,observed),observed)
  for(const draft of [{...previous,enabled:true},{...previous,delaySeconds:60},{...previous,maxAttempts:7}]){
    assert.equal(reconcileCrashDraft(draft,previous,observed),draft)
    assert.equal(sameCrashPolicy(draft,previous),false)
  }
  assert.deepEqual(reconcileCrashDraft(previous,null,observed),observed)
  assert.deepEqual(previous,defaultCrashPolicy)
  assert.deepEqual(observed,{enabled:true,delaySeconds:120,maxAttempts:5})
})
