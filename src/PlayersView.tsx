import { MovingIndicator, useListMotion } from './motion'
import {PlayerHead} from './PlayerHead'
import { useRef, useState, type FormEvent } from 'react'
import { api, type PlayersSnapshot, type Server, type WhitelistImport } from './api'

type Group = 'online' | 'known' | 'whitelist' | 'operators' | 'banned'
type Action = 'whitelist_add' | 'op' | 'ban' | 'kick' | 'ban_ip'

const groups: { id: Group; label: string; count: (players: PlayersSnapshot) => number }[] = [
  { id: 'online', label: 'Conectados', count: players => players.online.length },
  { id: 'known', label: 'Conocidos', count: players => players.known.length },
  { id: 'whitelist', label: 'Lista blanca', count: players => players.whitelisted.length },
  { id: 'operators', label: 'Operadores', count: players => players.operators.length },
  { id: 'banned', label: 'Bloqueados', count: players => players.bannedPlayers.length + players.bannedIps.length },
]

const actionLabel: Record<Action, string> = {
  whitelist_add: 'Añadir a lista blanca', op: 'Dar permisos OP', ban: 'Bloquear jugador',
  kick: 'Expulsar jugador', ban_ip: 'Bloquear IP',
}

function PlayerRow({name,uuid,detail,action,disabled,onAction}: {name:string;uuid?:string|null;detail?:string;action?:string;disabled:boolean;onAction?:()=>void}) {
  return <div className="player-row" data-layout-id={uuid || name}><PlayerHead name={name} uuid={uuid}/><div className="player-row-info"><strong>{name}</strong>{detail && <small title={detail}>{detail}</small>}</div>{action && <button className="text-button" disabled={disabled} onClick={onAction}>{action}</button>}</div>
}

export function PlayersView({server,players,busy,onAction,onToggleWhitelist,onImportWhitelist,onRefresh}: {
  server: Server;
  players: PlayersSnapshot | null;
  busy: boolean;
  onAction: (action:string,target?:string,reason?:string) => Promise<boolean>;
  onToggleWhitelist: () => Promise<boolean>;
  onImportWhitelist: (names:string[]) => Promise<boolean>;
  onRefresh: () => Promise<void>;
}) {
  const [group,setGroup] = useState<Group>('online')
  const listRef=useRef<HTMLDivElement>(null)
  useListMotion(listRef,`${server.id}:${group}:${JSON.stringify(players)}`)
  const [action,setAction] = useState<Action>('whitelist_add')
  const [target,setTarget] = useState('')
  const [reason,setReason] = useState('')
  const [notice,setNotice] = useState<string|null>(null)
  const [importFile,setImportFile] = useState<WhitelistImport|null>(null)
  const [importError,setImportError] = useState<string|null>(null)
  const running = server.state === 'running'
  const canSetWhitelist = running || ['stopped','crashed','error'].includes(server.state)
  const act = async (kind: string, name?: string, motive?: string) => {
    setNotice(null)
    const success = await onAction(kind,name,motive)
    if (success) {
      setNotice('Comando enviado. La lista se actualizará cuando el servidor lo procese.')
      await onRefresh()
    }
    return success
  }
  const submit = async (event: FormEvent) => {
    event.preventDefault()
    if (await act(action,target.trim(),reason.trim())) { setTarget(''); setReason('') }
  }
  const toggleWhitelist = async () => {
    setNotice(null)
    if (await onToggleWhitelist()) {
      setNotice(running ? 'Comando enviado. La lista se actualizará cuando el servidor lo procese.' : 'Acceso actualizado para el próximo inicio.')
      await onRefresh()
    }
  }
  const items = players ?? { online:[],known:[],whitelisted:[],operators:[],bannedPlayers:[],bannedIps:[],maxPlayers:0,whitelistEnabled:false,enforceWhitelist:false }
  const existing = new Set(items.whitelisted.map(player => player.name.toLowerCase()))
  const pendingNames = importFile?.names.filter(name => !existing.has(name.toLowerCase())) ?? []
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
  return <section className="players-view"><div className="section-heading"><div><h2>Jugadores</h2><p>Conexiones actuales y permisos guardados por Minecraft.</p></div><button onClick={onRefresh}>Actualizar</button></div>
    <div className="players-overview"><div><strong>{players ? players.online.length : '—'}<span> / {players ? players.maxPlayers : '—'}</span></strong><small>Conectados</small></div><div><strong>{players ? players.whitelisted.length : '—'}</strong><small>En lista blanca</small></div><div><strong>{players ? players.operators.length : '—'}</strong><small>Operadores</small></div></div>
    <div className="players-toolbar"><div><strong>Acceso al servidor</strong><p>{players?.whitelistEnabled ? `Lista blanca activa${players.enforceWhitelist ? ' · acceso estricto' : ''}` : 'Cualquier cuenta válida puede entrar si conoce la dirección.'}</p></div><button disabled={!canSetWhitelist || busy || !players} onClick={toggleWhitelist}>{players?.whitelistEnabled ? 'Desactivar lista blanca' : 'Activar lista blanca'}</button></div>
    <nav className="player-tabs" aria-label="Categorías de jugadores"><MovingIndicator active={group} axis="horizontal"/>{groups.map(item => <button key={item.id} className={`player-tab ${group === item.id ? 'active' : ''}`} aria-current={group === item.id ? 'page' : undefined} onClick={() => setGroup(item.id)}>{item.label}<span>{players ? item.count(players) : '—'}</span></button>)}</nav>
    <div className="player-list" ref={listRef}>
      {group === 'online' && (items.online.length ? items.online.map(name => <PlayerRow key={name} name={name} uuid={[...items.known,...items.whitelisted,...items.operators].find(player=>player.name.toLowerCase()===name.toLowerCase())?.uuid} detail="En el servidor" action="Expulsar" disabled={!running || busy} onAction={() => act('kick',name)}/>) : <p className="empty-inline">No hay jugadores conectados ahora.</p>)}
      {group === 'known' && (items.known.length ? items.known.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={player.uuid || 'UUID no disponible'} disabled={false}/>) : <p className="empty-inline">El servidor aún no ha guardado jugadores en usercache.json.</p>)}
      {group === 'whitelist' && (items.whitelisted.length ? items.whitelisted.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={player.uuid || undefined} action="Quitar" disabled={!running || busy} onAction={() => act('whitelist_remove',player.name)}/>) : <p className="empty-inline">No hay jugadores en la lista blanca.</p>)}
      {group === 'operators' && (items.operators.length ? items.operators.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={`Nivel ${player.level}${player.bypassesPlayerLimit ? ' · sin límite de jugadores' : ''}`} action="Retirar OP" disabled={!running || busy} onAction={() => act('deop',player.name)}/>) : <p className="empty-inline">Todavía no hay operadores.</p>)}
      {group === 'banned' && (items.bannedPlayers.length || items.bannedIps.length ? <>{items.bannedPlayers.map(player => <PlayerRow key={player.uuid || player.name} name={player.name} uuid={player.uuid} detail={[player.reason,player.source,player.created,player.expires].filter(Boolean).join(' · ')} action="Perdonar" disabled={!running || busy} onAction={() => act('pardon',player.name)}/>)}{items.bannedIps.map(entry => <PlayerRow key={entry.ip} name={entry.ip} detail={[entry.reason,entry.source,entry.created,entry.expires].filter(Boolean).join(' · ')} action="Perdonar IP" disabled={!running || busy || entry.ip.startsWith('<')} onAction={() => act('pardon_ip',entry.ip)}/>)}</> : <p className="empty-inline">No hay jugadores ni direcciones IP bloqueadas.</p>)}
    </div>
    {group === 'whitelist' && <div className="whitelist-import"><div><strong>Importar jugadores</strong><p>Elige un archivo .txt con un nombre por línea o un whitelist.json de Minecraft.</p></div><button onClick={chooseImport} disabled={busy}>Elegir archivo</button>{importError && <p className="form-error" role="alert">{importError}</p>}{importFile && <div className="whitelist-import-preview"><div><strong>{importFile.fileName}</strong><span>{pendingNames.length} nuevos · {importFile.names.length} en el archivo</span></div><p>{importFile.names.slice(0,8).join(', ')}{importFile.names.length > 8 ? ` y ${importFile.names.length - 8} más` : ''}</p><div><button className="text-button" onClick={() => setImportFile(null)}>Cancelar</button><button className="primary-button" disabled={!running || busy || !pendingNames.length} onClick={applyImport}>Añadir {pendingNames.length} jugadores</button></div>{!running && <p className="field-help">Inicia el servidor para aplicar la importación.</p>}</div>}</div>}
    <div className="player-management"><h3>Gestionar acceso</h3><p>Las acciones se ejecutan como comandos del servidor y Minecraft actualiza sus propios archivos.</p>{!running && <p className="inline-note">Inicia el servidor para modificar jugadores y permisos. Puedes consultar las listas mientras está detenido.</p>}<form onSubmit={submit}><label className="form-row" htmlFor="player-action"><span>Acción</span><select id="player-action" value={action} onChange={e => { setAction(e.target.value as Action); setReason('') }}>{Object.entries(actionLabel).map(([value,label]) => <option key={value} value={value}>{label}</option>)}</select></label><label className="form-row" htmlFor="player-target"><span>{action === 'ban_ip' ? 'Dirección IP' : 'Jugador'}</span><input id="player-target" required value={target} onChange={e => setTarget(e.target.value)} placeholder={action === 'ban_ip' ? '192.168.1.25' : 'Nombre de Minecraft'} maxLength={45}/></label>{['ban','ban_ip','kick'].includes(action) && <label className="form-row" htmlFor="player-reason"><span>Motivo · opcional</span><input id="player-reason" value={reason} onChange={e => setReason(e.target.value)} maxLength={120}/></label>}<button className="primary-button" disabled={!running || busy || !target.trim()}>Aplicar</button></form>{notice && <p className="player-notice" role="status">{notice}</p>}</div>
  </section>
}
