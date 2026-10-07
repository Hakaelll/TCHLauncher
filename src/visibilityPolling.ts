type Environment={visible:()=>boolean;subscribe:(callback:()=>void)=>()=>void;schedule:(callback:()=>void,delay:number)=>unknown;cancel:(handle:unknown)=>void}
const browserEnvironment=():Environment=>({
  visible:()=>document.visibilityState!=='hidden',
  subscribe:callback=>{document.addEventListener('visibilitychange',callback);return()=>document.removeEventListener('visibilitychange',callback)},
  schedule:(callback,delay)=>setTimeout(callback,delay),cancel:handle=>clearTimeout(handle as ReturnType<typeof setTimeout>),
})

/** Only controls observations; never server execution or scheduled tasks. */
export function startVisibilityPolling(read:()=>Promise<void>,intervalMs:number,options:{hiddenMs?:number;immediate?:boolean}={},environment:Environment=browserEnvironment()){
  let active=true,running=false,timer:unknown=null
  const delay=()=>environment.visible()?intervalMs:options.hiddenMs??0
  const clear=()=>{if(timer!==null)environment.cancel(timer);timer=null}
  const schedule=()=>{clear();const ms=delay();if(active&&ms>0)timer=environment.schedule(()=>{timer=null;void run()},ms)}
  const run=async()=>{
    if(!active||running)return
    running=true
    try{await read()}finally{running=false;if(active)schedule()}
  }
  const unsubscribe=environment.subscribe(()=>{
    clear()
    if(!running){if(environment.visible())void run();else schedule()}
  })
  if(options.immediate)void run();else schedule()
  return()=>{active=false;clear();unsubscribe()}
}
