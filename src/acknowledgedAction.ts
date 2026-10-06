export async function acknowledgedAction(action:()=>Promise<unknown>,refresh:()=>Promise<unknown>){
  await action()
  try{await refresh();return {refreshError:null}}
  catch(error){return {refreshError:error instanceof Error?error.message:String(error)}}
}
