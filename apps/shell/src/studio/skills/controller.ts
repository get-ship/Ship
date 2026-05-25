// Skills studio section — list. Refactored from the old views/skills (which
// handled /skills + /skills/:id in one view via path-watching + SAME_SHELL);
// the studio version splits list and detail into deck-routed views like agents.
// Data: ship.skills.summaries({q,kind,tag}) via liveExpr (server-side filtered,
// re-subscribes as q/kind change) — /sui-hosted views have no SDK client in ctx.
import { controller, signal, derived, live } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
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

type LiveResult = { kind: string; value?: RailValue }

export default controller<Scope>({
  scope: () => {
    const q = signal('')
    const kind = signal<KindFilter>('all')
    const rail = live(() =>
      liveExpr<RailValue>(
        `ship.skills.summaries(${JSON.stringify({ q: q(), kind: kind(), tag: '' })})`,
      ),
    )
    return { q, kind, rail }
  },

  derived: {
    loading: ({ rail }) => (rail() as LiveResult).kind !== 'ok',
    items: ({ rail }) => {
      const r = rail() as LiveResult
      if (r.kind !== 'ok' || !r.value) return []
      return r.value.groups.flatMap((g) => g.skills)
    },
    total: ({ rail }) => {
      const r = rail() as LiveResult
      if (r.kind !== 'ok' || !r.value) return 0
      return r.value.kind_counts.find((k) => k.id === 'all')?.count ?? 0
    },
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
