export type OperatorPermissions={level:number;bypassesPlayerLimit:boolean}
export type OperatorDraft={identity:string;baseline:OperatorPermissions;draft:OperatorPermissions}
export const samePermissions=(a:OperatorPermissions,b:OperatorPermissions)=>a.level===b.level&&a.bypassesPlayerLimit===b.bypassesPlayerLimit
export const operatorDirty=(state:OperatorDraft)=>!samePermissions(state.baseline,state.draft)
export function operatorDraft(identity:string,permissions:OperatorPermissions):OperatorDraft{return {identity,baseline:{...permissions},draft:{...permissions}}}
export function reconcileOperator(state:OperatorDraft,identity:string,permissions:OperatorPermissions){
  return state.identity!==identity||!operatorDirty(state)?operatorDraft(identity,permissions):state
}
export function operatorConflict(state:OperatorDraft,identity:string,permissions:OperatorPermissions){return state.identity!==identity||(operatorDirty(state)&&!samePermissions(state.baseline,permissions))}
