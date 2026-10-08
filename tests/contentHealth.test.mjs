import {test} from 'node:test'
import assert from 'node:assert/strict'
import {contentSignals,contentSignalsFromDiagnostics,prepareContentDiagnostics,contentStatus} from '../src/contentHealth.ts'

const file={filename:'backpack-1.0.jar',title:'Better Backpack',projectId:'real',versionId:'v1',versionNumber:'1.0',sha512:'a',size:42,modified:false}
const check={filename:file.filename,descriptor:{id:'backpack',name:'Better Backpack'},issues:[],listedInLog:true}
const line=(message,level='error',prefix='[12:00:00] [main/ERROR]: ')=>( {serverId:'real',level,message,line:prefix+message,timestamp:'2026-10-05T12:00:00Z',event:'mod_error'})

test('explicit loader and mixin references color only the mentioned mod and preserve evidence',()=>{
  const log=line("Could not execute entrypoint 'main', provided by 'backpack' at 'example.Entrypoint'")
  const signals=contentSignals(file,check,[log])
  assert.equal(signals.length,1);assert.equal(signals[0].query,'backpack');assert.equal(signals[0].message,log.message)
  assert.equal(contentStatus(check,signals),'error')
  assert.equal(contentSignals({...file,filename:'structures.jar',title:'Structures'},{...check,descriptor:{id:'structures'}},[log]).length,0)
  assert.equal(contentSignals(file,check,[line('Mixin apply for mod failed from mod backpack')]).length,1)
  assert.equal(contentSignals(file,check,[line('Mixin apply failed from mod backpack_extra')]).length,0)
})

test('jar paths, logger ids and actual plugin enabling errors remain attributable',()=>{
  assert.equal(contentSignals(file,check,[line("Failed to load mods/backpack-1.0.jar")]).length,1)
  assert.equal(contentSignals(file,check,[line('Initialization failed','error','[12:00:00] [main/ERROR] [backpack]: ')]).length,1)
  assert.equal(contentSignals(file,check,[line('Error occurred while enabling Better Backpack v1')]).length,1)
  assert.equal(contentSignals(file,check,[line('Failed to load backpack-1.0.jar.extra')]).length,0)
  assert.equal(contentSignals(file,check,[line('Error: Could not load backpack-1.0.jar','error','')]).length,1)
})

test('chat, commands, generic mentions and INFO discussions never accuse a mod',()=>{
  for(const log of [line('<player> mod backpack ERROR'),line('[Not Secure] <player> mod backpack ERROR'),line('Player issued server command: mod backpack'),line('No error occurred in mod backpack','error','[12:00:00] [main/INFO]: '),line('backpack is also installed'),line('Error: data about backpack_extra'),line('mod BackpackOther failed')]) {
    assert.deepEqual(contentSignals(file,check,[log]),[])
  }
})

test('warnings differ from errors; diagnostics and confirmed presence retain their meaning',()=>{
  const warning=contentSignals(file,check,[line("Mod 'backpack' has a warning",'warn','[12:00:00 WARN]: ')])
  assert.equal(contentStatus(check,warning),'warning')
  assert.equal(contentStatus(check,[]),'observed')
  assert.equal(contentStatus({...check,listedInLog:null},[]),'unknown')
  assert.equal(contentStatus({...check,issues:['Duplicate ID']},[]),'warning')
  assert.equal(contentStatus({...check,listedInLog:false},[]),'warning')
})

test('evidence is bounded, deduplicated and changes with the supplied server log',()=>{
  const entries=Array.from({length:5},(_,i)=>line(`Mod 'backpack' failed ${i}`))
  const signals=contentSignals(file,check,[...entries,entries[4]])
  assert.equal(signals.length,3);assert.match(signals[0].message,/failed 4/)
  assert.deepEqual(contentSignals(file,check,[]),[])
})

test('Fabric dependency continuations inherit only a bounded real diagnostic block',()=>{
  const header=line('Incompatible mods found!')
  const continuation=line("- Mod 'A backpack mod' (backpack) 1.0 requires library",'info','')
  assert.equal(contentSignals(file,check,[header,continuation])[0].level,'error')
  assert.equal(contentSignals(file,check,[continuation]).length,0)
  assert.equal(contentSignals(file,check,[header,line('Starting world','info','[12:00:01] [main/INFO]: '),continuation]).length,0)
  assert.equal(contentSignals(file,check,[header,...Array.from({length:33},()=>line('ordinary continuation','info','')),continuation]).length,0)
  const warning=line('WARN: not an ERROR in the header','warn','[12:00:00 WARN]: ')
  assert.equal(contentSignals(file,check,[warning,continuation])[0].level,'warn')
})

test('later warnings do not hide a related error that remains in the recent log window',()=>{
  const logs=[line("Mod 'backpack' failed"),...Array.from({length:5},(_,i)=>line(`Mod 'backpack' warning ${i}`,'warn','[12:00:00 WARN]: '))]
  const signals=contentSignals(file,check,logs)
  assert.equal(signals.length,3);assert.equal(contentStatus(check,signals),'error')
})

test('raw Log4j headers associate NeoForge diagnostics and reset continuation severity',()=>{
  const error=line('Failed to load backpack-1.0.jar','info','2026-10-05 12:00:00,001 main ERROR ')
  assert.equal(contentSignals(file,check,[error])[0].level,'error')
  const continuation=line("- Mod 'A backpack mod' (backpack) 1.0 requires library",'info','')
  assert.equal(contentSignals(file,check,[error,continuation])[0].level,'error')
  const info=line('Checking mod backpack for errors','error','2026-10-05 12:00:01,001 main INFO ')
  assert.deepEqual(contentSignals(file,check,[info,continuation]),[])
  const warning=line('WARN: mod backpack needs library','info','')
  assert.equal(contentSignals(file,check,[warning,continuation])[0].level,'warn')
})

test('secondary declared NeoForge identities associate errors with their shared JAR',()=>{
  const multi={...check,descriptor:{...check.descriptor,ids:['backpack','backpack_api']}}
  const error=line('Mixin failed from mod backpack_api')
  assert.equal(contentSignals(file,multi,[error])[0].query,'backpack_api')
  assert.deepEqual(contentSignals(file,check,[error]),[])
  assert.deepEqual(contentSignals(file,multi,[line('Mixin failed from mod backpack_api_extra')]),[])
})


test('shared inventory diagnostics preserve multi-file associations and reject chat and INFO mentions',()=>{
  const other={...file,filename:'structures.jar',title:'Structures'}
  const otherCheck={...check,filename:other.filename,descriptor:{id:'structures',name:'Structures'}}
  const logs=[line('Incompatible mods found!'),line("- Mod 'Backpack' (backpack) requires structures",'info',''),
    line('Failed to load structures.jar and backpack-1.0.jar','warn','[12:00:01 WARN]: '),
    line('<player> mod structures failed'),line('Checking mod backpack','info','[12:00:02] [main/INFO]: '),
    line("- Mod 'Backpack' (backpack) is mentioned",'info','')]
  const diagnostics=prepareContentDiagnostics(logs)
  const backpack=contentSignalsFromDiagnostics(file,check,diagnostics)
  const structures=contentSignalsFromDiagnostics(other,otherCheck,diagnostics)
  assert.deepEqual(backpack.map(signal=>[signal.level,signal.query]),[['warn',file.filename],['error','backpack']])
  assert.deepEqual(structures.map(signal=>[signal.level,signal.query]),[['warn',other.filename]])
  assert.equal(contentStatus(check,backpack),'error')
  assert.equal(contentStatus(otherCheck,structures),'warning')
})

test('prepared diagnostics are isolated snapshots reusable across files and new sessions',()=>{
  const logs=[line("Mod 'backpack' failed")]
  const diagnostics=prepareContentDiagnostics(logs)
  const original=contentSignalsFromDiagnostics(file,check,diagnostics)
  logs[0].message='<player> mod backpack failed';logs[0].line='[12:00:01] [main/INFO]: '+logs[0].message
  assert.deepEqual(contentSignalsFromDiagnostics(file,check,diagnostics),original)
  assert.deepEqual(contentSignalsFromDiagnostics(file,check,prepareContentDiagnostics(logs)),[])
  assert.deepEqual(contentSignalsFromDiagnostics(file,check,prepareContentDiagnostics([])),[])
  assert.deepEqual(contentSignalsFromDiagnostics(file,check,diagnostics),original)
})

test('Windows Paper prompts and ANSI preserve severity without treating chat or INFO as evidence',()=>{
  const error=line('Error occurred while enabling Better Backpack v1','info','> > \r  \r\x1b[31m[12:00:00 ERROR]: \x1b[0m')
  const snapshot=JSON.stringify(error)
  assert.equal(contentSignals(file,check,[error])[0].level,'error')
  assert.equal(JSON.stringify(error),snapshot)
  const warning=line("Mod 'backpack' needs a library",'info','> > [12:00:00 WARN]: ')
  assert.equal(contentSignals(file,check,[warning])[0].level,'warn')
  const continuation=line("- Mod 'Backpack' (backpack) requires library",'info','')
  const info=line('Starting world','error','> > [12:00:01 INFO]: ')
  assert.deepEqual(contentSignals(file,check,[info,continuation]),[])
  assert.deepEqual(contentSignals(file,check,[error,info,continuation]).map(signal=>signal.level),['error'])
  assert.deepEqual(contentSignals(file,check,[line('<player> mod backpack failed','error','> [12:00:01 ERROR]: ')]),[])
  assert.deepEqual(contentSignals(file,check,[line('> > [ERROR] mod backpack failed','info','[12:00:01 INFO]: ')]),[])
})

test('large diagnostic bursts retain the latest error and latest distinct warning occurrences',()=>{
  const error=line("Mod 'backpack' failed")
  const warnings=Array.from({length:1000},(_,i)=>line(`Mod 'backpack' warning ${i}`,'warn','[12:00:01 WARN]: '))
  const repeated={...warnings[0],timestamp:'2026-10-05T12:00:02Z'}
  const signals=contentSignals(file,check,[error,...warnings,repeated])
  assert.equal(signals.length,3)
  assert.deepEqual(signals.map(signal=>signal.message),[repeated.message,warnings[999].message,error.message])
  assert.equal(signals[0].timestamp,repeated.timestamp)
  assert.equal(contentStatus(check,signals),'error')
  const repeatedError={...error,timestamp:'2026-10-05T12:00:03Z'}
  assert.equal(contentSignals(file,check,[error,...warnings,repeatedError])[0].timestamp,repeatedError.timestamp)
})
