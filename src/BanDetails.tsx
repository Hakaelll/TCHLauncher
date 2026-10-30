import type {BannedIp} from './api'
import {t} from './locale'

export function BanDetails({identity,ban}:{identity:string;ban:Pick<BannedIp,'reason'|'source'|'created'|'expires'>}) {
  const expiry=ban.expires==='forever'?t('Sin caducidad'):ban.expires?.trim()?ban.expires:t('No registrado')
  return <details className="ban-details"><summary>{t('Caducidad')}: {expiry}<span className="sr-only"> · {identity}</span></summary>
    <dl className="data-list"><div><dt>{t('Motivo')}</dt><dd>{ban.reason||t('Motivo no registrado')}</dd></div>
      <div><dt>{t('Origen')}</dt><dd>{ban.source||t('No registrado')}</dd></div>
      <div><dt>{t('Fecha del bloqueo')}</dt><dd>{ban.created||t('No registrado')}</dd></div>
      <div><dt>{t('Caducidad')}</dt><dd>{expiry}</dd></div></dl>
  </details>
}
