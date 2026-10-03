import {test} from 'node:test'
import assert from 'node:assert/strict'
import {dismissTask,isTaskActive,orderedTasks,reconcileTasks} from '../src/taskState.ts'

const progress=(serverId,taskId,stage='downloading')=>({serverId,taskId,stage,kind:'install',label:'Java',received:20,total:100,bytesPerSecond:1,attempt:1,error:null})
test('native snapshots recover missing jobs and remove vanished active jobs without inventing an outcome',()=>{
  const completed=progress('a','old','completed')
  const missing=progress('b','gone')
  const active=progress('c','live')
  assert.deepEqual(reconcileTasks({a:completed,b:missing},[active]),{a:completed,c:active})
  assert.deepEqual(reconcileTasks({},[active]),{c:active})
})
test('an older snapshot cannot undo a terminal event for the same task, but a new task replaces the previous outcome',()=>{
  const completed=progress('a','old','failed')
  assert.deepEqual(reconcileTasks({a:completed},[progress('a','old')]),{a:completed})
  const newer=progress('a','new')
  assert.deepEqual(reconcileTasks({a:completed},[newer]),{a:newer})
})
test('only a matching terminal task can be dismissed; active tasks and a newer replacement are protected',()=>{
  const active={a:progress('a','new')}
  assert.equal(dismissTask(active,'a','new'),active)
  const completed={a:progress('a','old','cancelled')}
  assert.equal(dismissTask(completed,'a','wrong'),completed)
  assert.deepEqual(dismissTask(completed,'a','old'),{})
})
test('active tasks precede errors and completed tasks without treating terminal outcomes as active',()=>{
  const done=progress('a','a','completed'),failed=progress('b','b','failed'),active=progress('c','c')
  assert.deepEqual(orderedTasks({a:done,b:failed,c:active}),[active,failed,done])
  for(const stage of ['preparing','connecting','downloading','verifying','extracting','retrying'])assert.equal(isTaskActive(progress('x','x',stage)),true)
  for(const stage of ['completed','failed','cancelled'])assert.equal(isTaskActive(progress('x','x',stage)),false)
})
