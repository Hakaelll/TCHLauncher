import {errorText} from './format.ts'
export type ServerRead<T>={serverId:string|null;data:T|null;loading:boolean;error:string|null}
export function emptyServerRead<T>():ServerRead<T>{return {serverId:null,data:null,loading:false,error:null}}
export function createServerRead<T>(read:(id:string)=>Promise<T>,publish:(state:ServerRead<T>)=>void){
  let state=emptyServerRead<T>(),revision=0,active=true
  const emit=(next:ServerRead<T>)=>{state=next;publish(state)}
  return {
    isLoading(){return state.loading},
    select(id:string|null){revision++;emit({...emptyServerRead<T>(),serverId:id})},
    async refresh(id=state.serverId){
      if(!active||!id||id!==state.serverId)return
      const ticket=++revision
      emit({...state,loading:true})
      try{
        const data=await read(id)
        if(active&&ticket===revision&&state.serverId===id)emit({serverId:id,data,loading:false,error:null})
      }catch(error){
        if(!active||ticket!==revision||state.serverId!==id)return
        emit({...state,loading:false,error:errorText(error)})
        throw error
      }
    },
    // Only publish a native acknowledgement; invalidate reads begun before the write.
    acknowledge(id:string,data:T){
      if(!active||id!==state.serverId)return false
      revision++;emit({serverId:id,data,loading:false,error:null});return true
    },
    dispose(){active=false;revision++},
  }
}
