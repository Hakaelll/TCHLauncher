export const themeIds = ['end', 'obsidian', 'nether'] as const
export type ThemeId = typeof themeIds[number]
export type Appearance = { theme: ThemeId; font: 'minecraft' | 'reading' }
export const appearanceKey = 'tch-appearance-v1'
export const defaultAppearance: Appearance = { theme: 'end', font: 'minecraft' }

export function parseAppearance(raw: string | null): Appearance {
  try {
    const value: unknown = raw ? JSON.parse(raw) : null
    if (!value || typeof value !== 'object' || Array.isArray(value)) return { ...defaultAppearance }
    const prefs = value as Record<string, unknown>
    return {
      theme: themeIds.includes(prefs.theme as ThemeId) ? prefs.theme as ThemeId : defaultAppearance.theme,
      font: prefs.font === 'reading' ? 'reading' : 'minecraft',
    }
  } catch { return { ...defaultAppearance } }
}

type PreferenceStorage = { getItem: (key: string) => string | null; setItem: (key: string, value: string) => void }
export function readAppearance(storage: Pick<PreferenceStorage, 'getItem'> | null): Appearance {
  try { return parseAppearance(storage?.getItem(appearanceKey) ?? null) }
  catch { return { ...defaultAppearance } }
}
export function saveAppearance(storage: Pick<PreferenceStorage, 'setItem'> | null, value: Appearance): boolean {
  try {
    if (!storage) return false
    storage.setItem(appearanceKey, JSON.stringify(value))
    return true
  } catch { return false }
}
export function browserPreferenceStorage(): Storage | null {
  try { return window.localStorage } catch { return null }
}
export function applyAppearance(value: Appearance, root: HTMLElement = document.documentElement): void {
  root.dataset.theme = value.theme
  root.dataset.font = value.font
}
