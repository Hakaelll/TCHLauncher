/** Bounded console transport buffer. Raw server logs remain on disk. */
export function createLogBatch<T>(capacity:number,publish:(lines:T[])=>void,
  schedule:(callback:()=>void)=>unknown=callback=>setTimeout(callback,100),
  cancel:(handle:unknown)=>void=handle=>clearTimeout(handle as ReturnType<typeof setTimeout>)){
  if(!Number.isInteger(capacity)||capacity<1)throw Error('Invalid log capacity')
  const slots:Array<T|undefined>=new Array(capacity)
  let head=0,size=0,timer:unknown=null,revision=0,active=true,paused=false
  const clear=()=>{slots.fill(undefined);head=0;size=0}
  const queue=()=>{
    if(!active||paused||!size||timer!==null)return
    const ticket=revision;timer=schedule(()=>{
      if(!active||ticket!==revision)return
      timer=null
      const lines=Array.from({length:size},(_,index)=>slots[(head+index)%capacity]!)
      clear();if(lines.length)publish(lines)
    })
  }
  return {
    push(line:T){
      if(!active)return
      slots[(head+size)%capacity]=line
      if(size===capacity)head=(head+1)%capacity;else size++
      queue()
    },
    setPaused(value:boolean){
      if(!active||paused===value)return
      paused=value;revision++;if(timer!==null)cancel(timer);timer=null
      queue()
    },
    reset(){revision++;if(timer!==null)cancel(timer);timer=null;clear()},
    dispose(){active=false;this.reset()},
  }
}
