import { controller, signal, derived, ref, live } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '../ops'
import { route as routeFactory } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'

const PAIR_KEY_PREFIX = 'ship.canvas.pair.'

type AnnMode = 'off' | 'point' | 'region'

interface AnnotationRow {
  id: string
  kind: string
  note: string
}

interface AgentSummary { id: string; name?: string; description?: string }

interface CanvasHostEl extends HTMLElement {
  setContent(sui: string): void
  applyMorph(target: string, patch: Record<string, unknown>): void
  clear(): void
  getContentHtml(): string
}

interface AnnotationLayerHost extends HTMLElement {
  send: () => Promise<void>
  clear: () => void
  staged: ReadonlyArray<{
    id: string
    kind: 'point' | 'region'
    note?: string
    target?: { view_id?: string; selector?: string; rect?: [number, number, number, number] }
  }>
}

function readPair(canvasId: string): string {
  try { return localStorage.getItem(PAIR_KEY_PREFIX + canvasId) ?? '' } catch { return '' }
}

function writePair(canvasId: string, agentId: string): void {
  try {
    if (agentId) localStorage.setItem(PAIR_KEY_PREFIX + canvasId, agentId)
    else localStorage.removeItem(PAIR_KEY_PREFIX + canvasId)
  } catch { /* private mode */ }
}

function newCanvasId(): string {
  const c = (typeof crypto !== 'undefined' && crypto?.randomUUID) ? crypto.randomUUID() : ''
  return c || `c-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const agentsSummaries = live(() => ship.agents.summaries())
    const artifactsList = live(() => ship.artifacts.list())
    const route = routeFactory()
    const initialId = route.query.canvas() || newCanvasId()
    const canvasId = signal(initialId)
    const pairedAgentId = signal(readPair(initialId))
    const recentOpen = signal(false)
    const annMode = signal<AnnMode>('off')
    const annThread = signal<AnnotationRow[]>([])
    const statusText = signal('ready')
    const canvasEl = ref<CanvasHostEl>(null)
    const annLayerEl = ref<AnnotationLayerHost>(null)

    const shortId = derived(() => canvasId().slice(0, 8))
    const annCount = derived(() => annThread().length)

    // <data live> only supports zero-arg calls — pull the full artifacts
    // list via the SSE channel, then filter to collection_id === 'canvas'
    // client-side. The view-host environment (/sui/canvas) doesn't inject
    // a `ship` object into the controller scope, so an in-controller
    // live(ship.artifacts.list(...)) would throw. The <data> directive is
    // the host-portable path.
    const recentItems = derived(() => {
      const v = artifactsList() as {
        kind?: string
        value?: { items?: Array<{ id: string; name?: string; created_at?: string; collection_id?: string | null }> }
      }
      if (v?.kind !== 'ok') return []
      return (v.value?.items ?? []).filter((a) => a.collection_id === 'canvas')
    })

    const pairedAgentLabel = derived(() => {
      const id = pairedAgentId()
      if (!id) return ''
      const list = agentsSummaries() as { kind?: string; value?: { items?: AgentSummary[] } }
      if (list?.kind !== 'ok') return id
      const a = list.value?.items?.find((x) => x.id === id)
      return a?.name || id
    })

    return {
      canvasId, pairedAgentId, recentOpen, annMode, annThread, statusText,
      canvasEl, annLayerEl, shortId, annCount, pairedAgentLabel,
      recentItems, agentsSummaries, artifactsList,
    }
  },

  on: {
    pair: (s, agentId: string) => {
      const cid = s.canvasId
      s.navigate('/canvas?canvas=' + encodeURIComponent(cid))
      s.pairedAgentId = agentId
      writePair(cid, agentId)
    },
    unpair: (s) => {
      writePair(s.canvasId, '')
      s.pairedAgentId = ''
    },
    toggleRecent: (s) => { s.recentOpen = !s.recentOpen },
    newCanvas: (s) => {
      const id = newCanvasId()
      s.canvasId = id
      s.pairedAgentId = ''
      s.annThread = []
      s.statusText = 'new canvas'
      s.navigate('/canvas?canvas=' + encodeURIComponent(id))
    },
    setAnnMode: (s, mode: AnnMode) => { s.annMode = mode },
    clearAnnotations: (s) => {
      if (s.annLayerEl) s.annLayerEl.clear()
      s.annThread = []
    },
    sendAnnotations: async (s) => {
      const cid = s.canvasId
      const agentId = s.pairedAgentId
      const layer = s.annLayerEl
      const staged = layer ? Array.from(layer.staged) : []
      if (staged.length === 0) { s.statusText = 'no annotations to send'; return }
      const resp = await fetch('/api/v1/events/emit', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          event_type: 'canvas.annotation',
          entity_id: cid,
          actor_id: 'studio',
          payload: { canvas_id: cid, agent_id: agentId, annotations: staged },
        }),
      })
      if (!resp.ok) { s.statusText = 'send failed: ' + resp.status; return }
      if (layer) layer.clear()
      s.annThread = []
      s.statusText = `sent ${staged.length} annotation${staged.length === 1 ? '' : 's'}`
    },
    saveCanvas: async (s) => {
      const cid = s.canvasId
      const el = s.canvasEl
      if (!el) { s.statusText = 'canvas not ready'; return }
      const html = el.getContentHtml()
      if (!html || html.trim().length === 0) {
        s.statusText = 'nothing to save (empty canvas)'
        return
      }
      const fd = new FormData()
      const blob = new Blob([html], { type: 'text/html' })
      fd.append('file', blob, `${cid}.html`)
      fd.append('collection_id', 'canvas')
      fd.append('item_id', cid)
      const resp = await fetch('/api/v1/artifacts', {
        method: 'POST', credentials: 'same-origin', body: fd,
      })
      if (!resp.ok) {
        const t = await resp.text().catch(() => '')
        s.statusText = `save failed: ${resp.status} ${t.slice(0, 80)}`
        return
      }
      const body = await resp.json().catch(() => ({})) as { data?: { id?: string } }
      s.statusText = `saved · artifact ${(body.data?.id || '').slice(0, 8)}`
    },
    loadArtifact: async (s, artifactId: string) => {
      const resp = await fetch(`/api/v1/artifacts/${encodeURIComponent(artifactId)}`, {
        credentials: 'same-origin',
      })
      if (!resp.ok) { s.statusText = `load failed: ${resp.status}`; return }
      const html = await resp.text()
      const el = s.canvasEl
      if (!el) { s.statusText = 'canvas not mounted'; return }
      el.setContent(html)
      s.recentOpen = false
      s.statusText = `loaded · artifact ${artifactId.slice(0, 8)}`
    },
  },

  mount: (state, host) => {
    function refreshThread(): void {
      const layer = state.annLayerEl
      const staged = layer && Array.isArray(layer.staged) ? layer.staged : []
      type StagedItem = { id: string; kind: string; note?: string }
      state.annThread = staged.map((a: StagedItem) => ({
        id: a.id,
        kind: a.kind,
        note: a.note ?? '',
      }))
    }

    const onStaged = (e: Event): void => {
      const d = (e as CustomEvent<{
        id?: string; kind?: string; note?: string
      }>).detail
      if (!d?.id) return
      state.annThread = [
        ...state.annThread,
        { id: d.id, kind: d.kind ?? 'point', note: d.note ?? '' },
      ]
    }
    host.addEventListener('sui-annotation-staged', onStaged)

    // When the agent emits content the canvas dispatches sui-canvas-content;
    // surface it in the status footer as confirmation.
    const onContent = (e: Event): void => {
      const d = (e as CustomEvent<{ length?: number }>).detail
      state.statusText = `content updated (${d?.length ?? 0} bytes)`
    }
    const onMorph = (e: Event): void => {
      const d = (e as CustomEvent<{ target?: string; miss?: boolean }>).detail
      state.statusText = d?.miss
        ? `morph miss: ${d.target}`
        : `morph applied: ${d?.target}`
    }
    host.addEventListener('sui-canvas-content', onContent)
    host.addEventListener('sui-canvas-morph', onMorph)

    // Test hook — programmatic probe from web-qa uses these to drive
    // setContent / applyMorph without spinning up an agent. Lives at
    // window.__shipCanvas so the QA probe can call into the surface.
    interface ShipCanvasGlobals {
      setContent(sui: string): void
      applyMorph(target: string, patch: Record<string, unknown>): void
      clear(): void
      getContentHtml(): string
      canvasId(): string
      pairedAgentId(): string
    }
    ;(window as unknown as { __shipCanvas?: ShipCanvasGlobals }).__shipCanvas = {
      setContent: (sui) => state.canvasEl?.setContent(sui),
      applyMorph: (target, patch) => state.canvasEl?.applyMorph(target, patch),
      clear: () => state.canvasEl?.clear(),
      getContentHtml: () => state.canvasEl?.getContentHtml() ?? '',
      canvasId: () => state.canvasId,
      pairedAgentId: () => state.pairedAgentId,
    }

    // Refresh thread once on mount so a route-back finds existing pins
    // (the annotation layer rebuilds its own state from its endpoint;
    // staged: [] by default, so this is currently a no-op but documents
    // intent).
    refreshThread()

    return () => {
      host.removeEventListener('sui-annotation-staged', onStaged)
      host.removeEventListener('sui-canvas-content', onContent)
      host.removeEventListener('sui-canvas-morph', onMorph)
      try { delete (window as unknown as { __shipCanvas?: unknown }).__shipCanvas } catch { /* noop */ }
    }
  },
})
