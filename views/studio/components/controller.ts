// Components studio section — the canonical component explorer. Sources the
// catalog from the daemon (published SUI catalog + views) via loadCatalog; the
// custom-elements.json (CEM) feeds the prop metadata. This consolidates the
// prior storybook-style viewers (ship-ui-studio, the dev-panel picker, the docs
// pages) onto one surface; sharing the explorer as a component across the dev
// panel + docs is a follow-up.
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { loadCatalog, type CatalogEntry } from './catalog'

type FilterKind = 'all' | 'sui' | 'view'

export default controller<Scope>({
  scope: () => ({
    sui: signal<CatalogEntry[]>([]),
    views: signal<CatalogEntry[]>([]),
    loading: signal(true),
    filterKind: signal<FilterKind>('all'),
    filterText: signal(''),
  }),

  derived: {
    allItems: ({ sui, views }) =>
      [...(sui() as CatalogEntry[]), ...(views() as CatalogEntry[])],
    filtered: ({ filterKind, filterText, allItems }) => {
      const kind = filterKind() as FilterKind
      const text = (filterText() as string).toLowerCase()
      return (allItems() as CatalogEntry[]).filter((e) => {
        const kindMatch = kind === 'all' || e.kind === kind
        const textMatch =
          !text ||
          e.id.toLowerCase().includes(text) ||
          (e.description ?? '').toLowerCase().includes(text)
        return kindMatch && textMatch
      })
    },
    totalCount: ({ allItems }) => (allItems() as CatalogEntry[]).length,
    filteredCount: ({ filtered }) => (filtered() as CatalogEntry[]).length,
  },

  on: {
    onSearchInput: (s, e: Event) => {
      s.filterText = (e.target as HTMLInputElement).value ?? ''
    },
    clearSearch: (s) => {
      s.filterText = ''
    },
    setKind: (s, id: string) => {
      s.filterKind = id as FilterKind
    },
  },

  mount: async (state) => {
    const catalog = await loadCatalog()
    state.sui = catalog.sui
    state.views = catalog.views
    state.loading = false
  },
})
