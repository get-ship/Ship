// Canvas ↔ kernel bridge. Subscribes to the shell's window.ship kernel-
// event channel (the singleton EventSource opened in main.ts) and fans
// canvas.content / canvas.morph / canvas.clear payloads out to per-
// canvas-id subscribers. Interaction + annotation events flow outward
// via POST /api/v1/events/emit.

interface ShipApi {
  subscribe(ns: string, h: (e: { event_type: string; payload: unknown }) => void): () => void
}
function shipApi(): ShipApi | null {
  return (window as unknown as { ship?: ShipApi }).ship ?? null
}

export interface CanvasEventDetail {
  canvas_id: string
  kind?: string
  [k: string]: unknown
}

export interface CanvasContentPayload { canvas_id: string; sui: string }
export interface CanvasMorphPayload {
  canvas_id: string
  target: string
  patch: Record<string, unknown>
}
export interface CanvasSubscriber {
  onContent?: (p: CanvasContentPayload) => void
  onMorph?:   (p: CanvasMorphPayload) => void
  onClear?:   (p: { canvas_id: string }) => void
}

export interface CanvasBridge {
  subscribe(canvasId: string, sub: CanvasSubscriber): () => void
  postInteraction(detail: CanvasEventDetail): Promise<void>
  postAnnotations(canvasId: string, agentId: string, annotations: unknown[]): Promise<void>
  postSave(canvasId: string, contentHtml: string): Promise<{ ok: boolean; id?: string }>
}

let _bridge: CanvasBridge | null = null

export function getCanvasBridge(): CanvasBridge {
  if (_bridge) return _bridge
  _bridge = createBridge()
  return _bridge
}

function createBridge(): CanvasBridge {
  const subs = new Map<string, Set<CanvasSubscriber>>()
  let off: (() => void) | null = null

  function ensureSubscribed(): void {
    if (off) return
    const api = shipApi()
    if (!api) {
      console.warn('[canvas-bridge] window.ship not ready; canvas wire inert')
      return
    }
    off = api.subscribe('canvas.', (event) => {
      const payload = event.payload as { canvas_id?: string } | null
      const canvasId = payload?.canvas_id ?? ''
      const targets = subs.get(canvasId)
      if (!targets || targets.size === 0) return
      for (const sub of targets) {
        try {
          if (event.event_type === 'canvas.content' && sub.onContent) {
            sub.onContent(payload as CanvasContentPayload)
          } else if (event.event_type === 'canvas.morph' && sub.onMorph) {
            sub.onMorph(payload as CanvasMorphPayload)
          } else if (event.event_type === 'canvas.clear' && sub.onClear) {
            sub.onClear({ canvas_id: canvasId })
          }
        } catch (err) {
          console.warn('[canvas-bridge] subscriber error:', err)
        }
      }
    })
  }

  function subscribe(canvasId: string, sub: CanvasSubscriber): () => void {
    if (!canvasId) return () => undefined
    let set = subs.get(canvasId)
    if (!set) { set = new Set(); subs.set(canvasId, set) }
    set.add(sub)
    ensureSubscribed()
    return () => {
      const s = subs.get(canvasId)
      if (!s) return
      s.delete(sub)
      if (s.size === 0) subs.delete(canvasId)
      if (subs.size === 0 && off) { off(); off = null }
    }
  }

  // Canvas interaction/annotation emit is disabled for launch — the
  // events/emit endpoint was culled (no Connect replacement yet). These
  // stay no-ops so the canvas remains interactive without 404s; re-enable
  // when EventsService.Emit lands.
  async function postInteraction(_detail: CanvasEventDetail): Promise<void> {
    /* disabled for launch — events/emit culled */
  }

  async function postAnnotations(
    _canvasId: string,
    _agentId: string,
    _annotations: unknown[],
  ): Promise<void> {
    /* disabled for launch — events/emit culled */
  }

  async function postSave(canvasId: string, contentHtml: string): Promise<{ ok: boolean; id?: string }> {
    const fd = new FormData()
    const blob = new Blob([contentHtml], { type: 'text/html' })
    fd.append('file', blob, `${canvasId}.html`)
    fd.append('collection_id', 'canvas')
    fd.append('item_id', canvasId)
    const resp = await fetch('/api/v1/artifacts', {
      method: 'POST',
      credentials: 'same-origin',
      body: fd,
    })
    if (!resp.ok) {
      const text = await resp.text().catch(() => '')
      throw new Error(`canvas.save failed: ${resp.status} ${text}`)
    }
    const body = await resp.json().catch(() => ({})) as { data?: { id?: string } }
    void emit('canvas.saved', { canvas_id: canvasId, artifact_id: body.data?.id ?? '' })
    return { ok: true, ...(body.data?.id !== undefined ? { id: body.data.id } : {}) }
  }

  return { subscribe, postInteraction, postAnnotations, postSave }
}
