type Owner = {working:boolean}

export async function contentOperation({owner,isCurrent,setBusy,work}:{owner:Owner;isCurrent:()=>boolean;setBusy:(busy:boolean)=>void;work:()=>Promise<void>}){
  if(owner.working||!isCurrent())return
  owner.working=true
  try{setBusy(true);await work()}
  finally{owner.working=false;if(isCurrent())setBusy(false)}
}
