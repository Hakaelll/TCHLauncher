import type {Server} from './api'

export {fmtBytes} from './format'
export const platformName = (platform:Server['platform']) => ({vanilla:'Vanilla',paper:'Paper',fabric:'Fabric',neoforge:'NeoForge'})[platform]

export function Icon({name, size = 18}: {name: 'plus' | 'folder' | 'arrow' | 'back' | 'server' | 'check'; size?: number}) {
  const paths = {
    plus: <path d="M12 5v14M5 12h14"/>,
    folder: <path d="M3.5 7.5a2 2 0 0 1 2-2h4l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>,
    arrow: <><path d="M4 12h15"/><path d="m13 6 6 6-6 6"/></>,
    back: <><path d="M20 12H5"/><path d="m11 6-6 6 6 6"/></>,
    server: <><rect x="4" y="4" width="16" height="7" rx="2"/><rect x="4" y="13" width="16" height="7" rx="2"/><path d="M8 7.5h.01M8 16.5h.01"/></>,
    check: <path d="m5 12 4.5 4.5L19 7"/>,
  }
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{paths[name]}</svg>
}

export function ServerArtwork({server}: {server: Server}) {
  return server.imageData
    ? <img className="server-artwork" src={server.imageData} alt=""/>
    : <span className="server-artwork default" aria-hidden="true">{server.name.trim().slice(0,2).toLocaleUpperCase()}</span>
}
