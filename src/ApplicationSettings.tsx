import {errorText} from './format'
import {translatedMessage,localizedStatic,t} from './locale.ts'
import {useRef,useState} from 'react'
import {api,type ApplicationInfo,type Progress,type Server} from './api'
import type {Appearance} from './appearance'
import {AppearanceSettings,DiagnosticsSettings} from './AppearanceSettings'
import {RuntimePanel} from './RuntimePanel'
import {JvmPanel} from './JvmPanel'
import {NetworkPanel} from './NetworkPanel'
import {BackupPolicyPanel} from './BackupPolicyPanel'
import {StartupPanel} from './StartupPanel'
import {StoragePanel} from './StoragePanel'
import {TasksPanel} from './TasksPanel'
import {InstallProgress} from './InstallProgress'
import {MovingIndicator} from './motion'
import {isTaskActive} from './taskState'
import launcherEmblem from './assets/launcher-logo.png'
import {useServerRead} from './useServerRead'
import {acknowledgedAction} from './acknowledgedAction'

const categories=localizedStatic({general:'General',java:'Java',storage:t("Almacenamiento"),network:t("Red"),backups:t("Copias"),advanced:t("Avanzado"),about:t("Información")} as const)
type Category=keyof typeof categories
const locations=localizedStatic({library:'Biblioteca',servers:'Servidores guardados',runtimes:'Java administrado',backups:'Copias de seguridad'})
const INFO_SCOPE='application-info'
const readInfo=()=>api.applicationInfo()

type Props={appearance:Appearance;saved:boolean;onAppearance:(value:Appearance)=>void;servers:Server[];selectedId:string|null;onSelect:(id:string)=>void;busy:boolean;progress:Record<string,Progress>;onPrepare:(id:string,reinstall:boolean)=>void;onChanged:()=>Promise<void>;onOpenServer:(server:Server,section:'backups'|'settings')=>void;onDismiss:(progress:Progress)=>void}
export function ApplicationSettings(props:Props){
  const [category,setCategory]=useState<Category>('general')
  const {data:info,error:infoError,loading:infoLoading,refresh:refreshInfo}=useServerRead(INFO_SCOPE,readInfo)
  const [opening,setOpening]=useState<string|null>(null)
  const [folderError,setFolderError]=useState<string|null>(null)
  const [working,setWorking]=useState(false)
  const content=useRef<HTMLDivElement>(null)
  const server=props.servers.find(item=>item.id===props.selectedId)??props.servers[0]??null
  const task=server?props.progress[server.id]??null:null
  const busy=props.busy||working||!!task&&isTaskActive(task)
  const scoped=['java','storage','network','backups','advanced'].includes(category)
  const open=async(key:ApplicationInfo['locations'][number]['key']|'server')=>{
    if(opening)return
    setOpening(key);setFolderError(null)
    try{await acknowledgedAction(()=>api.openApplicationFolder(key,key==='server'?server?.id??null:null),()=>refreshInfo(INFO_SCOPE))}catch(error){setFolderError(errorText(error))}finally{setOpening(null)}
  }
  const selectCategory=(value:Category)=>{setCategory(value);setFolderError(null);content.current?.closest('.main')?.scrollTo({top:0,left:0})}
  const infoFailure=infoError&&<div className="panel-read-state failed"><p className="form-error" role="alert">{translatedMessage(infoError)}</p><button disabled={infoLoading} onClick={()=>void refreshInfo(INFO_SCOPE).catch(()=>{})}>{infoLoading?t('Consultando…'):t('Reintentar lectura')}</button></div>
  return <section className="application-settings" aria-labelledby="application-settings-title">
    <header className="application-settings-heading"><h1 id="application-settings-title">{t("Ajustes")}</h1><p>{t("Preferencias del launcher y herramientas de tu biblioteca.")}</p></header>
    <nav className="server-sections settings-categories" aria-label={t("Categorías de ajustes")}>
      <MovingIndicator active={category} axis="both" className="section-active-indicator"/>
      {(Object.keys(categories) as Category[]).map(key=><button key={key} disabled={working} aria-current={category===key?'page':undefined} onClick={()=>selectCategory(key)}>{categories[key]}</button>)}
    </nav>
    <div ref={content} className="application-settings-content">
      {scoped&&<div className="settings-server-context"><label htmlFor="settings-server">{t("Servidor")}<select id="settings-server" value={server?.id??''} disabled={!server||busy} onChange={event=>props.onSelect(event.target.value)}>{!server&&<option value="">{t("Sin servidores")}</option>}{props.servers.map(item=><option key={item.id} value={item.id}>{item.name} · {item.version}</option>)}</select></label><p>{t("Los cambios de esta sección se aplican al servidor seleccionado.")}</p></div>}
      {category==='general'&&<><AppearanceSettings embedded value={props.appearance} saved={props.saved} onChange={props.onAppearance}/><section className="settings-startup"><h2>{t("Inicio automático")}</h2><StartupPanel servers={props.servers} onBusy={setWorking}/></section></>}
      {category==='java'&&(server?<>
        {task&&(task.kind==='runtime'||task.kind==='runtime_reinstall')&&<InstallProgress key={task.taskId} progress={task} busy={busy} retryLabel={t("Revisar Java")} onRetry={()=>props.onPrepare(server.id,false)} onDismiss={()=>props.onDismiss(task)}/>}
        <RuntimePanel key={`runtime-${server.id}`} server={server} busy={busy} onPrepare={reinstall=>props.onPrepare(server.id,reinstall)} onChanged={props.onChanged}/><JvmPanel key={`jvm-${server.id}`} server={server} busy={busy}/>
      </>:<p className="empty-inline">{t("Crea o importa un servidor para configurar su runtime de Java.")}</p>)}
      {category==='storage'&&<section aria-labelledby="settings-storage-title"><h2 id="settings-storage-title">{t("Archivos de la aplicación")}</h2><p className="muted">{t("La biblioteca, Java y las copias se guardan en este equipo. Los servidores importados sin copiar conservan su carpeta original.")}</p>
        {infoError?infoFailure:!info?<p role="status">{t("Consultando ubicaciones…")}</p>:<dl className="settings-locations">{info.locations.map(location=><div key={location.key}><dt>{locations[location.key]}</dt><dd><code>{location.path}</code>{!location.exists&&<small>{t("Se creará cuando sea necesaria.")}</small>}</dd><button disabled={!!opening} onClick={()=>void open(location.key)}>{opening===location.key?t("Abriendo…"):t("Abrir carpeta")}</button></div>)}</dl>}
        {server&&<StoragePanel key={server.id} server={server} busy={busy}/>}{server&&<div className="settings-storage-server"><h3>{server.name}</h3><code>{server.directory}</code><p className="muted">{t("Objetivo de espacio:")} {server.storageTargetGb}  {t("GB. Activa avisos; no limita el disco.")}</p><div className="settings-inline-actions"><button disabled={!!opening} onClick={()=>void open('server')}>{t("Abrir carpeta del servidor")}</button><button onClick={()=>props.onOpenServer(server,'settings')}>{t("Cambiar objetivo de espacio")}</button></div></div>}{folderError&&<p className="form-error" role="alert">{translatedMessage(folderError)}</p>}
      </section>}
      {category==='network'&&(server?<NetworkPanel key={server.id} server={server}/>:<p className="empty-inline">{t("Crea o importa un servidor para comprobar su conexión.")}</p>)}
      {category==='backups'&&(server?<section><div className="section-heading"><div><h2>{t("Copias del servidor")}</h2><p>{t("Revisa la retención y accede a las copias guardadas.")}</p></div><button onClick={()=>props.onOpenServer(server,'backups')}>{t("Ver copias")}</button></div><BackupPolicyPanel key={server.id} id={server.id} disabled={busy||!['stopped','crashed','error'].includes(server.state)} onChanged={props.onChanged}/></section>:<p className="empty-inline">{t("Crea o importa un servidor para gestionar sus copias.")}</p>)}
      {category==='advanced'&&<><DiagnosticsSettings/>{server&&<TasksPanel key={server.id} id={server.id}/>}</>}
      {category==='about'&&<section className="settings-about" aria-labelledby="settings-about-title"><div className="settings-about-brand"><img src={launcherEmblem} alt=""/><div><h2 id="settings-about-title">TCHLauncher</h2><p>{t("Gestor local de servidores de Minecraft.")}</p></div></div>{infoError?infoFailure:!info?<p role="status">{t("Consultando la aplicación…")}</p>:<dl className="data-list"><div><dt>{t("Versión instalada")}</dt><dd>{info.version}</dd></div><div><dt>{t("Sistema")}</dt><dd>{info.os} · {info.architecture}</dd></div><div><dt>{t("EULA de Minecraft")}</dt><dd>{info.eulaRecorded?t("Aceptación guardada en esta biblioteca"):t("Sin aceptación guardada")}</dd></div></dl>}<p className="muted">{t("Minecraft es una marca de Mojang y Microsoft. TCHLauncher es un proyecto independiente.")}</p></section>}
    </div>
  </section>
}
