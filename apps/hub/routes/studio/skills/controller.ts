// Hub version of `apps/shell/routes/studio/skills/controller.ts` —
// the surface is byte-identical to the shell (the studio's skill-list
// view is one of the surfaces this job ports verbatim) EXCEPT the
// `ship` import seam: the shell pulls `ship` from its `../../../ops`
// wrapper, the hub has no `ops.ts` so it pulls the same
// `createShipConnectClient` singleton from `src/lib/client.ts` (the
// hub's per-app Connect client). All other code paths are unchanged.
import { controller, signal, live, derived } from '@ship/sui-runtime'
import { ship } from '../../../src/lib/client'
import type { LiveSource, ViewScope } from '@ship/sui-runtime'
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
      ship.skills.summaries({ q: q(), kind: kind(), tag: '' }) as LiveSource<RailValue>,
    )
    // `rail` is a `LiveSignal<unknown>` — call it directly and narrow the
    // `ok` envelope's `value` (unknown to the type system) to the payload.
    const loading = derived(() => rail().kind !== 'ok')
    // Defensive: the hub's `ship.skills.summaries` returns a different /
    // empty shape than the shell's. A missing `groups` or `kind_counts`
    // must NOT throw — the derived re-runs on every source tick, so an
    // unguarded `.flatMap` / `.find` on `undefined` triggers the runtime's
    // crash → re-subscribe loop and a CPU storm. Mirror the agents
    // controller's `v?.groups ?? []` pattern: degrade to empty arrays
    // and let the page's own loading/empty state render.
    const items = derived(() => {
      const r = rail()
      if (r.kind !== 'ok') return []
      const v = r.value as Partial<RailValue> | null | undefined
      const groups = Array.isArray(v?.groups) ? v!.groups : []
      return groups.flatMap((g) => Array.isArray(g?.skills) ? g.skills : [])
    })
    const total = derived(() => {
      const r = rail()
      if (r.kind !== 'ok') return 0
      const v = r.value as Partial<RailValue> | null | undefined
      const counts = Array.isArray(v?.kind_counts) ? v!.kind_counts : []
      return counts.find((k) => k?.id === 'all')?.count ?? 0
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
