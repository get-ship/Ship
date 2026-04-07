// Global annotation state — staged annotations persist across views.
// The session page adds annotations via the canvas/viewer overlays.
// The right drawer's chat tab reads them for sending to agents.

import { createContext, useContext, useState, useCallback, type ReactNode } from 'react'
import type { Annotation, StagedAnnotation } from './session/types'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from './hooks/useDaemonData'

let seq = 1
function genId(): string {
  return `ann-${Date.now().toString(36)}-${(seq++).toString(36)}`
}

interface AnnotationState {
  staged: StagedAnnotation[]
  stagedCount: number
  addAnnotation: (filePath: string, ann: Annotation) => void
  removeAnnotation: (id: string) => void
  clearAll: () => void
  sendToAgent: (message: string) => Promise<void>
}

const AnnotationCtx = createContext<AnnotationState | null>(null)

export function AnnotationProvider({ children }: { children: ReactNode }) {
  const [staged, setStaged] = useState<StagedAnnotation[]>([])
  const { workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')

  const addAnnotation = useCallback((filePath: string, ann: Annotation) => {
    const id = ann.id || genId()
    const withId = { ...ann, id }
    setStaged((prev) => [...prev, { filePath, id, ann: withId }])
  }, [])

  const removeAnnotation = useCallback((id: string) => {
    setStaged((prev) => prev.filter((s) => s.id !== id))
  }, [])

  const clearAll = useCallback(() => setStaged([]), [])

  const sendToAgent = useCallback(async (message: string) => {
    const annotations = staged.map(({ filePath, ann: a }) => {
      if (a.type === 'click') return { filePath, type: 'click', selector: a.selector, text: a.text, note: a.note, x: a.x, y: a.y }
      if (a.type === 'box') return { filePath, type: 'box', rect: a.rect, elements: a.elements, note: a.note }
      return { filePath, type: 'action', action: a.action, text: a.text }
    })

    await fetch(`${DAEMON_BASE_URL}/api/events/emit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        event_type: 'studio.message',
        entity_id: activeWs?.branch ?? 'studio',
        workspace_id: activeWs?.id,
        payload: {
          summary: message || undefined,
          annotations,
          annotation_count: annotations.length,
        },
      }),
    })

    setStaged([])
  }, [staged, activeWs])

  return (
    <AnnotationCtx value={{
      staged,
      stagedCount: staged.length,
      addAnnotation,
      removeAnnotation,
      clearAll,
      sendToAgent,
    }}>
      {children}
    </AnnotationCtx>
  )
}

export function useGlobalAnnotations(): AnnotationState {
  const ctx = useContext(AnnotationCtx)
  if (!ctx) throw new Error('useGlobalAnnotations must be used within AnnotationProvider')
  return ctx
}
