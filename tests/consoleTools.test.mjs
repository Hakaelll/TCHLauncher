import {test} from 'node:test'
import assert from 'node:assert/strict'
import {rememberCommand,recallCommand,countLogLevels} from '../src/consoleTools.ts'

test('command recall is bounded, preserves order and avoids consecutive duplicates',()=>{
  let history=[]
  for(let index=0;index<70;index++)history=rememberCommand(history,`say ${index}`)
  assert.equal(history.length,50)
  assert.equal(history[0],'say 20')
  assert.equal(rememberCommand(history,'  say 69  '),history)
  assert.equal(rememberCommand(history,'   '),history)
  assert.deepEqual(rememberCommand(['list','help'],'list'),['list','help','list'])
})

test('history navigation restores an unsent draft and clamps at the oldest command',()=>{
  const history=['list','help'];let state=recallCommand(history,null,'','say unfinished','up')
  assert.deepEqual(state,{cursor:1,draft:'say unfinished',value:'help'})
  state=recallCommand(history,state.cursor,state.draft,state.value,'up')
  assert.equal(state.value,'list')
  assert.equal(recallCommand(history,state.cursor,state.draft,state.value,'up').cursor,0)
  state=recallCommand(history,state.cursor,state.draft,state.value,'down')
  state=recallCommand(history,state.cursor,state.draft,state.value,'down')
  assert.deepEqual(state,{cursor:null,draft:'say unfinished',value:'say unfinished'})
  assert.equal(recallCommand([],null,'','draft','up').value,'draft')
})

test('typing a new draft restarts recall without resurrecting an older draft',()=>{
  assert.deepEqual(recallCommand(['list'],null,'obsolete','new draft','up'),{cursor:0,draft:'new draft',value:'list'})
  assert.equal(recallCommand(['list'],null,'obsolete','new draft','down').value,'new draft')
})

test('log category counts retain unknown native levels and represent the supplied searched lines only',()=>{
  const line=(level,message)=>({serverId:'one',level,message,line:message,timestamp:null,event:null})
  const logs=[line('error','mod A failed'),line('error','mod B failed'),line('warn','mod A warning'),line('debug','checking')]
  assert.deepEqual(countLogLevels(logs),{all:4,error:2,warn:1,debug:1})
  assert.deepEqual(countLogLevels(logs.filter(row=>row.message.includes('mod A'))),{all:2,error:1,warn:1})
  assert.deepEqual(countLogLevels([]),{all:0})
})
