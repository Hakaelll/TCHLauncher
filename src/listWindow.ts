/** Bound presentation without changing native identities, ordering or input arrays. */
export function listWindow<T>(rows:readonly T[],requestedPage:number,size:24|50=50){
  const pages=Math.max(1,Math.ceil(rows.length/size))
  const page=Math.min(pages-1,Math.max(0,Number.isFinite(requestedPage)?Math.floor(requestedPage):0))
  return {rows:rows.slice(page*size,page*size+size),total:rows.length,pages,page}
}
