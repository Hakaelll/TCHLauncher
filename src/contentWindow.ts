export type ContentState = 'error'|'warning'|'observed'|'unknown'
export type ContentFilter = 'all'|'attention'|'observed'|'unknown'
export type ContentSource = 'all'|'managed'|'local'
type InventoryRow = {
  title:string
  status:ContentState
  file:{filename:string;projectId?:string|null}
  check?:{descriptor:{id?:string|null;ids?:string[]}}
}

export function contentWindow<T extends InventoryRow>(rows:readonly T[],query:string,filter:ContentFilter,source:ContentSource,sort:'name'|'attention'):T[] {
  const search=query.trim().toLocaleLowerCase('es')
  const rank={error:0,warning:1,unknown:2,observed:3}
  return rows.filter(row=>
    (!search||[row.title,row.file.filename,row.check?.descriptor.id??'',...(row.check?.descriptor.ids??[])].some(value=>value.toLocaleLowerCase('es').includes(search)))&&
    (filter==='all'||filter==='attention'&&['error','warning'].includes(row.status)||row.status===filter)&&
    (source==='all'||source==='managed'&&!!row.file.projectId||source==='local'&&!row.file.projectId)
  ).sort((a,b)=>(sort==='attention'?rank[a.status]-rank[b.status]:0)||a.title.localeCompare(b.title,'es',{numeric:true,sensitivity:'base'}))
}
