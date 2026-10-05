import { useEffect, useState } from 'react'
import { api, type NetworkSnapshot, type Server } from './api'
import {RouterPanel} from './RouterPanel'
import {NatPmpPanel} from './NatPmpPanel'

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
    <div className="section-heading"><div><h2 id="network-title" tabIndex={-1}>Conexión</h2><p>Comparte la dirección con quien juegue en tu misma red.</p></div></div>
    {statusError && <p className="network-error" role="alert">{statusError}</p>}
    {actionError && <p className="network-error" role="alert">{actionError}</p>}
    <div className="connection-address">
      <span>En tu red local</span>
      {network?.lanAddress ? <div className="network-address-row"><input className="network-address-value" aria-label="Dirección LAN" readOnly value={network.lanAddress} onFocus={event => event.currentTarget.select()}/><button type="button" onClick={copy}>{copied ? 'Copiada' : 'Copiar'}</button></div> : <strong className="network-unavailable">{network ? 'No detectada' : 'Detectando…'}</strong>}
    </div>
    <dl className="data-list network-facts"><div><dt>IP local</dt><dd>{network?.localIp || '—'}</dd></div><div><dt>Puerto TCP</dt><dd>{network?.port ?? '—'}</dd></div><div><dt>Estado local</dt><dd>{network ? stateLabels[network.portState] : 'Comprobando…'}</dd></div><div><dt>En este equipo</dt><dd>{network?.localAddress || '—'}</dd></div></dl>
    {network?.configuredIp && <p className="muted-note">El servidor está configurado para escuchar en {network.configuredIp}. Si esta dirección es de loopback, otros equipos no podrán entrar.</p>}
    {!network?.lanAddress && network && <p className="muted-note">Comprueba que el ordenador esté conectado a la red y que el servidor no esté limitado a una dirección local.</p>}
    <section className="internet-access" aria-labelledby="internet-title"><div className="section-heading"><div><h3 id="internet-title">Jugar desde internet</h3><p>Detección automática · apertura temporal</p></div></div><div className="router-methods"><RouterPanel key={server.id} server={server} network={network}/><NatPmpPanel key={`natpmp:${server.id}`} server={server} network={network}/></div><div className="network-manual"><div className="network-public"><div><span>IP pública de salida</span><strong>{publicIp || 'Sin consultar'}</strong></div><button type="button" title="Consulta opcional a ipify; no abre puertos" disabled={checkingPublic} onClick={checkPublic}>{checkingPublic ? 'Consultando…' : 'Consultar IP pública'}</button></div><details className="connection-help"><summary>Apertura manual y ayuda de conexión</summary><p>En tu router, redirige <strong>{network?.port ?? 'el puerto del servidor'} TCP</strong> hacia <strong>{network?.localIp ?? 'la IP local de este equipo'}</strong> y permite Java en el firewall. La apertura automática se renueva mientras el servidor está activo y se intenta retirar al detenerlo. No comprueba el acceso desde internet. La consulta de IP usa ipify; es opcional y no abre puertos.</p></details></div></section>
  </section>
}
