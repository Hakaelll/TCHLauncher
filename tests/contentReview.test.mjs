import {test} from 'node:test'
import assert from 'node:assert/strict'
import {prepareContent} from '../src/contentReview.ts'

const deferred=()=>{let resolve;const promise=new Promise(done=>{resolve=done});return {promise,resolve}}
const versions=[{id:'beta',version_type:'beta'},{id:'stable',version_type:'release'}]
const request={kind:'project',projectId:'lithium',versionId:null}
const options={serverId:'fabric-server',request,isCurrent:()=>true}

test('project reviews select a release, honor explicit versions and allow an experimental-only catalog',async()=>{
  const calls=[]
  const source={contentVersions:async()=>versions,planContent:async(...args)=>{calls.push(args);return {token:args[2]}},planContentUpdates:async()=>{throw new Error('unexpected updates')}}
  assert.deepEqual(await prepareContent({...options,source}),{kind:'project',value:{token:'stable'},versions})
  await prepareContent({...options,source,request:{...request,versionId:'beta'}})
  source.contentVersions=async()=>[versions[0]]
  await prepareContent({...options,source})
  assert.deepEqual(calls,[['fabric-server','lithium','stable'],['fabric-server','lithium','beta'],['fabric-server','lithium','beta']])
})

test('a replacement publishes its entire review after planning and retains the previous result on failure',async()=>{
  const previous={kind:'updates',value:{token:'previous'}},planning=deferred()
  let selection=previous
  const source={contentVersions:async()=>versions,planContent:()=>planning.promise,planContentUpdates:async()=>({token:'updates'})}
  const pending=prepareContent({...options,source}).then(value=>{if(value)selection=value})
  await Promise.resolve()
  assert.equal(selection,previous)
  const review={token:'replacement'}
  planning.resolve(review);await pending
  assert.deepEqual(selection,{kind:'project',value:review,versions})
  const retained=selection,failure=new Error('catalog unavailable')
  source.planContentUpdates=async()=>{throw failure}
  await assert.rejects(prepareContent({...options,source,request:{kind:'updates'}}).then(value=>{if(value)selection=value}),error=>error===failure)
  assert.equal(selection,retained)
  source.planContent=async()=>{throw failure}
  source.contentVersions=async()=>[{id:'new-catalog',version_type:'release'}]
  await assert.rejects(prepareContent({...options,source}).then(value=>{if(value)selection=value}),error=>error===failure)
  assert.equal(selection,retained)
  source.planContentUpdates=async()=>({token:'fresh-updates'})
  assert.deepEqual(await prepareContent({...options,source,request:{kind:'updates'}}),{kind:'updates',value:{token:'fresh-updates'}})
})

test('obsolete catalog and planning completions cannot publish or start further planning',async()=>{
  const catalog=deferred()
  let current=true,plans=0
  const source={contentVersions:()=>catalog.promise,planContent:async()=>{plans++;throw new Error('obsolete catalog started planning')},planContentUpdates:async()=>{throw new Error('unexpected updates')}}
  const pending=prepareContent({...options,source,isCurrent:()=>current})
  current=false;catalog.resolve(versions)
  assert.equal(await pending,null)
  assert.equal(plans,0)
  for(const next of [request,{kind:'updates'}]){
    const completion=deferred()
    source.planContent=()=>{plans++;return completion.promise}
    source.planContentUpdates=()=>completion.promise
    current=true
    const pendingPlan=prepareContent({...options,source,request:next,isCurrent:()=>current})
    await Promise.resolve()
    current=false;completion.resolve({token:'obsolete'})
    assert.equal(await pendingPlan,null)
  }
  assert.equal(await prepareContent({...options,source,isCurrent:()=>false}),null)
})

test('an empty compatible catalog does not plan and read failures preserve their cause',async()=>{
  let plans=0
  const source={contentVersions:async()=>[],planContent:async()=>{plans++;return {token:'invalid'}}}
  await assert.rejects(prepareContent({...options,source}),/No hay una publicación compatible/)
  assert.equal(plans,0)
  const failure=new Error('native read failed')
  source.contentVersions=async()=>{throw failure}
  await assert.rejects(prepareContent({...options,source}),error=>error===failure)
  assert.equal(plans,0)
})

test('publication rechecks ownership after a plan resolves and before the awaiting caller resumes',async()=>{
  const planning=deferred(),previous={kind:'updates',value:{token:'retained'}}
  let sequence=1,selection=previous
  const started=sequence
  const source={planContentUpdates:()=>planning.promise}
  const pending=prepareContent({...options,source,request:{kind:'updates'},isCurrent:()=>sequence===started}).then(value=>{
    if(value&&sequence===started)selection=value
  })
  const invalidate=planning.promise.then(()=>{sequence++})
  planning.resolve({token:'obsolete'})
  await Promise.all([pending,invalidate])
  assert.equal(selection,previous)
})
