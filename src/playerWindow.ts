import {listWindow} from './listWindow.ts'

// Presentation only: preserve native identities, order and data; never merge names.
export function playerWindow<T>(rows:readonly T[],query:string,text:(row:T)=>string,requestedPage:number){
  const needle=query.trim().toLocaleLowerCase()
  const matching=needle?rows.filter(row=>text(row).toLocaleLowerCase().includes(needle)):rows
  return listWindow(matching,requestedPage)
}
