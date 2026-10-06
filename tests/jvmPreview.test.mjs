import {test} from 'node:test'
import assert from 'node:assert/strict'
import {jvmPreviewKey} from '../src/jvmPreview.ts'
const server={id:'a',platform:'fabric',version:'1.21.11',directory:'server-a',jarPath:'launcher.jar',javaPath:'java21.exe',memoryMinMb:1024,memoryMaxMb:4096}
const custom={mode:'custom',arguments:'-XX:+UseG1GC'}

test('a preview cannot authorize a different runtime, memory, launcher, server or profile',()=>{
  const original=jvmPreviewKey(server,custom)
  for(const change of [{id:'b'},{platform:'neoforge'},{version:'1.20.1'},{directory:'other'},{jarPath:null},{javaPath:'java17.exe'},{memoryMinMb:512},{memoryMaxMb:8192}])assert.notEqual(jvmPreviewKey({...server,...change},custom),original)
  assert.notEqual(jvmPreviewKey(server,{...custom,arguments:'-ea'}),original)
  assert.notEqual(jvmPreviewKey(server,{mode:'recommended',arguments:''}),original)
})

test('restoring ignored recommended-profile text does not invalidate equivalent effective arguments',()=>{
  assert.equal(jvmPreviewKey(server,{mode:'recommended',arguments:'-ea'}),jvmPreviewKey(server,{mode:'recommended',arguments:''}))
  assert.notEqual(jvmPreviewKey(server,{mode:'custom',arguments:'-ea'}),jvmPreviewKey(server,{mode:'custom',arguments:''}))
})
