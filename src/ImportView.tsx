import {errorText} from './format'
import {translatedMessage,t} from './locale.ts'
import {isTauri} from '@tauri-apps/api/core'
import {Icon,platformName} from './ServerPresentation'
import {SliderField} from './SliderField'
import {useEffect,useState} from 'react'
import {api,type ImportInput,type ImportInspection,type Version} from './api'

export function ImportView({onCancel,onImport,busy}: {onCancel:()=>void;onImport:(input:ImportInput)=>Promise<void>;busy:boolean}) {
  const [inspection,setInspection] = useState<ImportInspection|null>(null)
  const [versions,setVersions] = useState<Version[]>([])
  const [formError,setFormError] = useState<string|null>(null)
  const [imagePreview,setImagePreview] = useState<string|null>(null)
  const [picking,setPicking] = useState(false)
  const [input,setInput] = useState<ImportInput>({path:'',name:'',version:'',platform:'vanilla',jarFile:'',copy:true,storageTargetGb:20,imagePath:null})
  useEffect(() => {
    if (!isTauri()) { setFormError(t("Abre la aplicación de escritorio para importar una carpeta.")); return }
    api.versions().then(items => { setVersions(items) }).catch(e => setFormError(t("No se pudo consultar el catálogo: {0}", errorText(e))))
  },[])
  const choose = async () => {
    setPicking(true); setFormError(null)
    try {
      const path = await api.pickImportFolder()
      if (!path) return
      const info = await api.inspectImport(path)
      setInspection(info)
      const known=info.launchers.filter(launcher=>launcher.platform)
      const selected=known.length===1?known[0]:info.launchers.length===1?info.launchers[0]:null
      setInput(old=>({path:info.path,name:info.suggestedName,version:selected?.version??'',platform:selected?.platform??'vanilla',jarFile:selected?.filename??'',copy:true,storageTargetGb:old.storageTargetGb,imagePath:old.imagePath}))
    } catch(e) { setFormError(errorText(e)) }
    finally { setPicking(false) }
  }
  const chooseImage = async () => {
    setFormError(null)
    try { const image = await api.pickServerImage(); if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) } }
    catch(e) { setFormError(errorText(e)) }
  }
  return <div className="form-page import-page"><div className="form-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/>  {t("Servidores")}</button><h1>{t("Importar servidor")}</h1><p>{t("Trae un servidor existente a tu biblioteca sin perder sus archivos.")}</p></div><div className="import-source"><div className="import-source-icon"><Icon name="folder" size={22}/></div><div><strong>{inspection ? t("Carpeta seleccionada") : t("Selecciona la carpeta del servidor")}</strong><p>{inspection ? inspection.path : t("Debe contener server.properties y un archivo de arranque compatible")}</p></div><button onClick={choose} disabled={picking || busy}>{picking ? t("Inspeccionando…") : inspection ? t("Cambiar carpeta") : t("Elegir carpeta")}</button></div>{formError && <p className="form-error" role="alert">{translatedMessage(formError)}</p>}{inspection && <form className="create-form import-form" onSubmit={e => { e.preventDefault(); onImport(input) }}><div className="form-section-title"><span>01</span><div><h2>{t("Identidad")}</h2><p>{t("Así aparecerá en tu biblioteca.")}</p></div></div><div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt={t("Vista previa de la imagen")}/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage}>{t("Elegir imagen")}</button><p>{t("PNG, JPG o WebP · hasta 2 MB")}</p></div><div className="form-row"><label htmlFor="import-name">{t("Nombre del servidor")}</label><input id="import-name" required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})}/></div></div><div className="form-section-title"><span>02</span><div><h2>{t("Datos del servidor")}</h2><p>{t("Revisa los valores detectados.")}</p></div></div><div className="form-pair"><div className="form-row"><label htmlFor="import-version">{t("Versión de Minecraft")}</label><input id="import-version" list="import-versions" required value={input.version} onChange={e => setInput({...input,version:e.target.value})}/><datalist id="import-versions">{versions.map(v => <option key={v.id} value={v.id}/>)}</datalist></div><div className="form-row"><label htmlFor="import-platform">{t("Tipo")}</label><select id="import-platform" value={input.platform} onChange={e => setInput({...input,platform:e.target.value as ImportInput['platform']})}><option value="vanilla">Vanilla</option><option value="paper">Paper</option><option value="fabric">Fabric</option><option value="neoforge">NeoForge</option></select></div></div><p className="field-help">{t("Comprueba la versión del servidor original. La usaremos para elegir Java compatible.")}</p><div className="form-row"><label htmlFor="import-jar">{t("Archivo del servidor")}</label><select id="import-jar" required value={input.jarFile} onChange={e=>{const filename=e.target.value;const launcher=inspection.launchers.find(launcher=>launcher.filename===filename);setInput(old=>({...old,jarFile:filename,platform:launcher?.platform??'vanilla',version:launcher?.version??''}))}}><option value="">{t("Selecciona el archivo de arranque")}</option>{inspection.launchers.map(launcher=><option key={launcher.filename} value={launcher.filename}>{launcher.filename}{launcher.platform?` · ${platformName(launcher.platform)}`:''}{launcher.version?` · ${launcher.version}`:''}</option>)}</select>{inspection.launchers.find(launcher=>launcher.filename===input.jarFile)?.note&&<p className="field-help">{translatedMessage(inspection.launchers.find(launcher=>launcher.filename===input.jarFile)?.note)}</p>}</div><div className="import-storage"><SliderField id="import-storage" label={t("Objetivo de espacio")} value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/><p className="field-help">{t("Se usa para avisar cuando el servidor ocupa más espacio. No limita el disco.")}</p></div><fieldset className="import-mode"><legend>{t("Ubicación de los archivos")}</legend><label><input type="radio" checked={input.copy} onChange={() => setInput({...input,copy:true})}/>  {t("Copiar a la biblioteca de TCHLauncher")}</label><label><input type="radio" checked={!input.copy} onChange={() => setInput({...input,copy:false})}/>  {t("Administrar la carpeta actual")}</label><p className="field-help">{t("Si administras la carpeta actual, los cambios y las restauraciones afectarán a los archivos originales.")}</p></fieldset><div className="form-actions"><button type="button" onClick={onCancel}>{t("Cancelar")}</button><button className="primary-button" disabled={busy || !input.version || !input.jarFile || input.storageTargetGb < 1 || input.storageTargetGb > 2048}>{busy ? t("Importando…") : t("Importar servidor")}<Icon name="arrow" size={16}/></button></div></form>}</div>
}
