import { controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-session-list.sui'

// ── Session row contract ──────────────────────────────────────────────
// One row per session in the `sessions` projection. `rows` arrives as a
// JSON string (SUI props carry no array/object type — same convention as
// ship-approval-card's `request`). Production passes the live projection
// rows through this prop; nothing is hardcoded here.
export type SessionStatus =
  | 'active'
  | 'suspended'
  | 'completed'
  | 'failed'
  | 'cancelled'

export interface SessionRow {
  id: string
  status: SessionStatus
  harness: string // floki | claude | codex | …
  agent: string
  input: string
  model?: string
  time?: string
  turns?: number
  cost?: string
  reason?: string
  reasonTone?: 'warn' | 'fail' | 'mute'
  child?: string // e.g. "child of code-reviewer"
}

// Row enriched with the precomputed class strings the template binds, so the
// markup stays declarative (no status→class branching in the template).
interface EnrichedRow extends SessionRow {
  dotClass: string
  labelClass: string
  harnessClass: string
  reasonClass: string
  turnsLabel: string
}

const STATUSES: SessionStatus[] = [
  'active',
  'suspended',
  'completed',
  'failed',
  'cancelled',
]

function parseRows(raw: unknown): SessionRow[] {
  if (Array.isArray(raw)) return raw as SessionRow[]
  if (typeof raw !== 'string' || raw === '') return []
  try {
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? (parsed as SessionRow[]) : []
  } catch {
    return []
  }
}

function enrich(r: SessionRow): EnrichedRow {
  const pulse = r.status === 'active' ? ' pulse' : ''
  const turns = typeof r.turns === 'number' ? r.turns : 0
  return {
    ...r,
    dotClass: `dot d-${r.status}${pulse}`,
    labelClass: `stlabel l-${r.status}`,
    harnessClass: `harness h-${r.harness}`,
    reasonClass: `reason r-${r.reasonTone ?? 'mute'}`,
    turnsLabel: turns === 1 ? 'turn' : 'turns',
  }
}

export default controller({
  scope: ({ rows, scope }: ViewScope<Scope>, host) => {
    const all = derived<SessionRow[]>(() => parseRows(rows()))
    const filter = signal<string>('all')

    const visible = derived<EnrichedRow[]>(() =>
      all()
        .filter((r) => filter() === 'all' || r.status === filter())
        .map(enrich),
    )

    const countOf = (status: string): number =>
      status === 'all'
        ? all().length
        : all().filter((r) => r.status === status).length

    // Filter pills — `all` plus the canonical five-status fold. Each carries
    // its live count + a `select` closure and an `active` flag the template binds.
    interface Pill {
      key: string
      label: string
      count: number
      cls: string
      select: () => void
    }
    const pills = derived<Pill[]>(() =>
      ['all', ...STATUSES].map((key) => ({
        key,
        label: key,
        count: countOf(key),
        cls: filter() === key ? 'fpill on' : 'fpill',
        select() {
          filter.set(key)
          host?.dispatchEvent(
            new CustomEvent('ship-filter', {
              detail: { status: key },
              bubbles: true,
              composed: true,
            }),
          )
        },
      })),
    )

    const total = derived<number>(() => all().length)
    const scopeLabel = derived<string>(() => scope())

    return { visible, pills, total, scopeLabel }
  },
})
