import type {api,ContentReview,ContentVersion} from './api'
import {t} from './locale.ts'

export type ContentRequest = {kind:'project';projectId:string;versionId:string|null}|{kind:'updates'}
export type PreparedContent = {kind:'project';value:ContentReview;versions:ContentVersion[]}|{kind:'updates';value:ContentReview}

export async function prepareContent({source,serverId,request,isCurrent}:{source:Pick<typeof api,'contentVersions'|'planContent'|'planContentUpdates'>;serverId:string;request:ContentRequest;isCurrent:()=>boolean}):Promise<PreparedContent|null>{
  if(!isCurrent())return null
  if(request.kind==='updates'){
    const value=await source.planContentUpdates(serverId)
    return isCurrent()?{kind:'updates',value}:null
  }
  const versions=await source.contentVersions(serverId,request.projectId)
  if(!isCurrent())return null
  const version=request.versionId??versions.find(value=>value.version_type==='release')?.id??versions[0]?.id
  if(!version)throw new Error(t("No hay una publicación compatible con este servidor."))
  const value=await source.planContent(serverId,request.projectId,version)
  return isCurrent()?{kind:'project',value,versions}:null
}
