// Skills studio section — list. Refactored from the old views/skills (which
// handled /skills + /skills/:id in one view via path-watching + SAME_SHELL);
// the studio version splits list and detail into deck-routed views like agents.
// Data: ship.skills.summaries({q,kind,tag}) via liveExpr (server-side filtered,
// re-subscribes as q/kind change) — /sui-hosted views have no SDK client in ctx.
import { controller, signal, live, derived } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type KindFilter = 'all' | 'smart' | 'docs'

interface SkillSummary {
  id: string
  name: string
  description: string
  kind: string
  source: string
  tags: string[]
  files: string[]
  has_vars: boolean
  has_refs: boolean
  has_evals: boolean
}
interface SkillGroup {
  source: string
  skills: SkillSummary[]
}
interface RailValue {
  groups: SkillGroup[]
  kind_counts: Array<{ id: string; label: string; count: number; selected: boolean }>
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const q = signal('')
    const kind = signal<KindFilter>('all')
    const rail = live(() =>
      liveExpr<RailValue>(
        `ship.skills.summaries(${JSON.stringify({ q: q(), kind: kind(), tag: '' })})`,
      ),
    )
    // `rail` is a `LiveSignal<unknown>` — call it directly and narrow the
    // `ok` envelope's `value` (unknown to the type system) to the payload.
    const loading = derived(() => rail().kind !== 'ok')
    const items = derived(() => {
      const r = rail()
      if (r.kind !== 'ok') return []
      return (r.value as RailValue).groups.flatMap((g) => g.skills)
    })
    const total = derived(() => {
      const r = rail()
      if (r.kind !== 'ok') return 0
      return (r.value as RailValue).kind_counts.find((k) => k.id === 'all')?.count ?? 0
    })
    return { q, kind, rail, loading, items, total }
  },

  on: {
    onSearchInput: (s, e: Event) => {
      s.q = (e.target as HTMLInputElement).value ?? ''
    },
    clearSearch: (s) => {
      s.q = ''
    },
    setKind: (s, id: string) => {
      s.kind = id as KindFilter
    },
  },
})
