// Data layer for <sui-view-picker> — fetch/save + view filter +
// drag-pin/unpin state machine. Split from render.ts so each module
// stays under the 300-line engineering cap.

export interface ApiView {
  name: string
  type: string
  source: string
  rail?: { label: string; icon: string }
}

export interface Project {
  id: string
  name: string
  path: string
  label?: string
}

export const ZONE_PINNED_END = 1
export const ZONE_ALL = 2

const DEFAULT_PINS = ['dashboard', 'agents', 'skills', 'pipelines', 'git']

export function humanize(id: string): string {
  return id.replace(/-/g, ' ').split(' ')
    .map((w) => (w ? w[0]!.toUpperCase() + w.slice(1) : w)).join(' ')
}

export function matchView(v: ApiView, filterVal: string): boolean {
  const f = filterVal.trim().toLowerCase()
  if (!f) return true
  const label = (v.rail?.label ?? humanize(v.name)).toLowerCase()
  const src = (v.source ?? '').toLowerCase()
  return label.includes(f) || src.includes(f) || v.name.toLowerCase().includes(f)
}

export function partition(args: { views: ApiView[]; pins: string[]; filter: string }): {
  pinned: ApiView[]; others: ApiView[]
} {
  const { views, pins, filter } = args
  const byName = new Map(views.map((v) => [v.name, v]))
  const pinned = pins.map((n) => byName.get(n))
    .filter((v): v is ApiView => !!v && matchView(v, filter))
  const pinSet = new Set(pins)
  const others = views
    .filter((v) => !pinSet.has(v.name) && matchView(v, filter))
    .sort((a, b) => humanize(a.name).localeCompare(humanize(b.name)))
  return { pinned, others }
}

export async function fetchPins(settingsUrl: string, key: string): Promise<string[]> {
  try {
    const resp = await fetch(settingsUrl, { credentials: 'same-origin' })
    if (!resp.ok) return DEFAULT_PINS
    const body = (await resp.json()) as { data?: Record<string, unknown> }
    const raw = body.data?.[key]
    if (typeof raw === 'string') {
      try {
        const parsed = JSON.parse(raw)
        if (Array.isArray(parsed) && parsed.every((v) => typeof v === 'string')) return parsed as string[]
      } catch { /* default */ }
    } else if (Array.isArray(raw) && raw.every((v) => typeof v === 'string')) {
      return raw as string[]
    }
    return DEFAULT_PINS
  } catch { return DEFAULT_PINS }
}

export async function savePins(settingsUrl: string, key: string, pins: string[]): Promise<void> {
  try {
    await fetch(settingsUrl, {
      method: 'PUT', credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ key, value: JSON.stringify(pins) }),
    })
  } catch { /* fire-and-forget */ }
}

// Views the launch shell never surfaces — wizard, samples, dashboards we
// haven't shipped. Filtered out of /api/v1/views before render so the
// picker only shows the curated launch surface.
const HIDDEN_VIEWS = new Set([
  'setup', 'hello', 'sui-demo', 'demos', 'counter',
  'dashboard', 'overview', 'sessions', 'mesh', 'specs', 'workspaces',
  'pipelines-board',
])

export async function fetchViews(url: string): Promise<ApiView[]> {
  try {
    const resp = await fetch(url, { credentials: 'same-origin' })
    if (!resp.ok) return []
    const body = (await resp.json()) as { data?: { views?: ApiView[] }; views?: ApiView[] }
    const inferred = body.data?.views ?? body.views ?? []
    return (inferred ?? []).filter((v) => !HIDDEN_VIEWS.has(v.name))
  } catch { return [] }
}

export async function fetchProject(url: string): Promise<Project | null> {
  try {
    const resp = await fetch(url, { credentials: 'same-origin' })
    if (!resp.ok) return null
    const body = (await resp.json()) as { projects?: Project[]; active?: string }
    const list = body.projects ?? []
    return list.find((p) => p.id === body.active) ?? list[0] ?? null
  } catch { return null }
}

/** Full project list + the active project's id. Used by the unified
 *  view+project picker (per Task #54 — single popover for both). */
export async function fetchProjects(url: string): Promise<{ projects: Project[]; activeId: string | null }> {
  try {
    const resp = await fetch(url, { credentials: 'same-origin' })
    if (!resp.ok) return { projects: [], activeId: null }
    const body = (await resp.json()) as { projects?: Project[]; active?: string }
    return { projects: body.projects ?? [], activeId: body.active ?? null }
  } catch { return { projects: [], activeId: null } }
}

/** POST /api/v1/projects/<id>/activate — switches the daemon's active
 *  project. Returns true on success so the caller can refresh state. */
export async function switchProject(id: string): Promise<boolean> {
  try {
    const resp = await fetch(
      `/api/v1/projects/${encodeURIComponent(id)}/activate`,
      { method: 'POST', credentials: 'same-origin' },
    )
    return resp.ok
  } catch { return false }
}

// ── drag state + handlers ──────────────────────────────────────────
export interface DragState {
  name: string | null
  overName: string | null
  overZone: number | null
  fromPinned: boolean
}

export interface DragCtx {
  state: DragState
  pins: () => string[]
  setPins: (next: string[]) => void
  persist: (next: string[]) => Promise<void>
  repaint: () => void
  emit: (pins: string[]) => void
}

function resetDrag(s: DragState): void {
  s.name = null; s.overName = null; s.overZone = null; s.fromPinned = false
}

export function makeDrag(ctx: DragCtx) {
  const { state, pins, setPins, persist, repaint, emit } = ctx
  const commit = async (next: string[]) => {
    setPins(next); await persist(next); emit(next)
  }
  return {
    onDragStart(e: DragEvent, name: string, fromPinned: boolean): void {
      state.name = name; state.fromPinned = fromPinned
      e.dataTransfer?.setData('text/plain', name)
      if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move'
    },
    onDragOverRow(e: DragEvent, name: string): void {
      if (!state.name) return
      e.preventDefault()
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
      if (state.overName !== name || state.overZone !== null) {
        state.overName = name; state.overZone = null; repaint()
      }
    },
    onDragOverZone(e: DragEvent, zoneId: number): void {
      if (!state.name) return
      e.preventDefault()
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
      if (state.overZone !== zoneId || state.overName !== null) {
        state.overName = null; state.overZone = zoneId; repaint()
      }
    },
    async onDropRow(e: DragEvent, target: string, intoPinned: boolean): Promise<void> {
      e.preventDefault()
      const src = state.name; resetDrag(state)
      if (!src || src === target) { repaint(); return }
      const cur = pins()
      if (intoPinned && cur.includes(src)) {
        const without = cur.filter((n) => n !== src); const idx = without.indexOf(target)
        await commit([...without.slice(0, idx), src, ...without.slice(idx)])
      } else if (intoPinned && !cur.includes(src)) {
        const idx = cur.indexOf(target)
        await commit([...cur.slice(0, idx), src, ...cur.slice(idx)])
      } else if (!intoPinned && cur.includes(src)) {
        await commit(cur.filter((n) => n !== src))
      } else { repaint() }
    },
    async onDropZone(e: DragEvent, zoneId: number): Promise<void> {
      e.preventDefault()
      const src = state.name; resetDrag(state)
      if (!src) { repaint(); return }
      const cur = pins()
      if (zoneId === ZONE_PINNED_END) {
        await commit(cur.includes(src) ? [...cur.filter((n) => n !== src), src] : [...cur, src])
      } else if (zoneId === ZONE_ALL) {
        await commit(cur.filter((n) => n !== src))
      } else { repaint() }
    },
    onDragEnd(): void { resetDrag(state); repaint() },
  }
}
