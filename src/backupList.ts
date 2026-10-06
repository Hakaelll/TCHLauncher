import type {Backup} from './api'

export type BackupListState={serverId:string|null;copies:Backup[];loaded:boolean;loading:boolean;error:string|null}
export const emptyBackupList=():BackupListState=>({serverId:null,copies:[],loaded:false,loading:false,error:null})

export function createBackupList(read:(id:string)=>Promise<Backup[]>,publish:(state:BackupListState)=>void){
  let state=emptyBackupList(),revision=0,active=true
  const update=(next:BackupListState)=>{state=next;publish(next)}
  return {
    select(serverId:string|null){revision++;update({...emptyBackupList(),serverId})},
    async refresh(serverId=state.serverId){
      if(!active||!serverId||serverId!==state.serverId)return
      const ticket=++revision
      update({...state,loading:true,error:null})
      try{
        const copies=await read(serverId)
        if(!active||ticket!==revision||state.serverId!==serverId)return
        update({...state,copies,loaded:true,loading:false,error:null})
      }catch(cause){
        if(!active||ticket!==revision||state.serverId!==serverId)return
        update({...state,loading:false,error:String(cause)})
        throw cause
      }
    },
    dispose(){active=false;revision++},
  }
}
