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
    <h2 id="delete-server-title">¿Eliminar «{server.name}»?</h2>
    <p id="delete-server-description">{loading ? 'Comprobando los archivos del servidor…' : preview?.managedFiles ? 'Se eliminarán sus archivos, imagen y copias de seguridad. Esta acción no se puede deshacer.' : preview ? 'Se quitará de la biblioteca junto con sus copias. La carpeta original importada y su icono se conservarán.' : 'No se pudo comprobar el alcance de la eliminación.'}</p>
    {preview && <div className="delete-preview-details"><span>Carpeta</span><strong>{preview.directory}</strong><span>Copias guardadas</span><strong>{preview.backupCount}</strong></div>}
    {error && <p className="form-error" role="alert">{error}</p>}
    <div className="world-confirm-actions"><button ref={cancelRef} disabled={deleting} onClick={onClose}>Cancelar</button><button className="danger-button" disabled={loading || deleting || !preview} onClick={confirm}>{deleting ? 'Eliminando…' : 'Eliminar servidor'}</button></div>
  </Modal>
}
