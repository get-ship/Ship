import { controller, signal } from '@ship/ui-runtime'
import type { Scope } from './index.sui'
import { loadCatalog, type CatalogEntry } from './partials/catalog'

type StudioFilterKind = 'all' | 'sui' | 'view'

export default controller<Scope>({
  scope: () => ({
    sui: signal<CatalogEntry[]>([]),
    views: signal<CatalogEntry[]>([]),
    loading: signal(true),
    filterKind: signal<StudioFilterKind>('all'),
    filterText: signal(''),
  }),

  derived: {
    allItems: ({ sui, views }) =>
      [...(sui() as CatalogEntry[]), ...(views() as CatalogEntry[])],
    filtered: ({ filterKind, filterText, allItems }) => {
      const kind = filterKind()
      const text = filterText().toLowerCase()
      return (allItems() as CatalogEntry[]).filter(e => {
        const kindMatch = kind === 'all' || e.kind === kind
        const textMatch = !text || e.id.toLowerCase().includes(text)
        return kindMatch && textMatch
      })
    },
    totalCount: ({ allItems }) => (allItems() as CatalogEntry[]).length,
    filteredCount: ({ filtered }) => (filtered() as CatalogEntry[]).length,
  },

  mount: async (state) => {
    const catalog = await loadCatalog()
    state.sui = catalog.sui
    state.views = catalog.views
    state.loading = false
  },
})
