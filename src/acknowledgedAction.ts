import {errorText} from './format.ts'
export async function acknowledgedAction<T>(action:()=>Promise<T>,refresh:()=>Promise<unknown>){
  const value=await action()
  try{await refresh();return {value,refreshError:null}}
  catch(error){return {value,refreshError:errorText(error)}}
}
