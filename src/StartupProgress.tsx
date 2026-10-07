import {t} from './locale.ts'
import type {Metrics,ServerState,StartupPhase} from './api'

export function StartupProgress({state,metrics,observedPhase}:{state:ServerState;metrics:Metrics|null;observedPhase?:StartupPhase}){
  if(state!=='starting'&&state!=='running')return null
  const phase=state==='running'?'ready':observedPhase??metrics?.startupPhase
  const step=phase==='ready'?2:phase==='loading_world'||phase==='preparing_spawn'?1:0
  const text=phase==='ready'?t("Servidor listo para recibir jugadores."):phase==='preparing_spawn'?t("Preparando la zona inicial del mundo."):phase==='loading_world'?t("Cargando el mundo."):phase==='launching'?t("Java está iniciando el servidor."):t("Esperando información del proceso Java.")
  return <div className="startup-progress" data-ready={phase==='ready'}>
    <ol aria-label={t("Fases del arranque")}>{['Java',t("Mundo"),t("Listo")].map((label,index)=>{
      const complete=index<step||phase==='ready'
      return <li key={index} data-state={complete?'complete':index===step?'current':'pending'} aria-current={!complete&&index===step?'step':undefined}><span className="startup-step-number" aria-hidden="true">{complete?<svg viewBox="0 0 16 16" fill="currentColor" shapeRendering="crispEdges"><path d="M2 7h3v3h3V7h3V4h3v4h-3v3H8v3H5v-3H2z"/></svg>:index+1}</span><span>{label}</span><span className="sr-only">{complete?t(" · Completado"):index===step?t(" · En curso"):t(" · Pendiente")}</span></li>
    })}</ol>
    <p role="status">{text}</p>
  </div>
}
