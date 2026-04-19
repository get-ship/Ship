// Global annotation state — staged annotations persist across views.
// The session page adds annotations via the canvas/viewer overlays.
// The right drawer's chat tab reads them for sending to agents.

import { createContext, useContext, useState, useCallback, type ReactNode } from 'react'
import type { Annotation, StagedAnnotation } from './session/types'
import { daemonFetch } from '#/lib/daemon-config'
import { useDaemon } from './hooks/useDaemonData'
import { buildAnnotationEvent, type AnnotationEvent } from './session/annotation-helpers'

function stagedToEvent({ filePath, ann: a }: StagedAnnotation): AnnotationEvent {
  if (a.type === 'click') {
    return buildAnnotationEvent({
      kind: 'point',
      filePath,
      selector: a.selector,
      note: a.note || undefined,
      selection: a.text ? { kind: 'text', text: a.text } : undefined,
      createdAt: a.timestamp,
    })
  }
  if (a.type === 'box') {
    return buildAnnotationEvent({
      kind: 'region',
      filePath,
      rect: a.rect,
      note: a.note || undefined,
      selection: a.elements.length ? { kind: 'elements', text: a.elements.join(', ') } : undefined,
      createdAt: a.timestamp,
    })
  }
  return buildAnnotationEvent({
    kind: 'action',
    filePath,
    note: a.text ? `${a.action}: ${a.text}` : a.action,
    createdAt: a.timestamp,
  })
}

async function postFeedback(ev: AnnotationEvent): Promise<void> {
  const res = await daemonFetch('/api/runtime/feedback', {
    method: 'POST',
    body: JSON.stringify(ev),
  })
  if (!res.ok) throw new Error(`daemon: post feedback ${res.status}`)
}

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
  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0]

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
    for (const s of staged) {
      await postFeedback(stagedToEvent(s))
    }

    const text = message.trim()
    if (text) {
      await daemonFetch(`/api/events/emit`, {
        method: 'POST',
        body: JSON.stringify({
          event_type: 'studio.message',
          entity_id: activeWs?.name ?? 'studio',
          workspace_id: activeWs?.id,
          payload: { summary: text },
        }),
      })
    }

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
