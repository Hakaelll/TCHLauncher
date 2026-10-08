const normalize=(value:string)=>value.normalize('NFD').replace(/\p{M}/gu,'').toLowerCase().trim()
export const settingsMatch=(query:string,...text:string[])=>normalize(text.join(' ')).includes(normalize(query))
export function propertyAdditionError(key:string,value:string,values:Record<string,string>):string|null {
  key=key.trim()
  if(!/^[a-zA-Z0-9_.-]{1,80}$/.test(key))return 'Usa una clave de hasta 80 caracteres: letras, números, puntos, guiones o guiones bajos.'
  if(Object.hasOwn(values,key))return 'La propiedad ya existe. Modifica su valor en la lista.'
  if(/[\r\n]/.test(value)||new TextEncoder().encode(value).length>1024)return 'El valor debe ocupar como máximo 1024 bytes y no contener saltos de línea.'
  return null
}
