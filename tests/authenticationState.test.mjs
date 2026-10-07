import {test} from 'node:test'
import assert from 'node:assert/strict'
import {authenticationState} from '../src/authenticationState.ts'
const check=(enabled,key='settings.bungeecord')=>({file:'spigot.yml',key,enabled,problem:enabled===null?'unreadable':null})
test('enabled forwarding wins over online-mode and does not claim direct authentication',()=>{
  assert.equal(authenticationState(true,[check(true)]).kind,'proxy')
  assert.equal(authenticationState(false,[check(true),check(null,'other')]).kind,'proxy')
  assert.equal(authenticationState(true,[check(true),check(null)]).unknown.length,1)
})
test('unknown config never implies forwarding disabled or direct account validation',()=>{
  assert.equal(authenticationState(true,[check(null)]).kind,'unknown')
  assert.equal(authenticationState(false,[check(null)]).kind,'unknown')
  assert.equal(authenticationState(true,[check(false)]).kind,'direct')
  assert.equal(authenticationState(false,[check(false)]).kind,'offline')
  assert.equal(authenticationState(true,[]).kind,'direct')
})
