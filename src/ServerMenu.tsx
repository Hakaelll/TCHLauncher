import {t} from './locale.ts'
import {ActionMenu} from './ActionMenu'
import type {Server} from './api'

export type ServerMenuTarget={server:Server;x:number;y:number;trigger:HTMLButtonElement}
export function ServerMenu({target,busy,onClose,onPin,onSettings,onDelete}: {target:ServerMenuTarget;busy:boolean;onClose:(restoreFocus?:boolean)=>void;onPin:()=>void;onSettings:()=>void;onDelete:()=>void}) {
  return <ActionMenu target={target} label={t("Acciones de {0}",target.server.name)} name={target.server.name} onClose={onClose} actions={[
    {key:'pin',label:target.server.pinned?t("Desfijar servidor"):t("Fijar servidor"),disabled:busy,run:onPin,icon:<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m15 3 6 6-4 1-4 5-4-4 5-4zM9 15l-6 6"/></svg>},
    {key:'settings',label:t("Configuración"),run:onSettings,icon:<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3"/><circle cx="15" cy="17" r="3"/></svg>},
    {key:'delete',label:t("Eliminar servidor"),separator:true,destructive:true,disabled:busy||!['stopped','crashed','error'].includes(target.server.state),run:onDelete,icon:<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M4 6h16M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7m4-7v7"/></svg>},
  ]}/>
}
