// Right-click context menu for session artifacts.
// Supports copy path, promote/demote, and delete with confirmation.

import { useState, useRef, useEffect, useCallback } from 'react'
import { Trash2, Copy, CloudUpload, Check } from 'lucide-react'
import type { SessionFile } from './types'

export interface ArtifactMenuState {
  file: SessionFile
  x: number
  y: number
}

interface Props {
  menu: ArtifactMenuState
  onDelete: (path: string) => void
  onPromote?: (path: string) => void
  onClose: () => void
}

export function ArtifactContextMenu({ menu, onDelete, onPromote, onClose }: Props) {
  const menuRef = useRef<HTMLDivElement>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  useEffect(() => {
    if (!menuRef.current) return
    const el = menuRef.current
    const rect = el.getBoundingClientRect()
    if (rect.right > window.innerWidth) el.style.left = `${menu.x - rect.width}px`
    if (rect.bottom > window.innerHeight) el.style.top = `${menu.y - rect.height}px`
  }, [menu.x, menu.y])

  const handleCopyPath = useCallback(() => {
    void navigator.clipboard.writeText(menu.file.path)
    onClose()
  }, [menu.file.path, onClose])

  const handlePromote = useCallback(() => {
    onPromote?.(menu.file.path)
    onClose()
  }, [menu.file.path, onPromote, onClose])

  const handleDelete = useCallback(() => {
    onDelete(menu.file.path)
    onClose()
  }, [menu.file.path, onDelete, onClose])

  const isSaved = menu.file.tier === 'both' || menu.file.tier === 'project'
  const canPromote = !isSaved && onPromote

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose() }} />
      <div
        ref={menuRef}
        style={{ top: menu.y, left: menu.x }}
        className="fixed z-50 w-48 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in zoom-in-95 duration-100"
      >
        <div className="px-3 py-1.5 border-b border-border flex items-center gap-1.5">
          {isSaved && <Check className="size-3 text-emerald-500 shrink-0" />}
          <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide truncate">
            {menu.file.name}
          </span>
        </div>

        {confirmDelete ? (
          <div className="p-2 space-y-2">
            <p className="text-xs text-foreground">Delete {menu.file.name}?</p>
            <div className="flex items-center gap-1.5">
              <button
                onClick={onClose}
                className="flex-1 px-2 py-1 text-xs rounded border border-border hover:bg-muted/50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleDelete}
                className="flex-1 px-2 py-1 text-xs rounded bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        ) : (
          <div className="p-1">
            {canPromote && (
              <button
                onClick={handlePromote}
                className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-muted/50 transition-colors"
              >
                <CloudUpload className="size-3.5 shrink-0 text-primary" />
                <span className="text-xs text-foreground">Save to project</span>
              </button>
            )}
            {isSaved && (
              <div className="flex items-center gap-2 px-2.5 py-1.5">
                <Check className="size-3.5 shrink-0 text-emerald-500" />
                <span className="text-xs text-muted-foreground">Saved to project</span>
              </div>
            )}
            <button
              onClick={handleCopyPath}
              className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-muted/50 transition-colors"
            >
              <Copy className="size-3.5 shrink-0 text-muted-foreground" />
              <span className="text-xs text-foreground">Copy path</span>
            </button>
            <div className="h-px bg-border mx-2 my-1" />
            <button
              onClick={() => setConfirmDelete(true)}
              className="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-md text-left hover:bg-destructive/10 transition-colors"
            >
              <Trash2 className="size-3.5 shrink-0 text-destructive" />
              <span className="text-xs text-destructive">Delete</span>
            </button>
          </div>
        )}
      </div>
    </>
  )
}
