import { controller } from '@ship/ui-runtime'
import type { Scope } from './index.sui'
import { loadCatalog, type CatalogEntry } from './partials/catalog'

export default controller<Scope>({
  derived: {
    allItems: ({ lit, sui, views }) =>
      [...(lit() as CatalogEntry[]), ...(sui() as CatalogEntry[]), ...(views() as CatalogEntry[])],
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
  },

  mount: async (state) => {
    const catalog = await loadCatalog()
    state.lit = catalog.lit
    state.sui = catalog.sui
    state.views = catalog.views
    state.loading = false
  },
})
