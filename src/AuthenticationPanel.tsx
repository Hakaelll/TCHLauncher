import {t,translatedMessage} from './locale'
import type {PlayersSnapshot,ServerState} from './api'
import {authenticationState} from './authenticationState'
import {isOfflineUuid} from './playerIdentity'

export function AuthenticationPanel({players,state}:{players:PlayersSnapshot;state:ServerState}) {
  const evidence=authenticationState(players.onlineMode,players.authenticationChecks)
  const localSession=evidence.kind==='direct'&&state==='running'&&players.online.some(player=>player.uuid!=null&&isOfflineUuid(player.uuid))
  const needsReview=evidence.kind!=='direct'||localSession
  return <section className={`authentication-panel ${needsReview?'needs-review':''}`} aria-label={t('Identidad de Minecraft')}>
    <div className="authentication-heading"><strong>{t('Identidad de Minecraft')}</strong><span>{localSession?t('UUID de tipo local observado'):evidence.kind==='proxy'?t('Conexión mediante proxy'):evidence.kind==='unknown'?t('Configuración sin comprobar'):evidence.kind==='offline'?t('Autenticación desactivada'):t('Autenticación directa configurada')}</span></div>
    {needsReview&&<p>{localSession?t('Se muestran jugadores con UUID de tipo local, aunque los archivos indican autenticación directa. Si cambiaste estos ajustes, reinicia; si persiste, revisa la conexión y el proxy.'):evidence.kind==='proxy'?t('Hay ajustes de proxy activos. Si te conectas directamente a este ordenador, revisa estos valores; cambiar solo online-mode puede no ser suficiente.'):evidence.kind==='unknown'?t('No se pudieron comprobar todos los ajustes de proxy. Consulta los archivos indicados antes de cambiar la autenticación.'):t('online-mode=false no valida directamente tu cuenta de Minecraft. Para una conexión directa con tu cuenta, cambia online-mode a true en Configuración y reinicia.')}</p>}
    {!!evidence.enabled.length&&<ul>{evidence.enabled.map(check=><li key={`${check.file}:${check.key}`}><code>{check.file}</code><code>{check.key}=true</code></li>)}</ul>}
    {!!evidence.unknown.length&&<ul>{evidence.unknown.map(check=><li key={`${check.file}:${check.key}`}><code>{check.file}</code><span>{translatedMessage(check.problem??t('No comprobado'))}</span></li>)}</ul>}
    {needsReview&&<p className="field-help">{t('Los UUID locales y los de cuenta conservan inventarios y permisos separados. La aplicación no los fusiona ni borra.')}</p>}
    {['running','starting','stopping'].includes(state)&&<small>{t('Se muestran los archivos guardados. Los cambios de autenticación requieren reiniciar el servidor.')}</small>}
  </section>
}
