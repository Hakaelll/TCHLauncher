import type { CSSProperties } from 'react'
import type { Appearance, ThemeId } from './appearance'
import launcherEmblem from './assets/launcher-logo.png'

const themes: { id: ThemeId; name: string; description: string; paper: string; surface: string; accent: string }[] = [
  { id: 'end', name: 'End', description: 'Obsidiana, piedra oscura y luz violeta.', paper: '#131217', surface: '#1c1b22', accent: '#baa2ee' },
  { id: 'obsidian', name: 'Obsidiana', description: 'Gris antracita, contraste y calma.', paper: '#141518', surface: '#292b30', accent: '#c2c7d2' },
  { id: 'nether', name: 'Nether', description: 'Carbón oscuro y pequeños acentos de brasa.', paper: '#191414', surface: '#302526', accent: '#f1ab95' },
]

export function AppearanceSettings({ value, saved, onChange }: { value: Appearance; saved: boolean; onChange: (value: Appearance) => void }) {
  return <section className="appearance-settings" aria-labelledby="appearance-title">
    <header className="appearance-heading"><h1 id="appearance-title">Ajustes</h1><p>Haz tuyo el launcher.</p></header>
    <fieldset className="theme-picker"><legend>Tema</legend><p className="preference-help">Elige una apariencia. El cambio se aplica al momento.</p>
      <div className="theme-options">{themes.map(theme => <label className="theme-option" key={theme.id} style={{ '--sample-paper': theme.paper, '--sample-surface': theme.surface, '--sample-accent': theme.accent } as CSSProperties}>
        <input type="radio" name="launcher-theme" value={theme.id} checked={value.theme === theme.id} onChange={() => onChange({ ...value, theme: theme.id })}/>
        <span className="theme-preview" aria-hidden="true"><span className="preview-rail"><img src={launcherEmblem} alt=""/><i/><i/></span><span className="preview-workspace"><span className="preview-title"/><span className="preview-tabs"><i/><i/><i/></span><span className="preview-body"><i/><i/></span><span className="preview-button"/></span></span>
        <span className="theme-choice-heading"><strong>{theme.name}</strong><span className="theme-selected" aria-hidden="true"><svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2"><path d="m3 8 3 3 7-7"/></svg></span></span>
        <span className="theme-description">{theme.description}</span>
      </label>)}</div>
    </fieldset>
    <fieldset className="font-picker"><legend>Tipografía</legend><p className="preference-help">Títulos y controles con personalidad. Los textos largos conservan su legibilidad.</p><div className="font-options">
      <label><input type="radio" name="launcher-font" checked={value.font === 'minecraft'} onChange={() => onChange({ ...value, font: 'minecraft' })}/><span><strong>Minecraft</strong><span className="font-example minecraft">Mi servidor</span></span></label>
      <label><input type="radio" name="launcher-font" checked={value.font === 'reading'} onChange={() => onChange({ ...value, font: 'reading' })}/><span><strong>Lectura</strong><span className="font-example reading">Mi servidor</span></span></label>
    </div></fieldset>
    <p className={`preference-save ${saved ? '' : 'failed'}`} role="status">{saved ? 'Los cambios se guardan automáticamente en este equipo.' : 'El cambio está aplicado, pero no se pudo guardar. Comprueba el almacenamiento de la aplicación antes de cerrarla.'}</p>
  </section>
}
