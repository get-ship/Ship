// Hub version of `apps/shell/routes/studio/skills/controller.ts` —
// the surface is byte-identical to the shell (the studio's skill-list
// view is one of the surfaces this job ports verbatim) EXCEPT the
// client seam: the shell pulls `ship` from its `../../../ops` wrapper,
// the hub pulls `ops` from the AMBIENT `ctx.clients.ops` injected by
// the host's `entry.gen.ts` (deck-runtime maps `clients.<name>` into
// view scope). The relative `src/lib/client` import is gone — the
// client lives in the view mount context, not a per-view module path.
//
// The runtime also exposes `ctx.<name>` as a legacy alias (the
// `ship`/`ops` keys ride the spread), so pre-migration view code
// still resolves during the cutover. New code reads `ctx.clients.<name>`.
import { controller, signal, live, derived } from '@ship/sui-runtime'
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
  scope: (ctx: ViewScope<Scope>) => {
    const q = signal('')
    const kind = signal<KindFilter>('all')
    // AMBIENT client: the host's `entry.gen.ts` constructs `ops` from
    // `src/lib/client.ts` and injects it as `clients.ops`. The runtime
    // maps that to `ctx.clients.ops` (and the legacy `ctx.ops` alias).
    // Reading the client off `ctx` keeps the view free of any relative
    // `../../../lib/client` import — the host owns the construction site.
    const ops = ctx.clients.ops
    const rail = live(() =>
      ops.skills.summaries({ q: q(), kind: kind(), tag: '' }) as unknown as LiveSource<RailValue>,
    )
    // Defensive: the hub's `ship.skills.summaries` returns a different /
    // empty shape than the shell's. A missing `groups` or `kind_counts`
    // must NOT throw — the derived re-runs on every source tick, so an
    // unguarded `.flatMap` / `.find` on `undefined` triggers the runtime's
    // crash → re-subscribe loop and a CPU storm. Mirror the agents
    // controller's `v?.groups ?? []` pattern: degrade to empty arrays
    // and let the page's own loading/empty state render.
    // skills.summaries is a UNARY Connect op (no live mux resolver — see
    // apps/shipd/.../live_resolvers.rs: only agents.summaries / collections.list
    // / git.overview / account.state are live). It wraps its payload in a generic
    // `{ json }` envelope. Decode it (plain fn, not an extra signal node),
    // tolerating both the flat `{ items }` the daemon returns today and the
    // grouped `{ groups, kind_counts }` shape. A missing field degrades to empty.
    const payload = (): Partial<RailValue> & { items?: SkillSummary[] } => {
      const r = rail()
      if (r.kind !== 'ok') return {}
      let v = r.value as { json?: unknown } | Record<string, unknown> | null | undefined
      if (v && typeof (v as { json?: unknown }).json === 'string') {
        try { v = JSON.parse((v as { json: string }).json) as Record<string, unknown> } catch { v = {} }
      }
      return (v ?? {}) as Partial<RailValue> & { items?: SkillSummary[] }
    }
    const items = derived(() => {
      const v = payload()
      if (Array.isArray(v.items)) return v.items
      const groups = Array.isArray(v.groups) ? v.groups : []
      return groups.flatMap((g) => Array.isArray(g?.skills) ? g.skills : [])
    })
    const total = derived(() => {
      const v = payload()
      const counts = Array.isArray(v.kind_counts) ? v.kind_counts : []
      return counts.find((k) => k?.id === 'all')?.count ?? items().length
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
