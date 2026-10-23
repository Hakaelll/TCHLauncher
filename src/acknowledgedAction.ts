import {errorText} from './format.ts'
export async function acknowledgedAction<T>(action:()=>Promise<T>,refresh:(value:T)=>Promise<unknown>){
  const value=await action()
  try{await refresh(value);return {value,refreshError:null}}
  catch(error){return {value,refreshError:errorText(error)}}
}
