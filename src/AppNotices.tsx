import type {Server} from './api'
import type {AppNotice} from './noticeEvents'
import {t} from './locale'
import './appNotices.css'

export function AppNotices({notices,servers,onOpen,onDismiss}:{
  notices:AppNotice[];servers:Server[];onOpen:(notice:AppNotice)=>void;onDismiss:(id:string)=>void
}) {
  return <section className="app-notices" aria-label={t('Notificaciones')} aria-live="polite" aria-relevant="additions">
    {notices.map(notice=>{
      const server=servers.find(item=>item.id===notice.serverId)
      const failed=notice.type==='crash'||notice.task.stage==='failed'
      return <article className={`app-notice ${failed?'failed':'completed'}`} key={notice.id}>
        <span className="app-notice-mark" aria-hidden="true">{failed?'!':'✓'}</span>
        <div><strong>{server?.name??t('Servidor en preparación')}</strong>
          <p>{notice.type==='crash'?t('Se ha cerrado inesperadamente'):t(notice.task.stage==='failed'?'Tarea fallida: {0}':'Tarea completada: {0}',t(notice.task.label))}</p>
          <button className="app-notice-open" disabled={!server} onClick={()=>onOpen(notice)}>{t(notice.type==='crash'?'Abrir consola':'Ver resultado')}</button>
        </div>
        <button className="app-notice-dismiss" aria-label={t('Cerrar aviso')} onClick={()=>onDismiss(notice.id)}>×</button>
      </article>
    })}
  </section>
}
