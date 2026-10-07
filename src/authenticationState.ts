import type {AuthenticationCheck} from './api'

/** Configuration evidence only. A running server may still use pre-save values. */
export function authenticationState(onlineMode:boolean,checks:AuthenticationCheck[]) {
  const enabled=checks.filter(check=>check.enabled===true)
  const unknown=checks.filter(check=>check.enabled===null)
  return {enabled,unknown,kind:enabled.length?'proxy' as const:unknown.length?'unknown' as const:onlineMode?'direct' as const:'offline' as const}
}
