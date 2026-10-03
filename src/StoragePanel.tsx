import {useEffect,useRef,useState} from 'react'
import {api,type Server,type StorageUsage} from './api'
import {SliderField} from './SliderField'

const bytes=(value:number)=>value>=1024**3?`${(value/1024**3).toFixed(2)} GB`:value>=1024**2?`${(value/1024**2).toFixed(1)} MB`:`${Math.round(value/1024)} KB`
const errorText=(error:unknown)=>error instanceof Error?error.message:String(error)
export function StoragePanel({server,busy}:{server:Server;busy:boolean}){
  const [usage,setUsage]=useState<StorageUsage|null>(null)
  const [loading,setLoading]=useState(true)
  const [saving,setSaving]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const [draft,setDraft]=useState(90)
  const [saved,setSaved]=useState<number|null>(null)
  const [notice,setNotice]=useState<string|null>(null)
  const mounted=useRef(true)
  useEffect(()=>{
    mounted.current=true;let active=true
    void api.storageUsage(server.id).then(value=>{if(active){setUsage(value);setDraft(value.warningPercent);setSaved(value.warningPercent)}}).catch(error=>{if(active)setError(errorText(error))}).finally(()=>{if(active)setLoading(false)})
    return()=>{active=false;mounted.current=false}
  },[server.id])
  const refresh=async()=>{
    setLoading(true);setError(null)
    try{const value=await api.storageUsage(server.id);if(mounted.current){setUsage(value);setSaved(value.warningPercent)}}catch(error){if(mounted.current)setError(errorText(error))}finally{if(mounted.current)setLoading(false)}
  }
  const save=async()=>{
    if(!Number.isInteger(draft)||draft<1||draft>100)return
    setSaving(true);setError(null);setNotice(null)
    try{await api.saveStorageWarning(server.id,draft);if(mounted.current){setSaved(draft);setUsage(value=>value?{...value,warningPercent:draft}:value);setNotice(`Aviso configurado al ${draft} % del objetivo.`)}}catch(error){if(mounted.current)setError(errorText(error))}finally{if(mounted.current)setSaving(false)}
  }
  const target=server.storageTargetGb*1024**3
  const warning=usage&&usage.serverBytes>=target*usage.warningPercent/100
  const categories=usage?[{label:'Mundos detectados',kind:'worlds',value:usage.worldsBytes},{label:'Mods y plugins',kind:'content',value:usage.contentBytes},{label:'Logs',kind:'logs',value:usage.logsBytes},{label:'Otros archivos',kind:'other',value:usage.otherBytes}]:[]
  return <section className="storage-usage" aria-labelledby="storage-usage-title" aria-busy={loading||saving}>
    <div className="section-heading"><div><h2 id="storage-usage-title">Espacio de {server.name}</h2><p>Tamaño de archivos en este equipo, separado por contenido.</p></div><button disabled={loading||saving||busy} onClick={()=>void refresh()}>{loading?'Midiendo…':'Actualizar medición'}</button></div>
    {error&&<p className="form-error" role="alert">No se pudo actualizar el almacenamiento: {error}</p>}
    {!usage?loading&&<p role="status">Midiendo los archivos del servidor y sus copias…</p>:<>
      <div className="storage-total"><strong>{bytes(usage.serverBytes)}</strong><span>en el servidor · objetivo {server.storageTargetGb} GB</span></div>
      <div className="storage-composition" aria-hidden="true">{categories.filter(item=>item.value>0).map(item=><span key={item.kind} className={item.kind} style={{flexBasis:`${item.value/usage.serverBytes*100}%`}}/>)}</div>
      <dl className="storage-breakdown">{categories.map(item=><div key={item.kind}><dt><span className={`storage-key ${item.kind}`} aria-hidden="true"/>{item.label}</dt><dd>{bytes(item.value)}</dd></div>)}<div className="storage-backups"><dt>Copias y archivos en preparación</dt><dd>{bytes(usage.backupBytes)}</dd></div></dl>
      <dl className="data-list storage-volumes"><div><dt>Libre en el disco del servidor</dt><dd>{usage.serverVolume?`${bytes(usage.serverVolume.availableBytes)} / ${bytes(usage.serverVolume.totalBytes)}`:'No disponible'}</dd></div><div><dt>Libre en el disco de las copias</dt><dd>{usage.backupVolume?`${bytes(usage.backupVolume.availableBytes)} / ${bytes(usage.backupVolume.totalBytes)}`:'No disponible'}</dd></div></dl>
      {warning&&<p className="storage-threshold-warning" role="status">El servidor ha alcanzado el {usage.warningPercent} % del objetivo de espacio. Revisa los mundos, el contenido o los logs.</p>}
      <p className="field-help">Medición de {new Date(usage.sampledAt).toLocaleTimeString()}. El tamaño puede variar mientras el servidor está activo.{usage.skippedLinks>0&&` No se incluyeron ${usage.skippedLinks} enlaces simbólicos.`}</p>
    </>}
    <fieldset className="storage-warning-controls" disabled={loading||saving||busy||saved===null}><SliderField id="storage-warning-percent" label="Avisar al alcanzar" value={draft} min={1} max={100} maxInput={100} step={1} unit="%" onChange={setDraft}/><p className="field-help">Porcentaje del objetivo de {server.storageTargetGb} GB. Activa un aviso; no limita el disco ni elimina archivos.</p><button disabled={draft===saved||!Number.isInteger(draft)||draft<1||draft>100} onClick={()=>void save()}>{saving?'Guardando…':'Guardar aviso'}</button>{notice&&<p className="field-help" role="status">{notice}</p>}</fieldset>
  </section>
}
