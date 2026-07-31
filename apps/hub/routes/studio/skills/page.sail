<controller>
// Hub version of `apps/shell/routes/studio/skills/controller.ts` —
// the surface is byte-identical to the shell (the studio's skill-list
// view is one of the surfaces this job ports verbatim) EXCEPT the
// client seam: the shell pulls `ship` from its `../../../ops` wrapper,
// the hub pulls `ops` from the AMBIENT `ctx.ops` injected by
// the host's `entry.gen.ts` (deck-runtime maps `clients.<name>` into
// view scope). The relative `src/lib/client` import is gone — the
// client lives in the view mount context, not a per-view module path.
//
// The runtime also exposes `ctx.<name>` as a legacy alias (the
// `ship`/`ops` keys ride the spread), so pre-migration view code
// still resolves during the cutover. New code reads `ctx.<name>`.
import { controller, signal, live, derived } from '@ship/sail-runtime'
import type { LiveSource, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

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
    // maps that to `ctx.ops` (and the legacy `ctx.ops` alias).
    // Reading the client off `ctx` keeps the view free of any relative
    // `../../../lib/client` import — the host owns the construction site.
    const ops = ctx.ops
    const rail = live(() =>
      ops.skills.summaries({ q: q(), kind: kind(), tag: '' }) as unknown as LiveSource<RailValue>,
    )
    // `loading` is true until the source first resolves — the template
    // (`{#if loading()}`) renders the spinner meanwhile. Returned below + used
    // by the view, but it was never declared (ReferenceError "loading is not
    // defined" blanked the whole skills route). Mirrors the [id] controller's
    // `kind !== 'ok'` pattern.
    const loading = derived(() => rail().kind !== 'ok')
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
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <div>
      <div>

        <header>
          <div>
            <h1>Skills</h1>
            <p>
              <strong>{items().length}</strong> of {total()}
              <span aria-hidden="true"></span>
              capabilities agents can attach
            </p>
            <div aria-hidden="true"></div>
          </div>
        </header>

        <div>
          <div>
            <icon name="search" size="sm" tone="muted"></icon>
            <input type="text" placeholder="Search skills…" aria-label="Search skills"
              spellcheck="false" data-test="search" value={q()} on:input={onSearchInput} />
            <if cond={q()}>
              <button type="button" aria-label="Clear search" on:click={clearSearch}>
                <icon name="x" size="xs"></icon>
              </button>
            </if>
          </div>
          <toggle-group variant="segmented" type="single" value={kind()} size="sm" aria-label="Filter by kind"
            on:sail-change={(e) => setKind(e.detail.value)}>
            <toggle value="all">All</toggle>
            <toggle value="smart">Smart</toggle>
            <toggle value="docs">Docs</toggle>
          </toggle-group>
        </div>

        <if cond={loading()}>
          <div><spinner size="md" centered></spinner></div>
        <else-if cond={items().length === 0}>
          <empty-state icon="search-x" variant="default" size="sm" data-test="empty">
            <span slot="title">No matching skills</span>
            Nothing matches your search.
            <button slot="action" variant="outline" size="sm" on:click={clearSearch}>Clear search</button>
          </empty-state>
        </else-if><else>
          <div data-test="skills-list">
            <for as="s" of={items()} key={s.id}>
              <a data-test="skill-card"
                data-id={s.id} href={'/studio/skills/' + s.id}>
                <span data-kind={s.kind} aria-hidden="true">
                  <icon name={s.kind === 'docs' ? 'file-text' : 'sparkles'} size="sm"></icon>
                </span>
                <div>
                  <div>
                    <span>{s.name}</span>
                    <badge tone="muted" variant="outline" size="sm">{s.kind}</badge>
                    <span>{s.source}</span>
                  </div>
                  <p>{s.description}</p>
                  <div>
                    <span>
                      <icon name="file" size="xs" tone="muted"></icon>
                      {s.files.length} file{s.files.length === 1 ? '' : 's'}
                    </span>
                    <if cond={s.has_vars}>
                      <span><icon name="braces" size="xs" tone="muted"></icon> vars</span>
                    </if>
                    <for as="t" of={s.tags} key={t}>
                      <chip tone="muted" variant="soft" size="sm">{t}</chip>
                    </for>
                  </div>
                </div>
                <icon name="chevron-right" size="sm" tone="muted"></icon>
              </a>
            </for>
          </div>
        </else></if>

      </div>
    </div>
  </main>
</div>
