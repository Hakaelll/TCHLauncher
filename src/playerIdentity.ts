export function isOfflineUuid(uuid:string):boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-3[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(uuid)
}

export const identityKey=(uuid:string)=>uuid.replaceAll('-','').toLowerCase()
