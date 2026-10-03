import {test} from 'node:test'
import assert from 'node:assert/strict'
import {launcherCommands,filterCommands} from '../src/commands.ts'
const server={id:'test-server',name:'End de Yago',platform:'paper',version:'1.17.1',state:'stopped',jarPath:'server.jar',javaPath:'java.exe'}
test('process actions follow native state and background-operation restrictions',()=>{
  for(const state of ['stopped','running','starting','stopping','installing','crashed']) {
    const commands=launcherCommands([{...server,state}],false,new Set())
    const backup=commands.find(command=>command.action==='backup')
    assert.equal(!backup.disabledReason,['stopped','crashed'].includes(state))
    if(['running','starting'].includes(state))assert.ok(commands.find(command=>command.action==='stop'))
    else assert.equal(!commands.find(command=>command.action==='start').disabledReason,['stopped','crashed'].includes(state))
  }
  assert.ok(launcherCommands([server],false,new Set([server.id])).find(command=>command.action==='start').disabledReason)
  assert.ok(launcherCommands([{...server,javaPath:null}],false,new Set()).find(command=>command.action==='start').disabledReason)
})
test('search matches action and server words independently of accents',()=>{
  const commands=launcherCommands([server],false,new Set())
  assert.equal(filterCommands(commands,'copia yago')[0].action,'backup')
  assert.equal(filterCommands(commands,'configuracion').length,0)
  assert.equal(filterCommands(commands,'tipografia')[0].action,'appearance')
  assert.equal(filterCommands(commands,'inexistente').length,0)
})
test('Vanilla does not offer content browsing and navigation remains available while busy',()=>{
  const commands=launcherCommands([{...server,platform:'vanilla'}],true,new Set())
  assert.ok(!commands.some(command=>command.action==='content'))
  assert.ok(commands.find(command=>command.action==='start').disabledReason)
  assert.ok(!commands.find(command=>command.action==='console').disabledReason)
  assert.ok(!commands.find(command=>command.action==='appearance').disabledReason)
})
