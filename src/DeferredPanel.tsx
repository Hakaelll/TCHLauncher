import {translatedMessage} from './locale.ts'
import {t,getLanguage} from './locale.ts'
import {Component,Suspense,lazy,useState,type ComponentType,type ReactNode} from 'react'

type BoundaryProps={label:string;onRetry:()=>void;children:ReactNode}
class PanelBoundary extends Component<BoundaryProps,{error:Error|null}>{
  state:{error:Error|null}={error:null}
  static getDerivedStateFromError(error:unknown){return {error:error instanceof Error?error:new Error(String(error))}}
  render(){
    if(!this.state.error)return this.props.children
    return <section className="deferred-panel failed" role="alert"><div><h3>{t("No se pudo abrir")} {this.props.label.toLocaleLowerCase(getLanguage()==='en'?'en-US':'es-ES')}</h3><p>{t("Vuelve a intentarlo. Las demás secciones siguen disponibles.")}</p><details><summary>{t("Detalle técnico")}</summary><p>{this.state.error.message}</p></details></div><button onClick={this.props.onRetry}>{t("Reintentar")}</button></section>
  }
}

// Each panel owns its boundary, so a failed module cannot replace the app shell.
export function deferredPanel<P extends object>(label:string,load:()=>Promise<{default:ComponentType<P>}>){
  const initial=lazy(load)
  return function DeferredPanel(props:P){
    const [attempt,setAttempt]=useState(()=>({revision:0,View:initial}))
    const View=attempt.View
    return <PanelBoundary key={attempt.revision} label={translatedMessage(label)} onRetry={()=>setAttempt(current=>({revision:current.revision+1,View:lazy(load)}))}>
      <Suspense fallback={<section className="deferred-panel" role="status" aria-busy="true"><div><h3>{translatedMessage(label)}</h3><p>{t("Abriendo sección…")}</p></div></section>}><View {...props}/></Suspense>
    </PanelBoundary>
  }
}
