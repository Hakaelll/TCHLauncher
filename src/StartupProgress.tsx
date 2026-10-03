import type {Metrics,ServerState,StartupPhase} from './api'

export function StartupProgress({state,metrics,observedPhase}:{state:ServerState;metrics:Metrics|null;observedPhase?:StartupPhase}){
  if(state!=='starting'&&state!=='running')return null
  const phase=state==='running'?'ready':observedPhase??metrics?.startupPhase
  const step=phase==='ready'?2:phase==='loading_world'||phase==='preparing_spawn'?1:0
  const text=phase==='ready'?'Servidor listo para recibir jugadores.':phase==='preparing_spawn'?'Preparando la zona inicial del mundo.':phase==='loading_world'?'Cargando el mundo.':phase==='launching'?'Java está iniciando el servidor.':'Esperando información del proceso Java.'
  return <div className="startup-progress">
    <ol aria-label="Fases del arranque">{['Java','Mundo','Listo'].map((label,index)=><li key={label} data-state={index<step?'complete':index===step?'current':'pending'} aria-current={index===step?'step':undefined}><span className="startup-step-number" aria-hidden="true">{index<step?'✓':index+1}</span><span>{label}</span><span className="sr-only">{index<step?' completado':index===step?' actual':' pendiente'}</span></li>)}</ol>
    <p role="status">{text}</p>
  </div>
}
