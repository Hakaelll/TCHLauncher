import { useEffect, useState } from 'react'
import { api, type NetworkSnapshot, type Server } from './api'
import {RouterPanel} from './RouterPanel'

const stateLabels: Record<NetworkSnapshot['portState'], string> = {
  listening: 'Responde en este equipo',
  starting: 'Esperando al servidor',
  not_responding: 'No responde todavía',
  available: 'Disponible para iniciar',
  occupied: 'Ocupado por otro proceso',
  unknown: 'No se pudo comprobar',
}
const errorText = (error: unknown) => error instanceof Error ? error.message : String(error)

export function NetworkPanel({server}: {server: Server}) {
  const [network,setNetwork] = useState<NetworkSnapshot|null>(null)
  const [statusError,setStatusError] = useState<string|null>(null)
  const [actionError,setActionError] = useState<string|null>(null)
  const [publicIp,setPublicIp] = useState<string|null>(null)
  const [checkingPublic,setCheckingPublic] = useState(false)
  const [copied,setCopied] = useState(false)

  useEffect(() => {
    let active = true
    setNetwork(null); setStatusError(null); setActionError(null); setPublicIp(null); setCopied(false)
    const refresh = () => api.networkStatus(server.id)
      .then(value => { if (active) { setNetwork(value); setStatusError(null) } })
      .catch(cause => { if (active) { setNetwork(null); setStatusError(errorText(cause)) } })
    refresh()
    const timer = window.setInterval(refresh,5000)
    return () => { active = false; window.clearInterval(timer) }
  },[server.id])

  const copy = async () => {
    if (!network?.lanAddress) return
    setCopied(false); setActionError(null)
    try {
      await navigator.clipboard.writeText(network.lanAddress)
      setCopied(true)
      window.setTimeout(() => setCopied(false),2500)
    } catch { setActionError('No se pudo copiar automáticamente. Selecciona la dirección para copiarla.') }
  }
  const checkPublic = async () => {
    setCheckingPublic(true); setActionError(null)
    try { setPublicIp(await api.publicIp()) }
    catch (cause) { setActionError(errorText(cause)) }
    finally { setCheckingPublic(false) }
  }

  return <section className="connection-section" aria-labelledby="network-title">
    <div className="section-heading"><div><h2 id="network-title">Conexión</h2><p>Datos reales de este equipo y del puerto configurado.</p></div></div>
    {statusError && <p className="network-error" role="alert">{statusError}</p>}
    {actionError && <p className="network-error" role="alert">{actionError}</p>}
    <div className="connection-address">
      <span>Dirección para jugadores en tu red local</span>
      {network?.lanAddress ? <div className="network-address-row"><input className="network-address-value" aria-label="Dirección LAN" readOnly value={network.lanAddress} onFocus={event => event.currentTarget.select()}/><button type="button" onClick={copy}>{copied ? 'Copiada' : 'Copiar'}</button></div> : <strong className="network-unavailable">{network ? 'No detectada' : 'Detectando…'}</strong>}
    </div>
    <dl className="data-list network-facts"><div><dt>IP local</dt><dd>{network?.localIp || '—'}</dd></div><div><dt>Puerto TCP</dt><dd>{network?.port ?? '—'}</dd></div><div><dt>Estado local</dt><dd>{network ? stateLabels[network.portState] : 'Comprobando…'}</dd></div><div><dt>En este equipo</dt><dd>{network?.localAddress || '—'}</dd></div></dl>
    {network?.configuredIp && <p className="muted-note">El servidor está configurado para escuchar en {network.configuredIp}. Si esta dirección es de loopback, otros equipos no podrán entrar.</p>}
    {!network?.lanAddress && network && <p className="muted-note">Comprueba que el ordenador esté conectado a la red y que el servidor no esté limitado a una dirección local.</p>}
    <details className="network-guide"><summary>Jugar desde fuera de esta red</summary><div className="network-guide-content"><RouterPanel key={server.id} server={server} network={network}/><p>Si la apertura automática no está disponible, redirige manualmente el puerto <strong>{network?.port ?? 'del servidor'} TCP</strong> en tu router hacia <strong>{network?.localIp ?? 'la IP local de este equipo'}</strong> y permite Java en el firewall.</p><div className="network-public"><div><span>IP pública de salida</span><strong>{publicIp || 'Sin consultar'}</strong></div><button type="button" disabled={checkingPublic} onClick={checkPublic}>{checkingPublic ? 'Consultando…' : 'Consultar IP pública'}</button></div><p className="network-disclosure">Esta consulta opcional contacta con ipify. No comprueba el router ni crea una apertura de puerto; el servidor local funciona sin usar ese servicio.</p></div></details>
  </section>
}
