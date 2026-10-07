import {isCapacityValid} from './capacity'
import {errorText} from './format'
import {translatedMessage,t} from './locale.ts'
import {isTauri} from '@tauri-apps/api/core'
import {Icon,fmtBytes,platformName} from './ServerPresentation'
import {SliderField} from './SliderField'
import {useEffect,useRef,useState} from 'react'
import {api,type CreateInput,type Version,type WorldInspection} from './api'
import {MovingIndicator,AnimatedValue,useNavigationMotion} from './motion'
import worldIllustration from './assets/choice-world.png'

export function CreateView({onCancel,onCreate,busy,systemMemory}: {onCancel:()=>void;onCreate:(input:CreateInput,worldSource?:string)=>Promise<void>;busy:boolean;systemMemory:number}) {
  const steps = [t("Identidad"),t("Versión"),t("Mundo"),t("Capacidad"),t("Revisar")] as const
  const [step,setStep] = useState(0)
  const wizardRef=useRef<HTMLDivElement>(null)
  const [stepDirection,setStepDirection]=useState(1)
  const moveStep=(next:number)=>{setStepDirection(next>step?1:-1);setStep(next)}
  useNavigationMotion(wizardRef,String(step),stepDirection,16)
  const [furthestStep,setFurthestStep] = useState(0)
  const [versions,setVersions] = useState<Version[]>([])
  const [versionLoading,setVersionLoading]=useState(true)
  const [experimental,setExperimental]=useState(false)
  const [catalogRevision,setCatalogRevision]=useState(0)
  const [versionError,setVersionError] = useState<string|null>(null)
  const [imagePreview,setImagePreview] = useState<string|null>(null)
  const [imageError,setImageError] = useState<string|null>(null)
  const [worldMode,setWorldMode] = useState<'new'|'existing'>('new')
  const [worldSource,setWorldSource] = useState<WorldInspection|null>(null)
  const [worldError,setWorldError] = useState<string|null>(null)
  const [input,setInput] = useState<CreateInput>({ name:'',version:'',platform:'vanilla',memoryMinMb:1024,memoryMaxMb:2048,maxPlayers:20,port:25565,storageTargetGb:20,imagePath:null,seed:'' })
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(input.memoryMaxMb,systemMemoryMb || 16384))
  const invalidCapacity = !isCapacityValid(input)
  const invalidSeed=worldMode==='new'&&(new TextEncoder().encode(input.seed??'').length>1024||/[\u0000-\u001f\u007f]/.test(input.seed??''))
  const readyToCreate = !invalidSeed && !!input.name.trim() && !!input.version && !versionError && !versionLoading && !invalidCapacity && (worldMode === 'new' || !!worldSource)
  const canAdvance = step === 0 ? !!input.name.trim() : step === 1 ? !!input.version && !versionError && !versionLoading : step === 2 ? !invalidSeed && (worldMode === 'new' || !!worldSource) : step === 3 ? !invalidCapacity : readyToCreate
  const loadVersions = () => setCatalogRevision(value=>value+1)
  useEffect(() => {
    let active=true;setVersionLoading(true);setVersionError(null)
    if(!isTauri()){setVersionError(t("Abre la aplicación de escritorio para consultar versiones."));setVersionLoading(false);return}
    api.versions(input.platform,experimental&&['vanilla','fabric'].includes(input.platform)).then(items=>{
      if(!active)return
      setVersions(items)
      if(!items.length)setVersionError(t("No hay versiones compatibles disponibles."))
      setInput(old=>({...old,version:items.some(item=>item.id===old.version)?old.version:items[0]?.id || ''}))
    }).catch(cause=>{if(active)setVersionError(errorText(cause))}).finally(()=>{if(active)setVersionLoading(false)})
    return ()=>{active=false}
  },[input.platform,experimental,catalogRevision])
  const chooseImage = async () => {
    setImageError(null)
    try {
      const image = await api.pickServerImage()
      if (image) { setImagePreview(image.data); setInput(old => ({...old,imagePath:image.path})) }
    } catch(e) { setImageError(errorText(e)) }
  }
  const chooseWorld = async () => {
    setWorldError(null)
    try { const selected = await api.pickWorldFolder(); if (selected) setWorldSource(selected) }
    catch(e) { setWorldError(errorText(e)) }
  }
  const previousStep=useRef(step)
  useEffect(() => { if(previousStep.current!==step)wizardRef.current?.querySelector<HTMLElement>('h2')?.focus();previousStep.current=step },[step])
  const advance = () => {
    if (!canAdvance || busy) return
    if (step < steps.length - 1) {
      const next = step + 1
      moveStep(next)
      setFurthestStep(current => Math.max(current,next))
    } else onCreate({...input,seed:worldMode==='new'?input.seed:''},worldMode === 'existing' ? worldSource?.path : undefined)
  }
  return <div className="form-page">
    <div className="form-heading wizard-heading"><button className="back-button" onClick={onCancel}><Icon name="back" size={15}/>  {t("Servidores")}</button><div className="wizard-title-row"><h1>{t("Nuevo servidor")}</h1><span className="wizard-counter">{t("Paso")} {step + 1}  {t("de")} {steps.length}</span></div></div>
    <nav className="wizard-steps" aria-label={t("Pasos de creación")}><MovingIndicator active={String(step)} axis="horizontal"/>{steps.map((label,index) => <button type="button" key={label} className={index === step ? 'active' : index < step ? 'completed' : ''} aria-label={`Paso ${index+1}: ${label}`} aria-current={index === step ? 'step' : undefined} disabled={busy || index > furthestStep} onClick={() => moveStep(index)}><span aria-hidden="true">{index < step ? '✓' : String(index + 1).padStart(2,'0')}</span><strong>{translatedMessage(label)}</strong></button>)}</nav>
    <div className="form-layout wizard-layout"><form className="create-form wizard-form" onSubmit={e => { e.preventDefault(); advance() }}>
      <div className="wizard-panel" key={step} ref={wizardRef}>
      {step === 0 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Nombre e imagen")}</h2><p>{t("Así aparecerá en tu biblioteca.")}</p></div>
      <div className="identity-fields"><div className="image-selector"><div className="image-preview">{imagePreview ? <img src={imagePreview} alt={t("Vista previa de la imagen")}/> : <Icon name="server" size={28}/>}</div><button type="button" onClick={chooseImage} disabled={!isTauri()}>{t("Elegir imagen")}</button><p>{t("PNG, JPG o WebP · hasta 2 MB")}</p>{imageError && <p className="form-error" role="alert">{translatedMessage(imageError)}</p>}</div><div className="form-row"><label htmlFor="new-name">{t("Nombre del servidor")}</label><input id="new-name" autoFocus required maxLength={64} value={input.name} onChange={e => setInput({...input,name:e.target.value})} placeholder={t("Mi servidor")}/><p className="field-help">{t("Solo lo verás tú en este launcher.")}</p></div></div>
      </>}
      {step === 1 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Versión y tipo")}</h2><p>{t("Elige la versión de Minecraft que utilizarán tus jugadores.")}</p></div>
      <div className="form-pair version-step-grid"><div className="form-row"><label htmlFor="new-version">{t("Versión de Minecraft")}</label>{versionError ? <div className="form-error" role="alert">{t("No se pudo cargar el catálogo:")} {translatedMessage(versionError)} <button type="button" onClick={loadVersions}>{t("Reintentar")}</button></div> : <select id="new-version" className="minecraft-version-list" size={6} aria-describedby="new-version-help" disabled={versionLoading} required value={input.version} onChange={e => setInput({...input,version:e.target.value})}><option value="">{versionLoading ? t("Cargando versiones…") : t("Selecciona una versión")}</option>{versions.map(v => <option key={v.id} value={v.id}>{v.id}{v.kind==='snapshot'?' · Experimental':''}</option>)}</select>}<p id="new-version-help" className="field-help">{!versionLoading&&!versionError?t("{0} versiones en el catálogo · Desplázate para ver las anteriores.", versions.length):t("Versiones del catálogo oficial del tipo seleccionado.")}</p></div><fieldset className="version-platform-picker"><legend>{t("Tipo de servidor")}</legend><div>{(['vanilla','paper','fabric','neoforge'] as const).map(platform=><label key={platform}><input type="radio" name="new-platform" value={platform} checked={input.platform===platform} onChange={()=>{setVersionLoading(true);setVersionError(null);if(!['vanilla','fabric'].includes(platform))setExperimental(false);setInput(old=>({...old,platform}))}}/><span><strong>{platformName(platform)}</strong><small>{({vanilla:t("Minecraft original"),paper:t("Servidor con plugins"),fabric:t("Mods de Fabric"),neoforge:t("Mods de NeoForge")})[platform]}</small></span></label>)}</div></fieldset></div><p className="field-help type-help">{input.platform === 'neoforge' ? t("NeoForge permite mods de servidor. Incluye la edición histórica para Minecraft 1.20.1 y el catálogo moderno desde 1.20.2. No hay ediciones anteriores; usa mods compatibles con la versión elegida.") : input.platform === 'fabric' ? t("Fabric permite mods de servidor. Se elegirá automáticamente un loader estable compatible.") : input.platform === 'paper' ? t("Paper admite plugins y opciones extra de rendimiento.") : t("Vanilla ofrece la experiencia original de Minecraft.")}</p>
      {['vanilla','fabric'].includes(input.platform)&&<div className="experimental-versions"><label><input type="checkbox" checked={experimental} disabled={busy} onChange={event=>{setVersionLoading(true);setExperimental(event.target.checked)}}/><span>{t("Mostrar versiones experimentales")}</span></label>{experimental&&<p className="field-help">{t("Incluye snapshots y versiones de prueba del catálogo oficial. Pueden fallar y cambiar el formato de los mundos; crea una instancia separada para probarlas.")}</p>}</div>}
      </>}
      {step === 2 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Elige un mundo")}</h2><p>{t("Genera uno nuevo o importa un mundo de Minecraft Java.")}</p></div>
      <div className="world-choice"><label><input type="radio" name="world-mode" checked={worldMode === 'new'} onChange={() => setWorldMode('new')}/><span><strong>{t("Mundo nuevo")}</strong><small>{t("Minecraft lo generará en el primer inicio.")}</small></span></label><label><input type="radio" name="world-mode" checked={worldMode === 'existing'} onChange={() => setWorldMode('existing')}/><span><strong>{t("Usar un mundo existente")}</strong><small>{t("Se copiará a este servidor; el original se conserva.")}</small></span></label></div>
      {worldMode === 'new' && <div className="world-generation"><label htmlFor="new-world-seed"><strong>{t("Semilla del mundo")}</strong><span className="field-optional">{t("Opcional")}</span></label><input id="new-world-seed" type="text" value={input.seed??''} disabled={busy} aria-invalid={invalidSeed} aria-describedby="new-world-seed-help" placeholder={t("Aleatoria si la dejas vacía")} onChange={event=>setInput({...input,seed:event.target.value})}/><p id="new-world-seed-help" className={invalidSeed?'form-error':'field-help'}>{invalidSeed?t("Usa hasta 1024 bytes, sin saltos de línea ni caracteres de control."):t("Número o texto. Se usará al generar el mundo en su primer inicio.")}</p></div>}
      {worldMode === 'existing' && <div className="world-choice-source"><button type="button" onClick={chooseWorld}>{t("Elegir carpeta del mundo")}</button>{worldSource && <div><strong>{worldSource.suggestedName}</strong><span>{fmtBytes(worldSource.sizeBytes)} · {worldSource.path}</span></div>}{worldError && <p className="form-error" role="alert">{translatedMessage(worldError)}</p>}</div>}
      </>}
      {step === 3 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Capacidad del servidor")}</h2><p>{t("Reserva memoria para el sistema y para jugar.")}</p></div>
      <div className="capacity-grid"><SliderField id="new-memory-min" label="RAM inicial" value={input.memoryMinMb} min={512} max={Math.max(input.memoryMinMb,input.memoryMaxMb)} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMinMb:value,memoryMaxMb:value >= 512 && value <= 65536 ? Math.max(input.memoryMaxMb,value) : input.memoryMaxMb})}/><SliderField id="new-memory-max" label={t("RAM máxima")} value={input.memoryMaxMb} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setInput({...input,memoryMaxMb:value,memoryMinMb:value >= 512 && value <= 65536 ? Math.min(input.memoryMinMb,value) : input.memoryMinMb})}/><SliderField id="new-players" label={t("Jugadores máximos")} value={input.maxPlayers} min={1} max={Math.max(100,input.maxPlayers)} maxInput={1000} step={1} onChange={value => setInput({...input,maxPlayers:value})}/><SliderField id="new-storage" label={t("Objetivo de espacio")} value={input.storageTargetGb} min={1} max={Math.max(200,input.storageTargetGb)} maxInput={2048} step={1} unit="GB" onChange={value => setInput({...input,storageTargetGb:value})}/></div><p className="field-help">{systemMemory ? t("Tu equipo tiene {0} de RAM.", fmtBytes(systemMemory)) : t("Deja memoria libre para el sistema y para jugar.")}  {t("El objetivo de espacio sirve para avisarte; no impone una cuota al disco.")}</p>{systemMemoryMb > 0 && input.memoryMaxMb > systemMemoryMb * .75 && <p className="inline-note" role="status">{t("La RAM máxima supera el 75 % de la memoria física. Deja recursos para el sistema y para jugar.")}</p>}
      <details className="advanced"><summary>{t("Opciones de red")}</summary><div className="form-row"><label htmlFor="new-port">{t("Puerto del servidor")}</label><input id="new-port" type="number" min="1" max="65535" value={input.port} onChange={e => setInput({...input,port:Number(e.target.value)})}/></div></details>
      {invalidCapacity && <p className="form-error" role="alert">{t("Revisa RAM, jugadores, espacio y puerto antes de continuar.")}</p>}
      </>}
      {step === 4 && <>
      <div className="wizard-panel-heading"><h2 tabIndex={-1}>{t("Revisar y crear")}</h2><p>{t("Comprueba los datos antes de instalar.")}</p></div>
      <div className="wizard-review"><div><span>{t("Identidad")}</span><strong>{input.name.trim() || t("Sin nombre")}</strong><button type="button" className="text-button" onClick={() => moveStep(0)}>{t("Editar")}</button></div><div><span>{t("Versión y tipo")}</span><strong>{input.version || t("Sin versión")}{versions.find(version=>version.id===input.version)?.kind==='snapshot'?' · Experimental':''} · {platformName(input.platform)}</strong><button type="button" className="text-button" onClick={() => moveStep(1)}>{t("Editar")}</button></div><div><span>{t("Mundo")}</span><strong>{worldMode === 'existing' ? t("Copia de {0}", worldSource?.suggestedName || t("mundo pendiente")) : `Mundo nuevo · ${input.seed?t("Semilla elegida"):t("Semilla aleatoria")}`}</strong><button type="button" className="text-button" onClick={() => moveStep(2)}>{t("Editar")}</button></div><div><span>{t("Recursos")}</span><strong>{fmtBytes(input.memoryMinMb * 1024 * 1024)}–{fmtBytes(input.memoryMaxMb * 1024 * 1024)} · {input.maxPlayers}  {t("jugadores ·")} {input.storageTargetGb} GB</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>{t("Editar")}</button></div><div><span>{t("Puerto")}</span><strong>{input.port}</strong><button type="button" className="text-button" onClick={() => moveStep(3)}>{t("Editar")}</button></div></div>
      {worldMode==='new'&&input.seed&&<div className="wizard-seed-review"><span>{t("Semilla")}</span><code>{input.seed}</code></div>}
      <p className="wizard-final-note">{t("La instalación descargará Java y los archivos del servidor. En la primera descarga te pediremos aceptar la EULA de Minecraft. La decisión se recordará en este equipo.")}</p>
      </>}
      </div>
      <div className="form-actions wizard-actions"><div>{step > 0 && <button type="button" disabled={busy} onClick={() => moveStep(step - 1)}>{t("Anterior")}</button>}<button type="submit" className="primary-button" disabled={busy || !canAdvance || (step === steps.length - 1 && !readyToCreate)}>{busy ? t("Preparando…") : step === steps.length - 1 ? t("Crear e instalar") : t("Continuar")}<Icon name="arrow" size={16}/></button></div></div>
    </form><aside className="form-aside wizard-preview"><div className="wizard-preview-art" aria-hidden="true"><img src={worldIllustration} alt=""/></div><div className="aside-title">{t("Tu configuración")}</div><div className="aside-server"><div className="image-preview small">{imagePreview ? <img src={imagePreview} alt=""/> : <Icon name="server" size={22}/>}</div><div><strong>{input.name.trim() || t("Nuevo servidor")}</strong><span>{platformName(input.platform)} · {input.version || t("Versión pendiente")}</span></div></div><dl><div><dt>{t("Mundo")}</dt><dd>{worldMode === 'existing' ? worldSource?.suggestedName || t("Por elegir") : t("Nuevo")}</dd></div><div><dt>{t("Memoria")}</dt><dd><AnimatedValue value={input.memoryMaxMb * 1024 * 1024} format={fmtBytes}/></dd></div><div><dt>{t("Jugadores")}</dt><dd>{t("Hasta")} <AnimatedValue value={input.maxPlayers}/></dd></div><div><dt>{t("Objetivo de espacio")}</dt><dd><AnimatedValue value={input.storageTargetGb}/> GB</dd></div><div><dt>{t("Puerto")}</dt><dd>{input.port}</dd></div></dl><p>{t("El servidor se ejecutará en este ordenador. Java y los archivos se descargarán al crearlo.")}</p></aside></div>
  </div>
}

