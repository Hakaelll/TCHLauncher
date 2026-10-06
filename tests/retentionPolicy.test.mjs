import {test} from 'node:test'
import assert from 'node:assert/strict'
import {retentionPolicyErrors,sameRetentionPolicy} from '../src/retentionPolicy.ts'

test('automatic retention requires an effective limit and accepts native boundaries',()=>{
  const unlimited={enabled:false,maxCount:null,maxBytes:null}
  assert.deepEqual(retentionPolicyErrors(unlimited),{maxCount:null,maxBytes:null,limits:null})
  assert.ok(retentionPolicyErrors({...unlimited,enabled:true}).limits)
  for(const maxCount of [1,1000])for(const maxBytes of [1024**2,2048*1024**3]){
    assert.deepEqual(retentionPolicyErrors({enabled:true,maxCount,maxBytes}),{maxCount:null,maxBytes:null,limits:null})
  }
  assert.deepEqual(retentionPolicyErrors({...unlimited,enabled:true,maxCount:1}),{maxCount:null,maxBytes:null,limits:null})
  assert.deepEqual(retentionPolicyErrors({...unlimited,enabled:true,maxBytes:1024**2}),{maxCount:null,maxBytes:null,limits:null})
})
test('invalid deletion limits and fractional counts remain errors even while disabled',()=>{
  const policy={enabled:false,maxCount:10,maxBytes:20*1024**3}
  for(const maxCount of [0,-1,1001,1.5,Infinity,NaN])assert.ok(retentionPolicyErrors({...policy,maxCount}).maxCount)
  for(const maxBytes of [0,1024**2-1,2048*1024**3+1,1.5,Infinity,NaN])assert.ok(retentionPolicyErrors({...policy,maxBytes}).maxBytes)
  assert.equal(sameRetentionPolicy(policy,{...policy}),true)
  assert.equal(sameRetentionPolicy(policy,{...policy,maxCount:null}),false)
})
