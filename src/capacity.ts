type Capacity = {memoryMinMb:number;memoryMaxMb:number;maxPlayers:number;storageTargetGb:number;port?:number}
const within = (value:number,min:number,max:number) => Number.isInteger(value)&&value>=min&&value<=max
export const isCapacityValid = (value:Capacity) =>
  within(value.memoryMinMb,512,value.memoryMaxMb)&&within(value.memoryMaxMb,512,65536)&&
  within(value.maxPlayers,1,1000)&&within(value.storageTargetGb,1,2048)&&
  (value.port===undefined||within(value.port,1,65535))
