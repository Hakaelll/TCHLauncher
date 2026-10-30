type Capacity = {memoryMinMb:number;memoryMaxMb:number;maxPlayers:number;storageTargetGb:number;port?:number}
const within = (value:number,min:number,max:number) => Number.isInteger(value)&&value>=min&&value<=max
export const isCapacityValid = (value:Capacity) =>
  within(value.memoryMinMb,512,value.memoryMaxMb)&&within(value.memoryMaxMb,512,65536)&&
  within(value.maxPlayers,1,1000)&&within(value.storageTargetGb,1,2048)&&
  (value.port===undefined||within(value.port,1,65535))

/** Advisory starting heap, not a prediction of a world's or modpack's requirements. */
export function memoryStartingPoint({physicalBytes,modded,maxPlayers}:{physicalBytes:number;modded:boolean;maxPlayers:number}):number|null {
  if(!Number.isFinite(physicalBytes)||physicalBytes<=0||!within(maxPlayers,1,100))return null
  const physicalMb=Math.floor(physicalBytes/1024**2)
  const budget=Math.floor((physicalMb-Math.max(2048,physicalMb*.25))/256)*256
  const baseline=modded?4096:2048
  if(budget<baseline)return null
  const slots=Math.ceil(Math.max(0,maxPlayers-10)/10)*512
  return Math.min(budget,baseline+slots,8192)
}
