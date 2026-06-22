// Data layer for <sui-view-picker> — hub-side chrome copy.
//
// Hub version of the shell's data layer (apps/shell/src/components/
// sui-view-picker/data.ts). Same fetch/save contract; same drag state
// machine; same partition/match/humanize helpers. The two differences
// are import seams: the hub uses the shared Connect client singleton
// from `apps/hub/src/lib/client.ts` (a `ShipConnectClient`, not the
// shell's `ops` wrapper), and the per-helper RUNTIME settings reader/
// writer is local to this file — the shell's helpers live in
// `apps/shell/src/ops.ts`, which the hub does not import.
//
// The Connect ops are unchanged (`ship.space.list` / `ship.space.
// activate`) — the shell's data layer also called them, just through
// the `ops` alias. `ship` here is the same-shape `createShipConnectClient`
// instance the hub composes everywhere (settings, account-menu, etc.),
// so the wire calls are identical; only the import name differs.

import { SettingScope } from '@ship/sdk/connect'
import { ops } from '../../lib/client'

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

/** Read one RUNTIME setting, parsed from its JSON bytes. `null` when unset. */
async function readSetting<T = unknown>(key: string): Promise<T | null> {
  try {
    const setting = await ops.settings.get({ key, scope: SettingScope.RUNTIME })
    if (setting.valueJson.length === 0) return null
    return JSON.parse(new TextDecoder().decode(setting.valueJson)) as T
  } catch {
    return null
  }
}

/** Write one RUNTIME setting as JSON bytes. */
async function writeSetting(key: string, value: unknown): Promise<void> {
  await ops.settings.set({
    key,
    valueJson: new TextEncoder().encode(JSON.stringify(value)),
    scope: SettingScope.RUNTIME,
  })
}

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

export async function fetchPins(_settingsUrl: string, key: string): Promise<string[]> {
  const raw = await readSetting<unknown>(key)
  if (Array.isArray(raw) && raw.every((v) => typeof v === 'string')) return raw as string[]
  return DEFAULT_PINS
}

export async function savePins(_settingsUrl: string, key: string, pins: string[]): Promise<void> {
  try {
    await writeSetting(key, pins)
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

export async function fetchViews(_url?: string): Promise<ApiView[]> {
  // Views are client-side routes (routing IS the catalog); the daemon discovery
  // op was quarantined (06153c1a5) — deleted stays deleted, no sneak-restore.
  // Cut the dead call; empty until the router→view-list integration lands. The
  // component picker's needs are served by the `search` RPC, not a views catalog.
  return ([] as ApiView[]).filter((v) => !HIDDEN_VIEWS.has(v.name))
}

export async function fetchProject(_url?: string): Promise<Project | null> {
  try {
    const resp = await ops.space.list({})
    const list = resp.spaces.map((s) => ({ id: s.id, name: s.name, path: s.path }))
    return list.find((p) => p.id === resp.activeId) ?? list[0] ?? null
  } catch { return null }
}

/** Full project list + the active project's id. Used by the unified
 *  view+project picker (per Task #54 — single popover for both). */
export async function fetchProjects(_url?: string): Promise<{ projects: Project[]; activeId: string | null }> {
  try {
    const resp = await ops.space.list({})
    return {
      projects: resp.spaces.map((s) => ({ id: s.id, name: s.name, path: s.path })),
      activeId: resp.activeId ?? null,
    }
  } catch { return { projects: [], activeId: null } }
}

/** Switch the daemon's active space. Returns true on success so the caller
 *  can refresh state. */
export async function switchProject(id: string): Promise<boolean> {
  try {
    await ops.space.activate({ id })
    return true
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
