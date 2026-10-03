// The launcher can reduce motion further, but never override an OS accessibility preference.
export function shouldReduceMotion(preference: string | undefined, systemReduced: boolean): boolean {
  return preference === 'reduced' || systemReduced
}
