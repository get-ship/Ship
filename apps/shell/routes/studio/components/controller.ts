// Components studio index — the canonical component explorer. Sources the
// workspace catalog from the daemon (/_deck/catalog/workspace) via
// loadCatalog; the CEM feeds prop/slot metadata and per-component `examples`.
//
// The index is grouped: a flat package filter (All / Primitives / Charts /
// Pipelines) over a free-text search, rendering one section per semantic
// category (Actions, Inputs, Layout, …). Categories are derived from the tag
// (charts/pipelines packages fold into their own group). Subcomponents
// (sui-accordion-item ⊂ sui-accordion) are folded into the parent's part
// count and hidden from the index — one cell per shippable component.
import { controller, signal, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { loadCatalog, categoryRank, type CatalogEntry } from './catalog'

type Filter = 'all' | 'primitives' | 'charts' | 'pipelines'

const PKG_OF: Record<Exclude<Filter, 'all'>, string> = {
  primitives: '@ship/sui-primitives',
  charts: '@ship/sui-charts',
  pipelines: '@ship/pipelines',
}

interface IndexCell {
  id: string
  tag: string
  description: string
  propCount: number
  parts: number
}

interface IndexGroup {
  id: string
  label: string
  count: number
  cells: IndexCell[]
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const sui = signal<CatalogEntry[]>([])
    const loading = signal(true)
    const filterText = signal('')
    const filter = signal<Filter>('all')

    // Visible (non-subcomponent) entries, filtered by package + text, then
    // grouped into ordered category sections for the index.
    const groups = derived(() => {
      const items = sui() as CatalogEntry[]
      const text = filterText().toLowerCase().trim()
      const f = filter()
      const wantPkg = f === 'all' ? null : PKG_OF[f]
      const tags = items.map((i) => i.tag)
      const childCountOf = (tag: string) =>
        tags.filter((t) => t !== tag && t.startsWith(tag + '-')).length
      const isSubcomponent = (tag: string) =>
        tags.some((t) => t !== tag && tag.startsWith(t + '-'))

      const cells: Array<IndexCell & { category: string }> = items
        .filter((e) => !isSubcomponent(e.tag))
        .filter((e) => (wantPkg ? e.pkg === wantPkg : true))
        .filter((e) => {
          if (!text) return true
          return (
            e.id.toLowerCase().includes(text) ||
            e.category.toLowerCase().includes(text) ||
            (e.description ?? '').toLowerCase().includes(text)
          )
        })
        .map((e) => {
          const parts = childCountOf(e.tag)
          // No CEM description today → describe with real, present data:
          // the prop count (and folded part count) the component carries.
          const propCount = (e.props ?? []).length
          const description =
            e.description ??
            (parts > 0
              ? `${propCount} prop${propCount === 1 ? '' : 's'} · ${parts} part${parts === 1 ? '' : 's'}`
              : `${propCount} prop${propCount === 1 ? '' : 's'}`)
          return { id: e.id, tag: e.tag, description, propCount, parts, category: e.category }
        })

      const byCat = new Map<string, IndexCell[]>()
      for (const c of cells) {
        const arr = byCat.get(c.category) ?? []
        arr.push({ id: c.id, tag: c.tag, description: c.description, propCount: c.propCount, parts: c.parts })
        byCat.set(c.category, arr)
      }
      return [...byCat.entries()]
        .map(([label, list]): IndexGroup => ({
          id: label.toLowerCase(),
          label,
          count: list.length,
          cells: list.sort((a, b) => a.tag.localeCompare(b.tag)),
        }))
        .sort((a, b) => categoryRank(a.label) - categoryRank(b.label))
    })
    const totalCount = derived(() => {
      const items = sui() as CatalogEntry[]
      const tags = items.map((i) => i.tag)
      return items.filter((e) => !tags.some((t) => t !== e.tag && e.tag.startsWith(t + '-'))).length
    })
    // Per-package counts for the filter-pill badges (top-level components only).
    const counts = derived(() => {
      const items = sui() as CatalogEntry[]
      const tags = items.map((i) => i.tag)
      const roots = items.filter((e) => !tags.some((t) => t !== e.tag && e.tag.startsWith(t + '-')))
      const by = (pkg: string) => roots.filter((e) => e.pkg === pkg).length
      return {
        all: roots.length,
        primitives: by('@ship/sui-primitives'),
        charts: by('@ship/sui-charts'),
        pipelines: by('@ship/pipelines'),
      }
    })
    // `groups` is a closure-local derived — readable directly.
    const visibleCount = derived(() => groups().reduce((n, g) => n + g.count, 0))
    return { sui, loading, filterText, filter, groups, totalCount, counts, visibleCount }
  },

  on: {
    onSearchInput: (s, e: Event) => {
      s.filterText = (e.target as HTMLInputElement).value ?? ''
    },
    clearSearch: (s) => {
      s.filterText = ''
    },
    setFilter: (s, id: string) => {
      s.filter = id as Filter
    },
  },

  // mount may be async; the runtime awaits it (MountSection allows Promise<void>).
  mount: async (state) => {
    state.sui = await loadCatalog()
    state.loading = false
  },
})
