import {translatedMessage} from './locale.ts'
import {t} from './locale.ts'
import {useEffect,useRef,useState} from 'react'
import {api,type DeleteOutcome,type DeletePreview,type Server} from './api'
import {Modal} from './Modal'

export function ServerDeleteDialog({server,onClose,onDelete}: {server:Server;onClose:()=>void;onDelete:()=>Promise<DeleteOutcome>}) {
  const [preview,setPreview]=useState<DeletePreview|null>(null)
  const [loading,setLoading]=useState(true)
  const [deleting,setDeleting]=useState(false)
  const [error,setError]=useState<string|null>(null)
  const cancelRef=useRef<HTMLButtonElement>(null)
  useEffect(()=>{
    let active=true
    api.previewDeleteServer(server.id).then(value=>{if(active)setPreview(value)}).catch(cause=>{if(active)setError(String(cause))}).finally(()=>{if(active)setLoading(false)})
    return ()=>{active=false}
  },[server.id])
  const confirm=async()=>{
    setDeleting(true);setError(null)
    try {await onDelete();onClose()} catch(cause){setError(String(cause));setDeleting(false)}
  }
  return <Modal className="delete-dialog" labelledBy="delete-server-title" describedBy="delete-server-description" busy={deleting} initialFocusRef={cancelRef} onClose={onClose}>
    <h2 id="delete-server-title">{t("¿Eliminar «")}{server.name}»?</h2>
    <p id="delete-server-description">{loading ? t("Comprobando los archivos del servidor…") : preview?.managedFiles ? t("Se eliminarán sus archivos, imagen y copias de seguridad. Esta acción no se puede deshacer.") : preview ? t("Se quitará de la biblioteca junto con sus copias. La carpeta original importada y su icono se conservarán.") : t("No se pudo comprobar el alcance de la eliminación.")}</p>
    {preview && <div className="delete-preview-details"><span>{t("Carpeta")}</span><strong>{preview.directory}</strong><span>{t("Copias guardadas")}</span><strong>{preview.backupCount}</strong></div>}
    {error && <p className="form-error" role="alert">{translatedMessage(error)}</p>}
    <div className="world-confirm-actions"><button data-modal-close ref={cancelRef} disabled={deleting} onClick={onClose}>{t("Cancelar")}</button><button className="danger-button" disabled={loading || deleting || !preview} onClick={confirm}>{deleting ? t("Eliminando…") : t("Eliminar servidor")}</button></div>
  </Modal>
}
