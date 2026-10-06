import {test} from 'node:test'
import assert from 'node:assert/strict'
import {playerActionErrors} from '../src/playerActions.ts'

test('player input rejects command injection and preserves supported Minecraft names',()=>{
  assert.deepEqual(playerActionErrors('op',' Alex_1 ',''),{})
  for(const target of ['','ab','name with space','Alex\nstop','12345678901234567','álex'])assert.ok(playerActionErrors('op',target,'').target)
  assert.deepEqual(playerActionErrors('ban_ip','2001:db8::1',''),{})
  assert.ok(playerActionErrors('ban_ip',' ','').target)
})
test('reasons use Unicode character limits and reject embedded command controls',()=>{
  assert.deepEqual(playerActionErrors('ban','Alex','á'.repeat(120)),{})
  assert.ok(playerActionErrors('kick','Alex','á'.repeat(121)).reason)
  for(const reason of ['spam\nstop','spam\rstop','spam\u0085stop','spam\0stop'])assert.ok(playerActionErrors('ban','Alex',reason).reason)
  assert.deepEqual(playerActionErrors('op','Alex','unused reason\n'),{})
})
