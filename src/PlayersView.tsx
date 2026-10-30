import {translatedMessage,localizedStatic,t} from './locale.ts'
import { MovingIndicator, useListMotion } from './motion'
import {OperatorSettings} from './OperatorSettings'
import {OperatorGrant} from './OperatorGrant'
import {PlayerHead} from './PlayerHead'
import {identityKey,isOfflineUuid} from './playerIdentity'
import { useEffect, useRef, useState, type FormEvent } from 'react'
import { type PlayerIdentity, type PlayersSnapshot, type Server, type WhitelistSelection, type WhitelistOutcome, type AccessList, type AccessRemoval } from './api'
import {WhitelistImportReview} from './WhitelistImportReview'
import {playerActions,playerActionErrors,type PlayerAction as Action} from './playerActions'
import {playerWindow} from './playerWindow'
import {AuthenticationPanel} from './AuthenticationPanel'
import {BanDetails} from './BanDetails'
import {acknowledgedAction} from './acknowledgedAction'

type Group = 'online' | 'known' | 'whitelist' | 'operators' | 'banned'
const accessLabels:Record<AccessList,string>=localizedStatic({whitelist:'Lista blanca',operators:'Operadores',banned_players:'Jugadores bloqueados'})

const groups: { id: Group; label: string; count: (players: PlayersSnapshot) => number }[] = localizedStatic([
  { id: 'online', label: 'Conectados', count: players => players.online.length },
  { id: 'known', label: 'Conocidos', count: players => players.known.length },
  { id: 'whitelist', label: 'Lista blanca', count: players => players.whitelisted.length },
  { id: 'operators', label: 'Operadores', count: players => players.operators.length },
  { id: 'banned', label: 'Bloqueados', count: players => players.bannedPlayers.length + players.bannedIps.length },
])

function PlayerRow({name,uuid,detail,action,disabled,onAction}: {name:string;uuid?:string|null;detail?:string;action?:string;disabled:boolean;onAction?:()=>void}) {
  return <div className="player-row" data-layout-id={uuid || name}><PlayerHead name={name} uuid={uuid}/><div className="player-row-info"><strong>{name}</strong>{detail && <small title={translatedMessage(detail)}>{translatedMessage(detail)}</small>}</div>{action && <button className="text-button" disabled={disabled} onClick={onAction}>{translatedMessage(action)}</button>}</div>
}

export function PlayersView({server,players,loading,readError,busy,onAction,onToggleWhitelist,onImportWhitelist,onRemoveIdentity,onRemoveIp,onRefresh}: {
  server: Server;
  players: PlayersSnapshot | null;
  loading:boolean;
  readError:string|null;
  busy: boolean;
  onAction: (action:string,target?:string,reason?:string) => Promise<boolean>;
  onToggleWhitelist: () => Promise<boolean>;
  onImportWhitelist: (selection:WhitelistSelection) => Promise<WhitelistOutcome|null>;
  onRemoveIdentity: (list:AccessList,uuid:string) => Promise<AccessRemoval|null>;
  onRemoveIp: (ip:string) => Promise<AccessRemoval|null>;
  onRefresh: () => Promise<void>;
}) {
  const [group,setGroup] = useState<Group>('online')
  const [query,setQuery]=useState('')
  const [page,setPage]=useState(0)
  const listRef=useRef<HTMLDivElement>(null)
  const targetRef=useRef<HTMLInputElement>(null)
  const reasonRef=useRef<HTMLInputElement>(null)
  const [action,setAction] = useState<Action>('whitelist_add')
  const [target,setTarget] = useState('')
  const [reason,setReason] = useState('')
  const [notice,setNotice] = useState<string|null>(null)
  const [attempted,setAttempted]=useState(false)
  const running = server.state === 'running'
  const stopped = ['stopped','crashed','error'].includes(server.state)
  const canSetWhitelist = running || stopped
  const identityScope=useRef({active:true,working:false})
  useEffect(()=>{const current={active:true,working:false};identityScope.current=current;return()=>{current.active=false}},[server.id])
  const removeIdentity=async(list:AccessList,uuid:string|null)=>{
    const current=identityScope.current
    if(!uuid||!stopped||busy||loading||readError||!players||current.working)return
    current.working=true;setNotice(null)
    try{
      const result=await onRemoveIdentity(list,uuid)
      if(!result||!current.active)return
      setNotice(result.removed?t('Se quitó el UUID {0} de {1}. Copia anterior: {2}',uuid,accessLabels[list],result.previousFile??'—'):t('Ese UUID ya no está en {0}. No se modificaron archivos.',accessLabels[list]))
      await onRefresh().catch(()=>{})
    }finally{current.working=false}
  }
  const authorizeKnown=async(player:PlayerIdentity)=>{
    const current=identityScope.current
    if(!player.uuid||!stopped||busy||loading||readError||!players||current.working)return
    current.working=true;setNotice(null)
    try{
      const result=await onImportWhitelist({kind:'identities',players:[{name:player.name,uuid:player.uuid}]})
      if(!result||!current.active)return
      if(result.mode==='commands')setNotice(t('Se enviaron {0} solicitudes de lista blanca. Minecraft actualizará la lista al procesarlas.',result.submitted))
      else if(!result.added)setNotice(t('Ese UUID ya está autorizado. No se modificaron archivos.'))
      else setNotice(result.previousFile?t('Se autorizó el UUID {0} para el próximo inicio. Copia anterior: {1}',player.uuid,result.previousFile):t('Se autorizó el UUID {0} para el próximo inicio.',player.uuid))
      await onRefresh().catch(()=>{})
    }finally{current.working=false}
  }
  const act = async (kind: string, name?: string, motive?: string) => {
    const current=identityScope.current
    if(!running||busy||current.working)return false
    current.working=true;setNotice(null)
    try{
      const result=await acknowledgedAction(()=>onAction(kind,name,motive),async success=>{
        if(!success||!current.active)return
        setNotice(t("{0}comando enviado. La lista se actualizará cuando el servidor lo procese.", name ? `${name}: ` : ''))
        await onRefresh()
      })
      return current.active&&result.value
    }finally{current.working=false}
  }
  const pardonIp=async(ip:string)=>{
    if(running){await act('pardon_ip',ip);return}
    const current=identityScope.current
    if(!stopped||busy||loading||readError||!players||current.working)return
    current.working=true;setNotice(null)
    try{
      const result=await onRemoveIp(ip)
      if(!result||!current.active)return
      setNotice(result.removed?t('Se perdonó la IP {0}. Copia anterior: {1}',ip,result.previousFile??'—'):t('Esa IP ya no está bloqueada. No se modificaron archivos.'))
      await onRefresh().catch(()=>{})
    }finally{current.working=false}
  }
  const submit = async (event: FormEvent) => {
    event.preventDefault()
    setAttempted(true)
    const errors=playerActionErrors(action,target,reason)
    if(!running||busy)return
    if(errors.target){targetRef.current?.focus();return}
    if(errors.reason){reasonRef.current?.focus();return}
    if (await act(action,target.trim(),reason.trim())) { setTarget(''); setReason('');setAttempted(false) }
  }
  const toggleWhitelist = async () => {
    const current=identityScope.current
    if(!canSetWhitelist||busy||loading||readError||!players||current.working)return
    current.working=true;setNotice(null)
    try{
      await acknowledgedAction(onToggleWhitelist,async success=>{
        if(!success||!current.active)return
        setNotice(running ? t("Comando enviado. La lista se actualizará cuando el servidor lo procese.") : t("Acceso actualizado para el próximo inicio."))
        await onRefresh()
      })
    }finally{current.working=false}
  }
  const items = players ?? { online:[],known:[],whitelisted:[],operators:[],bannedPlayers:[],bannedIps:[],maxPlayers:0,whitelistEnabled:false,enforceWhitelist:false,onlineMode:true }
  const operators=new Set(items.operators.flatMap(player=>player.uuid?[identityKey(player.uuid)]:[]))
  const authorized=new Set(items.whitelisted.flatMap(player=>player.uuid?[identityKey(player.uuid)]:[]))
  const errors=playerActionErrors(action,target,reason)
  const windows={
    online:playerWindow(items.online,query,player=>`${player.name} ${player.uuid??''}`,page),
    known:playerWindow(items.known,query,player=>`${player.name} ${player.uuid??''}`,page),
    whitelist:playerWindow(items.whitelisted,query,player=>`${player.name} ${player.uuid??''}`,page),
    operators:playerWindow(items.operators,query,player=>`${player.name} ${player.uuid??''}`,page),
    banned:playerWindow([...items.bannedPlayers.map(entry=>({type:'player' as const,entry})),...items.bannedIps.map(entry=>({type:'ip' as const,entry}))],query,item=>item.type==='player'?`${item.entry.name} ${item.entry.uuid??''}`:item.entry.ip,page),
  }
  const window=windows[group]
  const identityGroup=group==='whitelist'||group==='operators'||group==='banned'
  const listNotice=identityGroup||group==='known'
  useListMotion(listRef,`${server.id}:${group}:${window.page}:${query}:${JSON.stringify(window.rows)}`)
  return <section className="players-view"><div className="section-heading"><div><h2>{t("Jugadores")}</h2><p>{t("Conexiones actuales y permisos guardados por Minecraft.")}</p></div><button disabled={loading} onClick={()=>void onRefresh().catch(()=>{})}>{loading?t("Consultando…"):t("Actualizar")}</button></div>
    {(readError||!players)&&<div className={`panel-read-state ${readError?'failed':''}`} role={readError?'alert':'status'}><span className="read-state-mark" aria-hidden="true">{readError?'!':'…'}</span><div><strong>{readError?t("Jugadores sin actualizar"):t("Consultando jugadores")}</strong><p>{readError?(players?t("Se muestran los últimos datos disponibles. "): '')+readError:t("Leyendo conexiones y permisos de Minecraft.")}</p></div></div>}
    <div className="players-overview"><div className="player-summary-connections"><div><strong>{players ? players.online.length : '—'}<span> / {players ? players.maxPlayers : '—'}</span></strong><small><i className={`state-dot ${server.state}`} aria-hidden="true"/>{t("Conectados")}</small></div>{players&&players.online.length>0&&<div className="player-summary-heads" aria-hidden="true">{players.online.slice(0,4).map(player=><PlayerHead key={player.uuid??player.name} name={player.name} uuid={player.uuid}/>)}{players.online.length>4&&<span>+{players.online.length-4}</span>}</div>}</div><div><strong>{players ? players.whitelisted.length : '—'}</strong><small>{t("En lista blanca")}</small></div><div><strong>{players ? players.operators.length : '—'}</strong><small>{t("Operadores")}</small></div></div>
    <div className="players-toolbar"><div><strong>{t("Acceso al servidor")}</strong><p>{!players?t("Acceso pendiente de consultar."):players.whitelistEnabled ? t(players.enforceWhitelist?'Lista blanca activa · acceso estricto':'Lista blanca activa') : t('Sin lista blanca.')}</p></div><button disabled={!canSetWhitelist || busy || loading || !!readError || !players} onClick={toggleWhitelist}>{players?.whitelistEnabled ? t("Desactivar lista blanca") : t("Activar lista blanca")}</button></div>
    {players&&<AuthenticationPanel players={players} state={server.state}/>}
    <nav className="player-tabs" aria-label={t("Categorías de jugadores")}><MovingIndicator active={group} axis="horizontal"/>{groups.map(item => <button key={item.id} className={`player-tab ${group === item.id ? 'active' : ''}`} aria-current={group === item.id ? 'page' : undefined} onClick={() => {setGroup(item.id);setPage(0)}}>{translatedMessage(item.label)}<span>{players ? item.count(players) : '—'}</span></button>)}</nav>
    {group === 'known' && <p className="field-help">{t("Historial de Minecraft. Un mismo nombre puede tener un UUID local y otro de cuenta; sus datos se guardan por separado.")}</p>}
    {group==='known'&&!stopped&&<p className="field-help">{t('Detén el servidor para añadir a la lista blanca el UUID seleccionado.')}</p>}
    <div className="player-list-tools"><label className="form-row" htmlFor="player-search"><span>{t("Buscar en")} {groups.find(item=>item.id===group)?.label.toLowerCase()}</span><input id="player-search" type="search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} placeholder={t("Nombre, UUID o IP")} maxLength={128} disabled={!players}/></label><span>{players?t("{0} resultados",window.total):t("Lista no disponible")}</span></div>
    {players&&query.trim()&&!window.total&&<p className="empty-inline" role="status">{t("No hay coincidencias. Prueba otro nombre, UUID o IP.")}</p>}
    {identityGroup&&!stopped&&<p className="field-help">{t('Detén el servidor para retirar la entrada seleccionada por UUID sin afectar a otra cuenta con el mismo nombre.')}</p>}
    {listNotice&&notice&&<p className="player-notice" role="status">{translatedMessage(notice)}</p>}
    <div className="player-list" ref={listRef}>

      {players && group === 'online' && (items.online.length ? windows.online.rows.map(player => <PlayerRow key={player.uuid??player.name} name={player.name} uuid={player.uuid} detail={player.uuid?t("UUID de esta sesión: {0}",player.uuid):t("UUID de sesión no observado")} action="Expulsar" disabled={!running || busy} onAction={() => act('kick',player.name)}/>) : <p className="empty-inline">{t("No hay jugadores conectados ahora.")}</p>)}
      {players && group === 'known' && (items.known.length ? windows.known.rows.map(player => {
        const allowed=!!player.uuid&&authorized.has(identityKey(player.uuid))
        return <div className="known-player" key={player.uuid || player.name}><PlayerRow name={player.name} uuid={player.uuid} detail={player.uuid ? `${isOfflineUuid(player.uuid) ? t("UUID local · ") : ''}${player.uuid}` : t("UUID no disponible")} action={allowed?'En lista blanca':'Añadir a lista blanca'} disabled={allowed||!stopped||busy||loading||!!readError||!player.uuid} onAction={()=>void authorizeKnown(player)}/><OperatorGrant server={server} player={player} alreadyOperator={!!player.uuid&&operators.has(identityKey(player.uuid))} disabled={busy||loading||!!readError||!players} onRefresh={onRefresh}/></div>
      }) : <p className="empty-inline">{t("El servidor aún no ha guardado jugadores en usercache.json.")}</p>)}
      {players && group === 'whitelist' && (items.whitelisted.length ? windows.whitelist.rows.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={player.uuid || t('UUID no disponible')} action="Quitar" disabled={!stopped || busy || loading || !!readError || !player.uuid} onAction={() => void removeIdentity('whitelist',player.uuid)}/>) : <p className="empty-inline">{t("No hay jugadores en la lista blanca.")}</p>)}
      {players && group === 'operators' && (items.operators.length ? windows.operators.rows.map(player => <div key={player.uuid || player.name}><PlayerRow name={player.name} uuid={player.uuid} detail={[player.uuid||t("UUID no disponible"),`${t("Nivel")} ${player.level}`,player.bypassesPlayerLimit?t("Sin límite de jugadores"):null].filter(Boolean).join(" · ")} action="Retirar OP" disabled={!stopped || busy || loading || !!readError || !player.uuid} onAction={() => void removeIdentity('operators',player.uuid)}/><OperatorSettings server={server} operator={player} disabled={busy||loading||!!readError} onRefresh={onRefresh}/></div>) : <p className="empty-inline">{t("Todavía no hay operadores.")}</p>)}
      {players && group === 'banned' && (items.bannedPlayers.length || items.bannedIps.length ? <>{windows.banned.rows.filter(item=>item.type==='player').map(({entry:player}) => <div className="banned-entry" key={`player:${player.uuid||player.name}`}><PlayerRow name={player.name} uuid={player.uuid} detail={player.uuid||t("UUID no disponible")} action="Perdonar" disabled={!stopped || busy || loading || !!readError || !player.uuid} onAction={() => void removeIdentity('banned_players',player.uuid)}/><BanDetails identity={player.name} ban={player}/></div>)}{windows.banned.rows.filter(item=>item.type==='ip').map(({entry}) => <div className="banned-entry" key={`ip:${entry.ip}`}><PlayerRow name={entry.ip} action="Perdonar IP" disabled={(!running&&!stopped)||busy||loading||!!readError||entry.ip.startsWith('<')} onAction={()=>void pardonIp(entry.ip)}/><BanDetails identity={entry.ip} ban={entry}/></div>)}</> : <p className="empty-inline">{t("No hay jugadores ni direcciones IP bloqueadas.")}</p>)}
    </div>
    {players&&window.pages>1&&<nav className="player-pagination" aria-label={t("Páginas de jugadores")}><span>{t("Página")} {window.page+1}  {t("de")} {window.pages}</span><div><button disabled={window.page===0} onClick={()=>setPage(window.page-1)}>{t("Anterior")}</button><button disabled={window.page+1>=window.pages} onClick={()=>setPage(window.page+1)}>{t("Siguiente")}</button></div></nav>}
    <div hidden={group!=='whitelist'}><WhitelistImportReview whitelisted={items.whitelisted} running={running} disabled={busy||loading||!players||!!readError||!canSetWhitelist} onImport={onImportWhitelist} onRefresh={onRefresh}/></div>
    <div className="player-management"><h3>{t("Gestionar acceso")}</h3><p>{t("Las acciones se envían a Minecraft; sus listas se actualizarán cuando procese el comando.")}</p>
      {!running && <p className="inline-note">{t("Los comandos requieren el servidor iniciado. En Conocidos puedes preparar la lista blanca y los permisos OP por UUID.")}</p>}
      <form noValidate onSubmit={submit}>
        <fieldset className="player-action-fields" disabled={busy} aria-label={t("Acción sobre un jugador")}>
          <label className="form-row" htmlFor="player-action"><span>{t("Acción")}</span><select id="player-action" value={action} onChange={e=>{setAction(e.target.value as Action);setReason('');setAttempted(false)}}>{Object.entries(playerActions).map(([value,item])=><option key={value} value={value}>{translatedMessage(item.label)}</option>)}</select></label>
          <label className="form-row" htmlFor="player-target"><span>{playerActions[action].ip?t("Dirección IP"):t("Jugador")}</span><input ref={targetRef} id="player-target" required value={target} onChange={e=>setTarget(e.target.value)} placeholder={playerActions[action].ip?t("IPv4 o IPv6"):t("Nombre de Minecraft")} maxLength={playerActions[action].ip?45:16} autoComplete="off" spellCheck={false} aria-invalid={attempted&&!!errors.target} aria-describedby={attempted&&errors.target?'player-target-error':undefined}/>{attempted&&errors.target&&<small className="form-error" id="player-target-error">{errors.target}</small>}</label>
          {['ban','ban_ip','kick'].includes(action)&&<label className="form-row" htmlFor="player-reason"><span>{t("Motivo · opcional")}</span><input ref={reasonRef} id="player-reason" value={reason} onChange={e=>setReason(e.target.value)} maxLength={120} aria-invalid={attempted&&!!errors.reason} aria-describedby={attempted&&errors.reason?'player-reason-error':undefined}/>{attempted&&errors.reason&&<small className="form-error" id="player-reason-error">{translatedMessage(errors.reason)}</small>}</label>}
        </fieldset>
        <div className={`player-action-preview ${['op','ban','ban_ip'].includes(action)?'important':''}`}><div><span>{playerActions[action].ip?t("Dirección"):t("Jugador")}</span><strong>{target.trim() || t("Por indicar")}</strong></div><p>{playerActions[action].effect}</p></div>
        <button className={['ban','ban_ip','kick'].includes(action)?'danger-outline-button':'primary-button'} disabled={!running || busy}>{busy?t("Enviando…"):playerActions[action].label}</button>
      </form>{!listNotice&&notice&&<p className="player-notice" role="status">{translatedMessage(notice)}</p>}
    </div>
  </section>
}
