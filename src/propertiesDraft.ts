export type PropertiesDraft={serverId:string|null;values:Record<string,string>;loaded:boolean;loading:boolean;dirty:boolean;error:string|null}
export const emptyPropertiesDraft=():PropertiesDraft=>({serverId:null,values:{},loaded:false,loading:false,dirty:false,error:null})

export function createPropertiesDraft(read:(id:string)=>Promise<Record<string,string>>,publish:(value:PropertiesDraft)=>void){
  let state=emptyPropertiesDraft(),revision=0,active=true
  const update=(value:PropertiesDraft)=>{state=value;publish(value)}
  return {
    select(serverId:string|null){revision++;update({...emptyPropertiesDraft(),serverId})},
    edit(values:Record<string,string>){
      if(!active||!state.loaded)return
      revision++;update({...state,values:{...values},dirty:true,loading:false})
    },
    acknowledge(serverId:string,values:Record<string,string>){
      if(!active||state.serverId!==serverId)return
      revision++;update({...state,values:{...values},dirty:false,loading:false,loaded:true,error:null})
    },
    async refresh(serverId=state.serverId,replace=false){
      if(!active||!serverId||state.serverId!==serverId||state.dirty&&!replace)return
      const ticket=++revision
      update(replace?{...emptyPropertiesDraft(),serverId,loading:true}:{...state,loading:true,error:null})
      try{
        const values=await read(serverId)
        if(!active||revision!==ticket||state.serverId!==serverId)return
        update({...state,values:{...values},loaded:true,loading:false,dirty:false,error:null})
      }catch(cause){
        if(!active||revision!==ticket||state.serverId!==serverId)return
        update({...state,loading:false,error:String(cause)});throw cause
      }
    },
    dispose(){active=false;revision++},
  }
}
