// Presentation only: preserve native identities, order and data; never merge names.
export function playerWindow<T>(rows:readonly T[],query:string,text:(row:T)=>string,requestedPage:number){
  const needle=query.trim().toLocaleLowerCase()
  const matching=needle?rows.filter(row=>text(row).toLocaleLowerCase().includes(needle)):rows
  const pages=Math.max(1,Math.ceil(matching.length/50))
  const page=Math.min(pages-1,Math.max(0,Number.isFinite(requestedPage)?Math.floor(requestedPage):0))
  return {rows:matching.slice(page*50,page*50+50),total:matching.length,pages,page}
}
