import type {PlayerIdentity} from './api'

export function isOfflineUuid(uuid:string):boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-3[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(uuid)
}

// usercache can retain both offline and account UUIDs for the same name.
// Never select whichever record happens to occur first. Authenticated direct
// connections cannot use a local UUID. Proxy/offline configurations can use
// either, so an ambiguous cache must keep a neutral avatar.
export function connectedIdentity(name:string,records:PlayerIdentity[],onlineMode:boolean):string|null {
  const candidates=new Set(records
    .filter(record=>record.name.toLowerCase()===name.toLowerCase())
    .map(record=>record.uuid?.toLowerCase())
    .filter((uuid):uuid is string=>!!uuid && /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(uuid))
    .filter(uuid=>!onlineMode || !isOfflineUuid(uuid)))
  return candidates.size===1 ? [...candidates][0] : null
}
