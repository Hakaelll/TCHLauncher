import { MovingIndicator, useListMotion } from './motion'
import {OperatorSettings} from './OperatorSettings'
import {PlayerHead} from './PlayerHead'
import {connectedIdentity, isOfflineUuid} from './playerIdentity'
import { useRef, useState, type FormEvent } from 'react'
import { api, type PlayersSnapshot, type Server, type WhitelistImport } from './api'
import {playerActions,playerActionErrors,type PlayerAction as Action} from './playerActions'
import {playerWindow} from './playerWindow'

type Group = 'online' | 'known' | 'whitelist' | 'operators' | 'banned'

const groups: { id: Group; label: string; count: (players: PlayersSnapshot) => number }[] = [
  { id: 'online', label: 'Conectados', count: players => players.online.length },
  { id: 'known', label: 'Conocidos', count: players => players.known.length },
  { id: 'whitelist', label: 'Lista blanca', count: players => players.whitelisted.length },
  { id: 'operators', label: 'Operadores', count: players => players.operators.length },
  { id: 'banned', label: 'Bloqueados', count: players => players.bannedPlayers.length + players.bannedIps.length },
]

function PlayerRow({name,uuid,detail,action,disabled,onAction}: {name:string;uuid?:string|null;detail?:string;action?:string;disabled:boolean;onAction?:()=>void}) {
  return <div className="player-row" data-layout-id={uuid || name}><PlayerHead name={name} uuid={uuid}/><div className="player-row-info"><strong>{name}</strong>{detail && <small title={detail}>{detail}</small>}</div>{action && <button className="text-button" disabled={disabled} onClick={onAction}>{action}</button>}</div>
}

export function PlayersView({server,players,loading,readError,busy,onAction,onToggleWhitelist,onImportWhitelist,onRefresh}: {
  server: Server;
  players: PlayersSnapshot | null;
  loading:boolean;
  readError:string|null;
  busy: boolean;
  onAction: (action:string,target?:string,reason?:string) => Promise<boolean>;
  onToggleWhitelist: () => Promise<boolean>;
  onImportWhitelist: (names:string[]) => Promise<boolean>;
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
  const [importFile,setImportFile] = useState<WhitelistImport|null>(null)
  const [importError,setImportError] = useState<string|null>(null)
  const running = server.state === 'running'
  const canSetWhitelist = running || ['stopped','crashed','error'].includes(server.state)
  const act = async (kind: string, name?: string, motive?: string) => {
    setNotice(null)
    const success = await onAction(kind,name,motive)
    if (success) {
      setNotice(`${name ? `${name}: ` : ''}comando enviado. La lista se actualizará cuando el servidor lo procese.`)
      await onRefresh()
    }
    return success
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
    setNotice(null)
    if (await onToggleWhitelist()) {
      setNotice(running ? 'Comando enviado. La lista se actualizará cuando el servidor lo procese.' : 'Acceso actualizado para el próximo inicio.')
      await onRefresh()
    }
  }
  const items = players ?? { online:[],known:[],whitelisted:[],operators:[],bannedPlayers:[],bannedIps:[],maxPlayers:0,whitelistEnabled:false,enforceWhitelist:false,onlineMode:true }
  const existing = new Set(items.whitelisted.map(player => player.name.toLowerCase()))
  const pendingNames = importFile?.names.filter(name => !existing.has(name.toLowerCase())) ?? []
  const errors=playerActionErrors(action,target,reason)
  const windows={
    online:playerWindow(items.online,query,name=>name,page),
    known:playerWindow(items.known,query,player=>`${player.name} ${player.uuid??''}`,page),
    whitelist:playerWindow(items.whitelisted,query,player=>`${player.name} ${player.uuid??''}`,page),
    operators:playerWindow(items.operators,query,player=>`${player.name} ${player.uuid??''}`,page),
    banned:playerWindow([...items.bannedPlayers.map(entry=>({type:'player' as const,entry})),...items.bannedIps.map(entry=>({type:'ip' as const,entry}))],query,item=>item.type==='player'?`${item.entry.name} ${item.entry.uuid??''}`:item.entry.ip,page),
  }
  const window=windows[group]
  useListMotion(listRef,`${server.id}:${group}:${window.page}:${query}:${JSON.stringify(window.rows)}`)
  const chooseImport = async () => {
    setImportError(null)
    try { const selection = await api.pickWhitelistFile(); if (selection) setImportFile(selection) }
    catch(e) { setImportError(e instanceof Error ? e.message : String(e)) }
  }
  const applyImport = async () => {
    setImportError(null)
    if (await onImportWhitelist(pendingNames)) {
      setNotice(`Se enviaron ${pendingNames.length} solicitudes de lista blanca. Minecraft actualizará la lista al procesarlas.`)
      setImportFile(null)
      await onRefresh()
    }
  }
  return <section className="players-view"><div className="section-heading"><div><h2>Jugadores</h2><p>Conexiones actuales y permisos guardados por Minecraft.</p></div><button disabled={loading} onClick={onRefresh}>{loading?'Consultando…':'Actualizar'}</button></div>
    {(readError||!players)&&<div className={`panel-read-state ${readError?'failed':''}`} role={readError?'alert':'status'}><span className="read-state-mark" aria-hidden="true">{readError?'!':'…'}</span><div><strong>{readError?'Jugadores sin actualizar':'Consultando jugadores'}</strong><p>{readError?(players?'Se muestran los últimos datos disponibles. ': '')+readError:'Leyendo conexiones y permisos de Minecraft.'}</p></div></div>}
    <div className="players-overview"><div><strong>{players ? players.online.length : '—'}<span> / {players ? players.maxPlayers : '—'}</span></strong><small>Conectados</small></div><div><strong>{players ? players.whitelisted.length : '—'}</strong><small>En lista blanca</small></div><div><strong>{players ? players.operators.length : '—'}</strong><small>Operadores</small></div></div>
    <div className="players-toolbar"><div><strong>Acceso al servidor</strong><p>{!players?'Acceso pendiente de consultar.':players.whitelistEnabled ? `Lista blanca activa${players.enforceWhitelist ? ' · acceso estricto' : ''}` : players?.onlineMode === false ? 'Sin lista blanca. Este servidor no autentica directamente las cuentas de Minecraft.' : 'Sin lista blanca. El servidor autentica las cuentas de Minecraft.'}</p></div><button disabled={!canSetWhitelist || busy || !players} onClick={toggleWhitelist}>{players?.whitelistEnabled ? 'Desactivar lista blanca' : 'Activar lista blanca'}</button></div>
    <nav className="player-tabs" aria-label="Categorías de jugadores"><MovingIndicator active={group} axis="horizontal"/>{groups.map(item => <button key={item.id} className={`player-tab ${group === item.id ? 'active' : ''}`} aria-current={group === item.id ? 'page' : undefined} onClick={() => {setGroup(item.id);setPage(0)}}>{item.label}<span>{players ? item.count(players) : '—'}</span></button>)}</nav>
    {group === 'known' && <p className="field-help">Historial de Minecraft. Un mismo nombre puede tener un UUID local y otro de cuenta; sus datos se guardan por separado.</p>}
    <div className="player-list-tools"><label className="form-row" htmlFor="player-search"><span>Buscar en {groups.find(item=>item.id===group)?.label.toLowerCase()}</span><input id="player-search" value={query} onChange={event=>{setQuery(event.target.value);setPage(0)}} placeholder="Nombre, UUID o IP" maxLength={128} disabled={!players}/></label><span>{players?`${window.total} resultados`:'Lista no disponible'}</span></div>
    {players&&query.trim()&&!window.total&&<p className="empty-inline" role="status">No hay coincidencias. Prueba otro nombre, UUID o IP.</p>}
    <div className="player-list" ref={listRef}>

      {players && group === 'online' && (items.online.length ? windows.online.rows.map(name => <PlayerRow key={name} name={name} uuid={connectedIdentity(name,[...items.known,...items.whitelisted,...items.operators],items.onlineMode)} detail="En el servidor" action="Expulsar" disabled={!running || busy} onAction={() => act('kick',name)}/>) : <p className="empty-inline">No hay jugadores conectados ahora.</p>)}
      {players && group === 'known' && (items.known.length ? windows.known.rows.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={player.uuid ? `${isOfflineUuid(player.uuid) ? 'UUID local · ' : ''}${player.uuid}` : 'UUID no disponible'} disabled={false}/>) : <p className="empty-inline">El servidor aún no ha guardado jugadores en usercache.json.</p>)}
      {players && group === 'whitelist' && (items.whitelisted.length ? windows.whitelist.rows.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={player.uuid || undefined} action="Quitar" disabled={!running || busy} onAction={() => act('whitelist_remove',player.name)}/>) : <p className="empty-inline">No hay jugadores en la lista blanca.</p>)}
      {players && group === 'operators' && (items.operators.length ? windows.operators.rows.map(player => <div key={player.uuid || player.name}><PlayerRow name={player.name} uuid={player.uuid} detail={`Nivel ${player.level}${player.bypassesPlayerLimit ? ' · sin límite de jugadores' : ''}`} action="Retirar OP" disabled={!running || busy} onAction={() => act('deop',player.name)}/><OperatorSettings server={server} operator={player} disabled={busy} onRefresh={onRefresh}/></div>) : <p className="empty-inline">Todavía no hay operadores.</p>)}
      {players && group === 'banned' && (items.bannedPlayers.length || items.bannedIps.length ? <>{windows.banned.rows.filter(item=>item.type==='player').map(({entry:player}) => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={[player.reason,player.source,player.created,player.expires].filter(Boolean).join(' · ')} action="Perdonar" disabled={!running || busy} onAction={() => act('pardon',player.name)}/>)}{windows.banned.rows.filter(item=>item.type==='ip').map(({entry}) => <PlayerRow key={entry.ip} name={entry.ip} detail={[entry.reason,entry.source,entry.created,entry.expires].filter(Boolean).join(' · ')} action="Perdonar IP" disabled={!running || busy || entry.ip.startsWith('<')} onAction={() => act('pardon_ip',entry.ip)}/>)}</> : <p className="empty-inline">No hay jugadores ni direcciones IP bloqueadas.</p>)}
    </div>
    {players&&window.pages>1&&<nav className="player-pagination" aria-label="Páginas de jugadores"><span>Página {window.page+1} de {window.pages}</span><div><button disabled={window.page===0} onClick={()=>setPage(window.page-1)}>Anterior</button><button disabled={window.page+1>=window.pages} onClick={()=>setPage(window.page+1)}>Siguiente</button></div></nav>}
    {group === 'whitelist' && <div className="whitelist-import"><div><strong>Importar jugadores</strong><p>Elige un archivo .txt con un nombre por línea o un whitelist.json de Minecraft.</p></div><button onClick={chooseImport} disabled={busy}>Elegir archivo</button>{importError && <p className="form-error" role="alert">{importError}</p>}{importFile && <div className="whitelist-import-preview"><div><strong>{importFile.fileName}</strong><span>{pendingNames.length} nuevos · {importFile.names.length} en el archivo</span></div><p>{importFile.names.slice(0,8).join(', ')}{importFile.names.length > 8 ? ` y ${importFile.names.length - 8} más` : ''}</p><div><button className="text-button" onClick={() => setImportFile(null)}>Cancelar</button><button className="primary-button" disabled={!running || busy || !pendingNames.length} onClick={applyImport}>Añadir {pendingNames.length} jugadores</button></div>{!running && <p className="field-help">Inicia el servidor para aplicar la importación.</p>}</div>}</div>}
    <div className="player-management"><h3>Gestionar acceso</h3><p>Las acciones se envían a Minecraft; sus listas se actualizarán cuando procese el comando.</p>
      {!running && <p className="inline-note">Inicia el servidor para modificar jugadores y permisos. Puedes preparar una acción mientras está detenido.</p>}
      <form noValidate onSubmit={submit}>
        <fieldset className="player-action-fields" disabled={busy} aria-label="Acción sobre un jugador">
          <label className="form-row" htmlFor="player-action"><span>Acción</span><select id="player-action" value={action} onChange={e=>{setAction(e.target.value as Action);setReason('');setAttempted(false)}}>{Object.entries(playerActions).map(([value,item])=><option key={value} value={value}>{item.label}</option>)}</select></label>
          <label className="form-row" htmlFor="player-target"><span>{playerActions[action].ip?'Dirección IP':'Jugador'}</span><input ref={targetRef} id="player-target" required value={target} onChange={e=>setTarget(e.target.value)} placeholder={playerActions[action].ip?'IPv4 o IPv6':'Nombre de Minecraft'} maxLength={playerActions[action].ip?45:16} autoComplete="off" spellCheck={false} aria-invalid={attempted&&!!errors.target} aria-describedby={attempted&&errors.target?'player-target-error':undefined}/>{attempted&&errors.target&&<small className="form-error" id="player-target-error">{errors.target}</small>}</label>
          {['ban','ban_ip','kick'].includes(action)&&<label className="form-row" htmlFor="player-reason"><span>Motivo · opcional</span><input ref={reasonRef} id="player-reason" value={reason} onChange={e=>setReason(e.target.value)} maxLength={120} aria-invalid={attempted&&!!errors.reason} aria-describedby={attempted&&errors.reason?'player-reason-error':undefined}/>{attempted&&errors.reason&&<small className="form-error" id="player-reason-error">{errors.reason}</small>}</label>}
        </fieldset>
        <div className={`player-action-preview ${['op','ban','ban_ip'].includes(action)?'important':''}`}><div><span>{playerActions[action].ip?'Dirección':'Jugador'}</span><strong>{target.trim() || 'Por indicar'}</strong></div><p>{playerActions[action].effect}</p></div>
        <button className={['ban','ban_ip','kick'].includes(action)?'danger-outline-button':'primary-button'} disabled={!running || busy}>{busy?'Enviando…':playerActions[action].label}</button>
      </form>{notice&&<p className="player-notice" role="status">{notice}</p>}
    </div>
  </section>
}
