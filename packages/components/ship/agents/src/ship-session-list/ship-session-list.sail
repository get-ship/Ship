<controller>
import { controller, derived, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-session-list.sail'

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

export default controller(({ rows, scope }: ViewScope<Scope>, host) => {

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
})
</controller>

<!--
  ship-session-list.sail — @ship/agents · the sessions in a Space (or cross-space
  in shell). One row per session: status dot (folded state) · harness badge · agent ·
  the input · turn count · time · outcome/reason. Covers the full five-status fold —
  active · suspended · completed · failed · cancelled. Rows arrive as data through the
  `rows` prop (JSON); the status-filter header derives its counts from them and emits
  ship-filter on click. No hardcoded fixtures — production passes the live `sessions`
  projection straight through. Tokens only.  Props: rows · scope.
-->
<!-- <prop name="rows" type="string" default="[]" /> -->
<!-- <prop name="scope" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <span>Sessions</span>
    <span data-test="list-count">{total()} sessions</span>
    <span>{scopeLabel()}</span>
  </div>

  <row gap="1">
    <for as="pill" of={pills()} key={pill.key}>
    <button type="button" data-test="filter-pill" on:click={pill.select}>{pill.label}<span>{pill.count}</span></button>
    </for>
  </row>

  <if cond={visible().length === 0}>
  <div data-test="list-empty">No sessions.</div>
  </if>
  <for as="row" of={visible()} key={row.id}>
  <div data-test="session-row">
    <span><span></span><span>{row.status}</span></span>
    <span>{row.harness}</span>
    <column gap="1">
      <span><span>{row.agent}</span><if cond={row.child}><span>{row.child}</span></if></span>
      <span>{row.input}</span>
      <if cond={row.reason}>
      <span>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9"></circle><path d="M12 8v4M12 16h.01"></path></svg>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9"></circle><path d="M15 9l-6 6M9 9l6 6"></path></svg>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9"></circle><path d="M8 12h8"></path></svg>
        {row.reason}
      </span>
      </if>
    </column>
    <column align="end" gap="1"><span>{row.turns} <span>{row.turnsLabel}</span></span><span>{row.model} · {row.time}</span><if cond={row.cost}><span>{row.cost}</span></if></column>
  </div>
  </for>
</div>
