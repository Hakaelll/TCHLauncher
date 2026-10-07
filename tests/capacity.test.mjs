import {test} from 'node:test'
import assert from 'node:assert/strict'
import {isCapacityValid} from '../src/capacity.ts'
const base={memoryMinMb:1024,memoryMaxMb:2048,maxPlayers:20,storageTargetGb:20,port:25565}
test('capacity accepts native integer boundaries and settings without a port',()=>{
  assert.equal(isCapacityValid(base),true)
  assert.equal(isCapacityValid({memoryMinMb:512,memoryMaxMb:512,maxPlayers:1,storageTargetGb:1,port:1}),true)
  assert.equal(isCapacityValid({memoryMinMb:65536,memoryMaxMb:65536,maxPlayers:1000,storageTargetGb:2048,port:65535}),true)
  const {port,...settings}=base;assert.equal(isCapacityValid(settings),true)
})
test('creation and settings reject fractional, unavailable and reversed capacity',()=>{
  for(const key of Object.keys(base))for(const value of [base[key]+0.5,NaN,Infinity,-Infinity,0])
    assert.equal(isCapacityValid({...base,[key]:value}),false,`${key}: ${value}`)
  for(const change of [{memoryMinMb:511},{memoryMinMb:2049},{memoryMaxMb:65537},{maxPlayers:1001},{storageTargetGb:2049},{port:65536}])
    assert.equal(isCapacityValid({...base,...change}),false,JSON.stringify(change))
})
