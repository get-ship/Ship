// <ship-activity-runs> — @ship/home · recent pipeline runs on the hub `/`.
//
// Reads ship.pipelines.summaries() (Connect SDK LiveSource) and renders
// the active_runs list. The shell's studio-pipelines page owns the
// full summaries view; this card is the dashboard slice — recent runs,
// bounded, with a "view all" link.
//
// On loading + zero runs the component renders the empty-state. The
// summaries payload is `unknown` from the SDK; the controller reads
// the shape defensively (the shell's controller does the same — see
// apps/shell/routes/studio/pipelines/controller.ts).
//
// No CLI work — dispatch lives in the studio surface, not the home.

import { controller, derived, live, unwrap, signal } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-activity-runs.sui'

interface RunRow {
  slug: string
  agent?: string
  job_id: string
  state: string
  updated_at: string
  branch?: string
  steps?: Array<{ id: string; state: string; step_type: string; agent?: string }>
}

interface Summaries {
  defs?: unknown[]
  active_runs?: RunRow[]
  providers?: unknown[]
  stats_subtitle?: string
  workspace_id?: string
}

const MIN = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

function readRuns(value: unknown): RunRow[] {
  const v = unwrap(value as LiveSignal<Partial<Summaries>>)
  if (!v || !Array.isArray(v.active_runs)) return []
  return v.active_runs
}

function formatRelative(iso: string): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = Date.now() - t
  const abs = Math.abs(diff)
  if (abs < MIN) return 'just now'
  if (abs < HOUR) return `${Math.floor(abs / MIN)}m ago`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ago`
  return `${Math.floor(abs / DAY)}d ago`
}

function stateTone(state: string): string {
  if (state === 'succeeded' || state === 'success' || state === 'ok') return 'success'
  if (state === 'failed' || state === 'error' || state === 'err') return 'destructive'
  if (state === 'running' || state === 'active' || state === 'pending') return 'info'
  if (state === 'queued' || state === 'waiting') return 'warning'
  return 'muted'
}

function stateLabel(state: string): string {
  return state || 'idle'
}

interface RunView {
  key: string
  slug: string
  state: string
  agent: string
  branch: string
  updatedLabel: string
  tone: string
  stateLabel: string
}

export default controller({
  scope: ({ limit }: ViewScope<Scope>, host?: Element) => {
    const summaries = live(() => ship.pipelines.summaries() as LiveSource<Summaries>)
    const rows = derived<RunRow[]>(() => {
      const all = readRuns(summaries)
      const n = Number.isFinite(limit()) ? Math.max(0, Math.floor(limit())) : 6
      return all.slice(0, n)
    })
    const hasRuns = derived<boolean>(() => rows().length > 0)
    const sub = derived<string>(() => {
      const v = unwrap(summaries as LiveSignal<Partial<Summaries>>)
      const r = rows().length
      if (!v) return 'loading recent runs…'
      if (r === 0) return 'no active runs'
      if (r === 1) return '1 active run'
      return `${r} active runs`
    })
    // A tick signal (30s) so the relative time labels re-render without
    // an SDK round-trip. The derived `items` reads the tick so it
    // re-evaluates on every interval.
    const tick = signal<number>(Date.now())

    const items = derived<RunView[]>(() => {
      // Subscribe to the tick so the derived re-runs every 30s.
      tick()
      return rows().map((r) => ({
        key: r.slug,
        slug: r.slug,
        state: r.state,
        agent: r.agent ?? 'pipeline',
        branch: r.branch ?? '',
        updatedLabel: formatRelative(r.updated_at),
        tone: stateTone(r.state),
        stateLabel: stateLabel(r.state),
      }))
    })

    const onOpen = (e: MouseEvent, slug: string): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-pipeline-open', {
          detail: { slug },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { summaries, rows, hasRuns, items, sub, tick, onOpen }
  },

  mount(state) {
    const tickSig = state.$signals.tick
    const stop = setInterval(() => tickSig.set(Date.now()), 30_000)
    return () => clearInterval(stop)
  },
})
