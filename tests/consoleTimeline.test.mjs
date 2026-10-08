import {test} from 'node:test'
import assert from 'node:assert/strict'
import {appendConsoleLines,mergeConsoleHistory,consoleBoundary,consoleLinesAfterClear} from '../src/consoleTimeline.ts'
const line=(arrival,text='output',timestamp=null,serverId='one')=>({serverId,level:'info',line:text,message:text,timestamp,event:null,arrival})

test('clear keeps new arrival order despite absent, legacy, future or reversed timestamps',()=>{
  const before=[line(1,'old',null),line(2,'future','2099-01-01T00:00:00Z'),line(0,'history','12:00:00')]
  const boundary=consoleBoundary(before,'one')
  const after=appendConsoleLines(before,[line(3,'new',null),line(4,'legacy','00:00:00'),line(5,'past','2000-01-01T00:00:00Z')])
  assert.deepEqual(consoleLinesAfterClear(after,'one',boundary).map(row=>row.line),['new','legacy','past'])
  assert.deepEqual(consoleLinesAfterClear(after,'one',null),after)
  assert.equal(consoleLinesAfterClear([line(1)],'one',0).length,1)
})
test('history refresh preserves live identities and duplicate occurrence counts after clear',()=>{
  const first=line(1),second=line(2),newer=line(3,'next')
  const current=[first,second,newer]
  const saved=[line(0),line(0)]
  const merged=mergeConsoleHistory(current,saved,'one')
  assert.deepEqual(merged,current)
  assert.equal(merged[0],first);assert.equal(merged[1],second)
  assert.deepEqual(consoleLinesAfterClear(merged,'one',2),[newer])
  assert.deepEqual(mergeConsoleHistory(current,[line(0)],'one'),current)
})
test('a reload cannot resurrect cleared disk history, mix servers or alter native data',()=>{
  const current=[line(1),line(2,'foreign',null,'two')],saved=[line(0),line(0,'older'),line(0,'foreign',null,'two')]
  const snapshot=JSON.stringify(saved)
  const merged=mergeConsoleHistory(current,saved,'one')
  assert.deepEqual(consoleLinesAfterClear(merged,'one',1),[])
  assert.equal(merged.length,2);assert.equal(JSON.stringify(saved),snapshot)
  assert.equal(consoleBoundary(current,'one'),1)
})
test('burst appends and repeated history refreshes retain only the newest bounded view',()=>{
  const current=Array.from({length:1000},(_,index)=>line(index+1,String(index)))
  const incoming=Array.from({length:1500},(_,index)=>line(index+1001,String(index+1000)))
  const bounded=appendConsoleLines(current,incoming)
  assert.equal(bounded.length,1000);assert.equal(bounded[0].arrival,1501)
  const saved=bounded.map(row=>({...row,arrival:0}))
  for(let i=0;i<10;i++)assert.deepEqual(mergeConsoleHistory(bounded,saved,'one'),bounded)
  assert.equal(current.length,1000);assert.equal(incoming.length,1500)
})

test('disk duplicates get distinct stable identities across refreshes and bounded appends',()=>{
  const saved=Array.from({length:1000},()=>line(0))
  const first=mergeConsoleHistory([],saved,'one')
  assert.equal(new Set(first.map(row=>row.arrival)).size,1000)
  assert.ok(first.every(row=>row.arrival<0))
  const refreshed=mergeConsoleHistory(first,saved,'one')
  assert.ok(refreshed.every((row,index)=>row===first[index]))
  const appended=appendConsoleLines(refreshed,[line(1,'new')])
  assert.equal(appended.length,1000)
  assert.ok(appended.slice(0,-1).every((row,index)=>row===first[index+1]))
  const merged=mergeConsoleHistory(appended,[...saved,line(0,'new'),line(0,'other')],'one')
  assert.equal(new Set(merged.map(row=>row.arrival)).size,1000)
  assert.deepEqual(consoleLinesAfterClear(merged,'one',0).map(row=>row.line),['new'])
  assert.equal(consoleBoundary(first,'one'),0)
  assert.deepEqual(consoleLinesAfterClear(first,'one',0),[])
})
