type Clock={schedule:(callback:()=>void)=>unknown;cancel:(handle:unknown)=>void}
const retryClock:Clock={schedule:callback=>setTimeout(callback,5000),cancel:handle=>clearTimeout(handle as ReturnType<typeof setTimeout>)}

export function retryingSubscription(register:()=>Promise<()=>void>,onUnavailable:(unavailable:boolean)=>void,clock:Clock=retryClock){
  let active=true,timer:unknown=null,unlisten:(()=>void)|undefined
  const connect=async()=>{
    try{
      const stop=await register()
      if(!active){stop();return}
      unlisten=stop;onUnavailable(false)
    }catch{
      if(!active)return
      onUnavailable(true)
      timer=clock.schedule(()=>{timer=null;void connect()})
    }
  }
  void connect()
  return()=>{active=false;if(timer!==null)clock.cancel(timer);try{unlisten?.()}catch{}}
}
