// <ship-agent-list> — @ship/agents · the hub `/studio/agents` index.
//
// Page-level list composite. Owns the data wiring for the agents list,
// the search/filter state, and renders a grid of <ship-agent-card>-style
// tiles. The card is the per-agent summary cell; the list owns the
// aggregate story (header, search, source filter, empty / loading /
// error states).
//
// Subscribes to ship.agents.summaries() via live() inside the
// controller; no SDK call in the consumer (the hub route is a thin
// composition). The summaries payload is `unknown` from the SDK; the
// controller narrows defensively to the fields the surface renders —
// the same pattern as the shell's studio/agents controller and the
// home pack's <ship-activity-runs>.
//
// Filter model: free-text search across name / description / id /
// tags, plus a source-kind segmented control (all / project / user).
// `kind === 'project'` matches source === 'PROJECT AGENTS';
// `kind === 'user'` matches 'USER AGENTS'. Anything else is "all".
// The matcher is exported so the unit test exercises it directly.
//
// The shell's matching helpers (`matchesKind`, `matchesQuery`) are
// inlined into the controller body so the list composite has no
// sibling-route dependency — the hub re-derives the same shape
// from the same source data (the per-pack mixin rule).

import { controller, derived, live, signal, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-agent-list.sui'

type Kind = 'all' | 'project' | 'user'

export interface AgentRow {
  id: string
  name: string
  description: string
  source: string
  tags: string[]
  active: boolean
  skills_count: number
  mcp_count: number
  rules_count: number
  letter: string
  hue: string
}

const HUES = [
  'orange', 'amber', 'red', 'violet', 'blue',
  'teal', 'green', 'pink', 'slate',
] as const

function pickHue(id: string): string {
  if (!id) return 'slate'
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return HUES[Math.abs(h) % HUES.length]
}

function pickLetter(id: string, name: string): string {
  const src = (id || name || '').trim()
  if (!src) return '?'
  const parts = src.split(/[-_\s]+/).filter(Boolean)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  const head = parts[0] || src
  return (head.length >= 2 ? head.slice(0, 2) : head).toUpperCase()
}

interface Summaries {
  items?: Array<{
    id?: unknown
    name?: unknown
    description?: unknown
    source?: unknown
    tags?: unknown
    active?: unknown
    skills_count?: unknown
    mcp_count?: unknown
    rules_count?: unknown
  }>
}

const EMPTY: Summaries = { items: [] }

export function readRows(value: unknown): AgentRow[] {
  const v = (value ?? {}) as Partial<Summaries>
  const items = Array.isArray(v.items) ? v.items : []
  const out: AgentRow[] = []
  for (const a of items) {
    if (!a || typeof a.id !== 'string') continue
    const id = a.id
    const name = typeof a.name === 'string' && a.name ? a.name : id
    const description = typeof a.description === 'string' ? a.description : ''
    const source = typeof a.source === 'string' ? a.source : ''
    const tags = Array.isArray(a.tags) ? a.tags.filter((t) => typeof t === 'string') as string[] : []
    const active = a.active === true
    const skills = Number.isFinite(a.skills_count) ? Number(a.skills_count) : 0
    const mcp = Number.isFinite(a.mcp_count) ? Number(a.mcp_count) : 0
    const rules = Number.isFinite(a.rules_count) ? Number(a.rules_count) : 0
    out.push({
      id, name, description, source, tags, active,
      skills_count: skills, mcp_count: mcp, rules_count: rules,
      letter: pickLetter(id, name),
      hue: pickHue(id),
    })
  }
  return out
}

function matchesKind(a: AgentRow, kind: Kind): boolean {
  if (kind === 'all') return true
  if (kind === 'project') return a.source === 'PROJECT AGENTS'
  if (kind === 'user') return a.source === 'USER AGENTS'
  return true
}

function matchesQuery(a: AgentRow, q: string): boolean {
  if (!q) return true
  const needle = q.toLowerCase().trim()
  if (!needle) return true
  if (a.name.toLowerCase().includes(needle)) return true
  if (a.id.toLowerCase().includes(needle)) return true
  if (a.description.toLowerCase().includes(needle)) return true
  for (const t of a.tags) if (t.toLowerCase().includes(needle)) return true
  return false
}

export default controller({
  scope: ({ limit }: ViewScope<Scope>, host?: Element) => {
    const summaries = live(() => ship.agents.summaries())

    const q = signal('')
    const kind = signal<Kind>('all')

    const allRows = derived<AgentRow[]>(() => readRows(unwrapOr(summaries, EMPTY)))
    const rows = derived<AgentRow[]>(() => {
      const all = allRows()
      const term = q().trim()
      const k = kind()
      const n = Number.isFinite(limit()) ? Math.max(0, Math.floor(limit())) : 24
      return all
        .filter((a) => matchesKind(a, k) && matchesQuery(a, term))
        .slice(0, n)
    })
    const total = derived<number>(() => allRows().length)
    const hasResults = derived<boolean>(() => rows().length > 0)
    const isLoaded = derived<boolean>(() => summaries().kind === 'ok')

    const sub = derived<string>(() => {
      if (summaries().kind === 'loading') return 'loading agents…'
      const t = total()
      if (t === 0) return 'no agents yet'
      if (t === 1) return '1 agent'
      return `${t} agents`
    })

    const kindCounts = derived<{ all: number; project: number; user: number }>(() => {
      const all = allRows()
      return {
        all: all.length,
        project: all.filter((a) => a.source === 'PROJECT AGENTS').length,
        user: all.filter((a) => a.source === 'USER AGENTS').length,
      }
    })

    const onOpen = (e: MouseEvent, id: string): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-agent-open', {
          detail: { id },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onSearchInput = (e: Event): void => {
      const target = e.target as HTMLInputElement | null
      q.set(target?.value ?? '')
    }
    const clearSearch = (): void => q.set('')
    const resetFilters = (): void => {
      q.set('')
      kind.set('all')
    }
    const setKind = (next: string): void => {
      if (next === 'all' || next === 'project' || next === 'user') kind.set(next)
    }

    return {
      summaries, q, kind,
      allRows, rows, total, hasResults, isLoaded,
      sub, kindCounts,
      onOpen, onSearchInput, clearSearch, resetFilters, setKind,
    }
  },
})

// Re-export the helpers so a unit test can exercise the matchers + the
// shape-narrower without going through the controller scope. The
// exports are pure (no runtime imports) — they keep the test surface
// in one place.
export const __test__ = { pickHue, pickLetter, readRows, matchesKind, matchesQuery, HUES }
