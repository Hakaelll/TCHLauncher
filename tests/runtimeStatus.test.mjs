import {test} from 'node:test'
import assert from 'node:assert/strict'
import {runtimeStatus} from '../src/runtimeStatus.ts'
import {createServerRead} from '../src/serverRead.ts'
const ready={status:'ready',compatibility:'compatible',version:'21.0.12',managed:true,path:'java.exe',error:null,requiredMajor:21,distribution:'Eclipse Temurin'}

test('failed runtime observation cannot preserve a ready label from earlier successful data',async()=>{
  let state
  const replies=[Promise.resolve(ready),Promise.reject(new Error('IPC disconnected')),Promise.resolve(ready)]
  const reader=createServerRead(()=>replies.shift(),value=>state=value)
  reader.select('a');await reader.refresh()
  assert.equal(runtimeStatus(state.data,state.loading,state.error).label,'Listo')
  await assert.rejects(reader.refresh(),/IPC disconnected/)
  assert.equal(state.data,ready)
  const failed=runtimeStatus(state.data,state.loading,state.error)
  assert.equal(failed.kind,'unavailable');assert.equal(failed.label,'Sin comprobar');assert.match(failed.description,/Última lectura/)
  await reader.refresh();assert.equal(runtimeStatus(state.data,state.loading,state.error).label,'Listo')
})

test('verified version does not imply known compatibility and invalid or missing Java stays actionable',()=>{
  assert.equal(runtimeStatus({...ready,compatibility:'unknown'},false,null).label,'Versión comprobada')
  assert.equal(runtimeStatus({...ready,compatibility:'incompatible'},false,null).label,'Incompatible')
  assert.equal(runtimeStatus({...ready,status:'invalid',compatibility:'unknown'},false,null).label,'No válido')
  assert.equal(runtimeStatus({...ready,status:'missing',compatibility:'unknown'},false,null).label,'Sin instalar')
  assert.equal(runtimeStatus(ready,true,null).label,'Comprobando')
  assert.equal(runtimeStatus(null,false,'timeout').label,'Sin comprobar')
})
