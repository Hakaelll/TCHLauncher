// Bound background compatibility work and stop scheduling when the page changes.
export async function checkPage<T>(items:T[],check:(item:T)=>Promise<void>,active:()=>boolean,parallel=4) {
  let next=0
  await Promise.all(Array.from({length:Math.min(parallel,items.length)},async()=>{
    while(active()) {const index=next++;if(index>=items.length)return;await check(items[index])}
  }))
}
