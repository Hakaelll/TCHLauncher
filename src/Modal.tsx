import { useLayoutEffect, useRef, type PointerEvent, type ReactNode, type RefObject } from 'react'
import { createPortal } from 'react-dom'

type ModalProps = {
  children: ReactNode
  labelledBy: string
  describedBy?: string
  className?: string
  busy?: boolean
  initialFocusRef?: RefObject<HTMLButtonElement | null>
  onClose: () => void
}

export function Modal({children,labelledBy,describedBy,className = '',busy = false,initialFocusRef,onClose}: ModalProps) {
  const dialogRef = useRef<HTMLDialogElement>(null)
  const backdropPressed = useRef(false)

  useLayoutEffect(() => {
    const dialog = dialogRef.current
    if (!dialog) return
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
    // The top layer keeps the dialog anchored to the viewport, outside page transforms and scroll containers.
    dialog.showModal()
    const initialFocus = initialFocusRef?.current ?? dialog.querySelector<HTMLElement>('[autofocus]')
    initialFocus?.focus({preventScroll:true})
    return () => {
      dialog.close()
      if (previousFocus?.isConnected) previousFocus.focus({preventScroll:true})
    }
  },[initialFocusRef])

  const outsideDialog = (event: PointerEvent<HTMLDialogElement>) => {
    if (event.target !== event.currentTarget) return false
    const rect = event.currentTarget.getBoundingClientRect()
    return event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom
  }

  return createPortal(<dialog ref={dialogRef} className={`close-dialog modal-dialog ${className}`} aria-labelledby={labelledBy} aria-describedby={describedBy} aria-busy={busy || undefined}
    onCancel={event => { event.preventDefault(); if (!busy) onClose() }}
    onPointerDown={event => { backdropPressed.current = outsideDialog(event) }}
    onPointerUp={event => { if (backdropPressed.current && outsideDialog(event) && !busy) onClose(); backdropPressed.current = false }}
    onPointerCancel={() => { backdropPressed.current = false }}>
    {children}
  </dialog>,document.body)
}
