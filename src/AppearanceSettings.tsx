import {translatedMessage,localizedStatic,t} from './locale.ts'
import {useState,type CSSProperties} from 'react'
import {api} from './api'
import type { Appearance, ThemeId } from './appearance'
import launcherEmblem from './assets/launcher-logo.png'
import {getLanguage,languageSaved,setLanguage} from './locale.ts'
import {useLanguage} from './useLanguage'
import {browserPreferenceStorage} from './appearance'

const themes: { id: ThemeId; name: string; description: string; paper: string; surface: string; accent: string }[] = localizedStatic([
  { id: 'end', name: 'End', description: 'Obsidiana, piedra oscura y luz violeta.', paper: '#131217', surface: '#1c1b22', accent: '#baa2ee' },
  { id: 'obsidian', name: 'Obsidiana', description: 'Gris antracita, contraste y calma.', paper: '#141518', surface: '#292b30', accent: '#c2c7d2' },
  { id: 'nether', name: 'Nether', description: 'Carbón oscuro y pequeños acentos de brasa.', paper: '#191414', surface: '#302526', accent: '#f1ab95' },
])

export function AppearanceSettings({ value, saved, onChange, embedded=false }: { embedded?:boolean; value: Appearance; saved: boolean; onChange: (value: Appearance) => void }) {
  useLanguage()
  return <section className="appearance-settings" aria-label={t("Apariencia")}>
    {!embedded&&<header className="appearance-heading"><h1 id="appearance-title">{t("Ajustes")}</h1><p>{t("Haz tuyo el launcher.")}</p></header>}
    <fieldset className="theme-picker"><legend>{t("Tema")}</legend><p className="preference-help">{t("Elige una apariencia. El cambio se aplica al momento.")}</p>
      <div className="theme-options">{themes.map(theme => <label className="theme-option" key={theme.id} style={{ '--sample-paper': theme.paper, '--sample-surface': theme.surface, '--sample-accent': theme.accent } as CSSProperties}>
        <input type="radio" name="launcher-theme" value={theme.id} checked={value.theme === theme.id} onChange={() => onChange({ ...value, theme: theme.id })}/>
        <span className="theme-preview" aria-hidden="true"><span className="preview-rail"><img src={launcherEmblem} alt=""/><i/><i/></span><span className="preview-workspace"><span className="preview-title"/><span className="preview-tabs"><i/><i/><i/></span><span className="preview-body"><i/><i/></span><span className="preview-button"/></span></span>
        <span className="theme-choice-heading"><strong>{theme.name}</strong><span className="theme-selected" aria-hidden="true"><svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="2"><path d="m3 8 3 3 7-7"/></svg></span></span>
        <span className="theme-description">{translatedMessage(theme.description)}</span>
      </label>)}</div>
    </fieldset>
    <fieldset className="language-picker"><legend>{t("Idioma de la aplicación")}</legend><p className="preference-help">{t("El cambio se aplica al momento y se guarda en este equipo.")}</p><div className="language-options">
      <label lang="es"><input type="radio" name="launcher-language" checked={getLanguage()==='es'} onChange={()=>setLanguage('es',browserPreferenceStorage(),document.documentElement)}/><span>Español</span></label>
      <label lang="en"><input type="radio" name="launcher-language" checked={getLanguage()==='en'} onChange={()=>setLanguage('en',browserPreferenceStorage(),document.documentElement)}/><span>English</span></label>
    </div>{!languageSaved()&&<p className="form-error" role="alert">{t("No se pudo guardar el idioma. El cambio está aplicado hasta que cierres la aplicación.")}</p>}</fieldset>
    <fieldset className="font-picker"><legend>{t("Tipografía")}</legend><p className="preference-help">{t("Títulos y controles con personalidad. Los textos largos conservan su legibilidad.")}</p><div className="font-options">
      <label><input type="radio" name="launcher-font" checked={value.font === 'minecraft'} onChange={() => onChange({ ...value, font: 'minecraft' })}/><span><strong>Minecraft</strong><span className="font-example minecraft">{t("Mi servidor")}</span></span></label>
      <label><input type="radio" name="launcher-font" checked={value.font === 'reading'} onChange={() => onChange({ ...value, font: 'reading' })}/><span><strong>{t("Lectura")}</strong><span className="font-example reading">{t("Mi servidor")}</span></span></label>
    </div></fieldset>
    <section className="preference-diagnostics" aria-labelledby="motion-title"><h2 id="motion-title">{t("Movimiento")}</h2><label className="motion-preference"><input type="checkbox" checked={value.motion==='reduced'} onChange={event=>onChange({...value,motion:event.target.checked?'reduced':'system'})}/><span>{t("Reducir animaciones")}</span></label><p>{t("Evita desplazamientos animados y transiciones. La preferencia de movimiento reducido del sistema siempre se respeta.")}</p></section>
    <p className={`preference-save ${saved ? '' : 'failed'}`} role="status">{saved ? t("Los cambios se guardan automáticamente en este equipo.") : t("El cambio está aplicado, pero no se pudo guardar. Comprueba el almacenamiento de la aplicación antes de cerrarla.")}</p>

  </section>
}

export function DiagnosticsSettings(){
  const [openingLogs,setOpeningLogs]=useState(false)
  const [logError,setLogError]=useState<string|null>(null)
  const openLogs=async()=>{
    if(openingLogs)return
    setOpeningLogs(true);setLogError(null)
    try{await api.openApplicationLogs()}catch(error){setLogError(error instanceof Error?error.message:String(error))}finally{setOpeningLogs(false)}
  }
  return <section className="application-diagnostics" aria-label={t("Diagnóstico y atajos")}>
    <section className="preference-diagnostics" aria-labelledby="diagnostics-title"><h2 id="diagnostics-title">{t("Diagnóstico")}</h2><p>{t("Logs del launcher con inicio, cierre y estados de los servidores. Se conservan el registro actual y uno anterior, hasta 2 MB cada uno. La consola de Minecraft se consulta en su servidor.")}</p><button disabled={openingLogs} onClick={()=>void openLogs()}>{openingLogs?t("Abriendo…"):t("Abrir logs de la aplicación")}</button>{logError&&<p className="form-error" role="alert">{translatedMessage(logError)}</p>}</section>
    <section className="preference-diagnostics" aria-labelledby="shortcuts-title"><h2 id="shortcuts-title">{t("Atajos")}</h2><dl className="data-list"><div><dt>{t("Buscar acciones y servidores")}</dt><dd><kbd>Ctrl + K</kbd></dd></div><div><dt>{t("Nuevo servidor")}</dt><dd><kbd>Ctrl + N</kbd></dd></div><div><dt>{t("Buscar en la vista actual")}</dt><dd><kbd>Ctrl + F</kbd></dd></div></dl></section>
  </section>
}
