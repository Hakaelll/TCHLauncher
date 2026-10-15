import {test} from 'node:test'
import assert from 'node:assert/strict'
import {scheduledResult,scheduledState} from '../src/scheduledState.ts'
import {setLanguage} from '../src/locale.ts'

test('scheduled errors, interruptions and requests remain distinct from completed server startup',()=>{
  assert.equal(scheduledResult('Error: Copia omitida: servidor activo').tone,'error')
  assert.equal(scheduledResult('Interrumpida al cerrar la aplicación; no se repite esta ejecución').label,'Interrumpida')
  assert.equal(scheduledResult('Copia cancelada; servidor detenido').label,'Cancelada')
  assert.equal(scheduledResult('Inicio solicitado').label,'Solicitud enviada')
  assert.equal(scheduledResult('Un resultado desconocido').tone,'neutral')
  assert.equal(scheduledResult(null).label,'Sin ejecución anterior')
})
test('current schedule state does not hide a paused task or treat epoch timestamps as missing',()=>{
  const task={invalid:false,running:false,enabled:false,lastAt:0,intervalMinutes:null}
  assert.equal(scheduledState(task),'Finalizada')
  assert.equal(scheduledState({...task,intervalMinutes:60}),'Pausada')
  assert.equal(scheduledState({...task,enabled:true}),'Programada')
  assert.equal(scheduledState({...task,running:true,enabled:true}),'En ejecución')
  assert.equal(scheduledState({...task,invalid:true,running:true}),'Datos inválidos')
})

test('task state and result labels follow language changes without changing native result classification',()=>{
  const task={invalid:false,running:false,enabled:false,lastAt:0,intervalMinutes:null}
  setLanguage('en',null)
  try{
    assert.equal(scheduledState(task),'Completed')
    assert.equal(scheduledState({...task,intervalMinutes:60}),'Paused')
    assert.equal(scheduledResult('Interrumpida al cerrar la aplicación').label,'Interrupted')
    assert.equal(scheduledResult('Copia cancelada; servidor detenido').label,'Cancelled')
    assert.equal(scheduledResult('Reinicio solicitado').label,'Request sent')
    assert.equal(scheduledResult('Error: native read').tone,'error')
  }finally{setLanguage('es',null)}
  assert.equal(scheduledState(task),'Finalizada')
})
