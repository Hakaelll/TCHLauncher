import {settingsMatch,propertyAdditionError} from './settingsSearch'
import './server-settings.css'
import {isCapacityValid} from './capacity'
import {useEffect,useRef,useState} from 'react'
import {t,translatedMessage} from './locale.ts'
import type {Server,Progress} from './api'
import {Icon,ServerArtwork,fmtBytes} from './ServerPresentation'
import {SliderField} from './SliderField'
import {MemoryAdvice} from './MemoryAdvice'
import {SettingsPropertyGroups,knownSettings,invalidNumericSetting} from './SettingsProperties'
import {RuntimePanel} from './RuntimePanel'
import {JvmPanel} from './JvmPanel'
import {MinecraftUpdatePanel} from './MinecraftUpdatePanel'
import {LoaderPanel} from './LoaderPanel'
import {TasksPanel} from './TasksPanel'

export function ServerSettings({onConsole,onRuntimeChanged,onPrepareRuntime,server,props,savedProps,setProps,propertiesLoaded,propertiesLoading,propertiesError,propertiesDirty,onReloadProperties,memory,setMemory,storageTarget,setStorageTarget,systemMemory,busy,progress,onSave,onDiscard,onPickImage,onClearImage,onDelete}: {onConsole:()=>void;onRuntimeChanged:()=>Promise<void>;onPrepareRuntime:(reinstall:boolean)=>void;server:Server;props:Record<string,string>;savedProps:Record<string,string>;setProps:(p:Record<string,string>)=>void;propertiesLoaded:boolean;propertiesLoading:boolean;propertiesError:string|null;propertiesDirty:boolean;onReloadProperties:()=>void;memory:{min:number;max:number};setMemory:(v:{min:number;max:number})=>void;storageTarget:number;setStorageTarget:(value:number)=>void;systemMemory:number;busy:boolean;progress:Progress|null;onSave:()=>void;onDiscard:()=>void;onPickImage:()=>void;onClearImage:()=>void;onDelete:()=>void}) {
  const [query,setQuery]=useState('')
  const fields=useRef<HTMLFieldSetElement>(null)
  const [invalidKey,setInvalidKey]=useState<string|null>(null)
  useEffect(()=>{
    if(!invalidKey)return
    const invalid=invalidNumericSetting(props,savedProps)
    const input=fields.current?.querySelector<HTMLInputElement>(`[data-setting-key="${invalidKey}"]`)
    if(invalid?.key===invalidKey&&input){input.setCustomValidity(t('Revisa {0}: usa un número entero entre {1} y {2}.',invalid.label,invalid.min,invalid.max));input.reportValidity()}
    setInvalidKey(null)
  },[invalidKey,props,savedProps])
  const save=()=>{
    const invalid=invalidNumericSetting(props,savedProps)
    if(invalid){setQuery(invalid.key);setInvalidKey(invalid.key);return}
    onSave()
  }
  const stopped=['stopped','crashed','error'].includes(server.state)
  const [newKey,setNewKey] = useState('')
  const [newValue,setNewValue] = useState('')
  const additionError=propertyAdditionError(newKey,newValue,props)
  const systemMemoryMb = Math.floor(systemMemory / (1024 * 1024))
  const memorySliderMax = Math.min(65536,Math.max(memory.max,systemMemoryMb || 16384))
  const maxPlayers = Number(props['max-players'] || 20)
  const invalidCapacity = !isCapacityValid({memoryMinMb:memory.min,memoryMaxMb:memory.max,maxPlayers,storageTargetGb:storageTarget})
  return <section className="settings-view"><div className="section-heading settings-save-heading"><div><h2>{t("Configuración")}</h2><p>{t("Personaliza la apariencia y los ajustes del servidor.")}</p></div><div className="settings-save-actions">{propertiesDirty&&<button disabled={busy||propertiesLoading||!propertiesLoaded||!stopped} onClick={()=>{onDiscard();setNewKey('');setNewValue('')}}>{t("Descartar cambios")}</button>}<button className="primary-button" disabled={busy || !propertiesLoaded || propertiesLoading || !!propertiesError || invalidCapacity || !stopped} onClick={save}>{t("Guardar cambios")} <Icon name="check" size={16}/></button></div></div>
    <h3>{t("Imagen del servidor")}</h3><div className="settings-image"><ServerArtwork server={server}/><div><strong>{server.name}</strong><p>{t("La misma imagen en el launcher y en la lista de Minecraft. Se adapta a 64 × 64. PNG, JPG o WebP, hasta 2 MB.")}</p><div className="settings-image-actions"><button disabled={busy || !stopped} onClick={onPickImage}>{server.imageData ? t("Cambiar imagen") : t("Elegir imagen")}</button>{server.imageData && <button className="text-button" disabled={busy || !stopped} onClick={onClearImage}>{t("Quitar imagen")}</button>}</div></div></div>
    {!stopped && <p className="inline-note">{t("Detén el servidor para editar su configuración.")}</p>}
    <div className={`properties-load-state ${propertiesError?'error':''}`} role={propertiesError?'alert':'status'}><div><strong>{propertiesLoading?t("Consultando propiedades…"):propertiesError?t("Propiedades no disponibles"):!propertiesLoaded?t("Cargando propiedades…"):propertiesDirty?t("Cambios sin guardar"):t("Propiedades cargadas")}</strong><p>{propertiesError?`${translatedMessage(propertiesError)}${propertiesLoaded?' '+t('Se conservan los últimos valores leídos.'):''}`:propertiesDirty?t("Las actualizaciones automáticas conservarán tus cambios."):t("Los cambios se aplican en el próximo inicio del servidor.")}</p></div><button disabled={busy||propertiesLoading||propertiesDirty} onClick={onReloadProperties}>{t("Recargar propiedades")}</button></div>
    {propertiesLoaded&&<div className="settings-search"><label><Icon name="search" size={16}/><input type="search" maxLength={200} aria-label={t("Buscar ajustes")} placeholder={t("Buscar por nombre o propiedad…")} value={query} onChange={event=>setQuery(event.target.value)}/></label>{query&&<button className="text-button" onClick={()=>setQuery('')}>{t("Borrar búsqueda")}</button>}</div>}
    {propertiesLoaded&&<fieldset ref={fields} className="settings-edit-fields" aria-label={t("Propiedades y capacidad")} disabled={busy || propertiesLoading || !!propertiesError || !stopped}><h3>{t("Capacidad")}</h3><div className="capacity-grid settings-capacity"><SliderField id="settings-memory-min" label="RAM inicial" value={memory.min} min={512} max={Math.max(memory.min,memory.max)} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,min:value,max:value >= 512 && value <= 65536 ? Math.max(memory.max,value) : memory.max})}/><SliderField id="settings-memory-max" label={t("RAM máxima")} value={memory.max} min={512} max={memorySliderMax} maxInput={65536} step={256} unit="MB" onChange={value => setMemory({...memory,max:value,min:value >= 512 && value <= 65536 ? Math.min(memory.min,value) : memory.min})}/><SliderField id="settings-players" label={t("Jugadores máximos")} value={maxPlayers} min={1} max={Math.max(100,maxPlayers)} maxInput={1000} step={1} onChange={value => setProps({...props,'max-players':String(value)})}/><SliderField id="settings-storage" label={t("Objetivo de espacio")} value={storageTarget} min={1} max={Math.max(200,storageTarget)} maxInput={2048} step={1} unit="GB" onChange={setStorageTarget}/></div><MemoryAdvice physicalBytes={systemMemory} modded={server.platform==='fabric'||server.platform==='neoforge'} maxPlayers={maxPlayers}/><p className="field-help">{systemMemory ? t("Memoria física: {0}.", fmtBytes(systemMemory)) : ''}  {t("El objetivo de espacio solo activa avisos; no impone una cuota al disco.")}</p>{systemMemoryMb > 0 && memory.max > systemMemoryMb * .75 && <p className="inline-note" role="status">{t("La RAM máxima supera el 75 % de la memoria física.")}</p>}{invalidCapacity && <p className="form-error" role="alert">{t("Revisa los límites de RAM, jugadores y espacio antes de guardar.")}</p>}
    <h3>{t("Propiedades del servidor")}</h3><p className="field-help">{t("Configura el juego antes del primer arranque. Los valores guardados se conservan; las opciones ausentes usan valores iniciales al guardar.")}</p><SettingsPropertyGroups values={props} onChange={setProps} version={server.version} query={query}/>
    <details className="advanced-properties" open={query.trim()?true:undefined}><summary>{t("Otras propiedades")}</summary><div className="settings-fields">{Object.entries(props).filter(([key]) => key !== 'max-players' && !knownSettings.has(key) && settingsMatch(query,key)).map(([key,value]) => <label className="setting-line" key={key}><span>{key}</span><input value={value} onChange={e => setProps({...props,[key]:e.target.value})}/></label>)}</div><div className="add-property"><input aria-describedby={additionError&&(newKey||newValue)?"add-property-error":undefined} aria-invalid={!!newKey&&!!propertyAdditionError(newKey,'',props)} aria-label={t("Nombre de propiedad")} placeholder={t("Nombre de propiedad")} value={newKey} onChange={e => setNewKey(e.target.value)}/><input aria-describedby={additionError&&(newKey||newValue)?"add-property-error":undefined} aria-label={t("Valor")} placeholder={t("Valor")} value={newValue} onChange={e => setNewValue(e.target.value)}/><button disabled={!!additionError} onClick={() => { if (!propertyAdditionError(newKey,newValue,props)) { setProps({...props,[newKey.trim()]:newValue}); setNewKey(''); setNewValue('') } }}>{t("Añadir")}</button></div>{additionError&&(newKey||newValue)&&<p id="add-property-error" className="form-error" role="status">{t(additionError)}</p>}</details>
    </fieldset>}<section className="settings-maintenance"><h3>{t("Java y mantenimiento")}</h3>
    <RuntimePanel key={server.id} onChanged={onRuntimeChanged} server={server} busy={busy} onPrepare={onPrepareRuntime}/>
    <JvmPanel key={`jvm-${server.id}`} server={server} busy={busy}/>
    <MinecraftUpdatePanel key={`minecraft-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged} currentProgress={progress} onConsole={onConsole}/>
    <LoaderPanel key={`loader-${server.id}`} server={server} busy={busy} onChanged={onRuntimeChanged}/>
    <TasksPanel key={`tasks-${server.id}`} id={server.id}/>
    </section><div className="delete-server-section"><div><h3>{t("Eliminar servidor")}</h3><p>{t("Quita este servidor de la biblioteca. Si sus archivos están guardados aquí, también se eliminarán junto con sus copias de seguridad.")}</p></div><button className="danger-outline-button" disabled={busy || !stopped} onClick={onDelete}>{t("Eliminar servidor")}</button></div>
  </section>
}
