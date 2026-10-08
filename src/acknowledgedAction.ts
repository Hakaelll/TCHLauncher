import {errorText} from './format.ts'
export async function acknowledgedAction(action:()=>Promise<unknown>,refresh:()=>Promise<unknown>){
  await action()
  try{await refresh();return {refreshError:null}}
  catch(error){return {refreshError:errorText(error)}}
}
