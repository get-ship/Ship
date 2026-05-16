// view-mcp.ts — temporary stubs for the view-authoring MCP surface.
//
// The backend half of view authoring (`job/view-authoring-backend`) is
// landing the real MCP tools + per-view SSE channels in parallel. This
// module fakes them so the frontend can build + render end-to-end
// without waiting. Delete this file (and `import { initViewMocks }` in
// main.ts) once the real surface ships.
//
// Mocks installed
//   - `view.read_source(name)`   → returns a hardcoded `.sui` sample
//   - `view.write_source(name)`  → resolves `{ ok: true }`
//   - `view.list_snapshots(name)`→ returns 3-5 fake snapshots
//   - SSE `sui:view:annotation`  → fires every 5s with a random selector
//
// The MCP shims attach as `window.__viewMcp.*`; consumers call them
// directly. The annotation timer dispatches on `document`, scoped to
// the current view via `window.__suiCurrentViewId`.

const SAMPLE_SOURCE = `<!-- mocked source — sample budget card view -->
<sui-card>
  <h2 slot="header">Budget</h2>
  <sui-progress prop:value={spent} prop:max={budget}></sui-progress>
  <p>{spent} / {budget} spent this month.</p>
</sui-card>
`

const SAMPLE_SELECTORS = [
  'sui-card',
  'sui-progress',
  'h2',
  'p',
]

const SAMPLE_LABELS = [
  'Added budget card',
  'Wired progress to spent/budget',
  'Renamed header to Budget',
  'Switched p to muted-foreground',
]

export interface ViewMcpStubs {
  readSource(name: string): Promise<{ source: string }>
  writeSource(name: string, source: string): Promise<{ ok: true }>
  listSnapshots(name: string): Promise<Array<{ id: string; label: string; ts: number }>>
  /** Test handle — fire a synthetic annotation event immediately. */
  fireAnnotation(selector?: string, label?: string): void
}

declare global {
  interface Window {
    __viewMcp?: ViewMcpStubs
    __viewMcpAnnotationTimer?: number
    /** Re-declared from apps/shell2/src/main.ts so the annotation event
     *  can stamp the current view-id without importing the shell module
     *  (which would create a circular dep). */
    __suiCurrentViewId?: string | null
  }
}

let timerId: number | null = null
let seq = 0

function dispatchAnnotation(selector: string, label: string): void {
  if (typeof document === 'undefined') return
  seq += 1
  document.dispatchEvent(new CustomEvent('sui:view:annotation', {
    detail: {
      id: `mock-ann-${seq}`,
      view: window.__suiCurrentViewId ?? null,
      selector,
      label,
      turn: seq,
      ts: Date.now(),
    },
    bubbles: true,
  }))
}

function startAnnotationTimer(): void {
  if (typeof window === 'undefined') return
  if (timerId !== null) return
  timerId = window.setInterval(() => {
    const sel = SAMPLE_SELECTORS[seq % SAMPLE_SELECTORS.length]!
    const label = SAMPLE_LABELS[seq % SAMPLE_LABELS.length]!
    dispatchAnnotation(sel, label)
  }, 5000)
  window.__viewMcpAnnotationTimer = timerId
}

export function stopAnnotationTimer(): void {
  if (timerId !== null && typeof window !== 'undefined') {
    window.clearInterval(timerId)
    timerId = null
    delete window.__viewMcpAnnotationTimer
  }
}

export function initViewMocks(): ViewMcpStubs {
  const stubs: ViewMcpStubs = {
    async readSource(_name) { return { source: SAMPLE_SOURCE } },
    async writeSource(_name, _source) { return { ok: true as const } },
    async listSnapshots(_name) {
      const now = Date.now()
      return [
        { id: 'snap-1', label: 'Initial scaffold', ts: now - 60 * 60 * 1000 },
        { id: 'snap-2', label: 'Added budget card', ts: now - 30 * 60 * 1000 },
        { id: 'snap-3', label: 'Wired progress', ts: now - 10 * 60 * 1000 },
        { id: 'snap-4', label: 'Renamed header', ts: now - 2 * 60 * 1000 },
      ]
    },
    fireAnnotation(selector = 'sui-card', label = 'Manual fire') {
      dispatchAnnotation(selector, label)
    },
  }
  if (typeof window !== 'undefined') {
    window.__viewMcp = stubs
    startAnnotationTimer()
  }
  return stubs
}
