export const normalizeSearchText=(value:string)=>value.normalize('NFD').replace(/\p{M}/gu,'').toLowerCase().trim()
