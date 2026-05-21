// Components studio section — the canonical component explorer. Sources the
// catalog from the daemon (published SUI catalog + views) via loadCatalog; the
// custom-elements.json (CEM) feeds the prop metadata. This consolidates the
// prior storybook-style viewers (ship-ui-studio, the dev-panel picker, the docs
// pages) onto one surface; sharing the explorer as a component across the dev
// panel + docs is a follow-up.
//
// User vs Dev mode: users should see published *packs*, not every element. A
// component made of subcomponents (sui-accordion + sui-accordion-item) ships as
// one pack. Until the catalog carries real pack metadata (a bundler-maturation
// follow-up), we fold deterministically: an entry is a subcomponent when
// another entry's tag is a prefix of it (sui-accordion-item ⊂ sui-accordion).
// User mode shows pack roots only; Dev mode shows everything. Persisted so the
// choice sticks across reloads.
import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { loadCatalog, type CatalogEntry } from './catalog'

type FilterKind = 'all' | 'sui' | 'view'
type Mode = 'user' | 'dev'

const MODE_KEY = 'studio-components-mode'
const readMode = (): Mode => {
  if (typeof localStorage === 'undefined') return 'user'
  return localStorage.getItem(MODE_KEY) === 'dev' ? 'dev' : 'user'
}

export default controller<Scope>({
  scope: () => ({
    sui: signal<CatalogEntry[]>([]),
    views: signal<CatalogEntry[]>([]),
    loading: signal(true),
    filterKind: signal<FilterKind>('all'),
    filterText: signal(''),
    mode: signal<Mode>(readMode()),
  }),

  derived: {
    allItems: ({ sui, views }) =>
      [...(sui() as CatalogEntry[]), ...(views() as CatalogEntry[])],
    filtered: ({ filterKind, filterText, allItems, mode }) => {
      const items = allItems() as CatalogEntry[]
      const kind = filterKind() as FilterKind
      const text = (filterText() as string).toLowerCase()
      const m = mode() as Mode
      const tags = items.map((i) => i.tag)
      const childCountOf = (tag: string) =>
        tags.filter((t) => t !== tag && t.startsWith(tag + '-')).length
      const isSubcomponent = (tag: string) =>
        tags.some((t) => t !== tag && tag.startsWith(t + '-'))
      return items
        .filter((e) => {
          const modeMatch = m === 'dev' || !isSubcomponent(e.tag)
          const kindMatch = kind === 'all' || e.kind === kind
          const textMatch =
            !text ||
            e.id.toLowerCase().includes(text) ||
            (e.description ?? '').toLowerCase().includes(text)
          return modeMatch && kindMatch && textMatch
        })
        .map((e) => ({ ...e, parts: childCountOf(e.tag) }))
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
    setMode: (s, id: string) => {
      s.mode = id as Mode
      if (typeof localStorage !== 'undefined') localStorage.setItem(MODE_KEY, id)
    },
  },

  mount: async (state) => {
    const catalog = await loadCatalog()
    state.sui = catalog.sui
    state.views = catalog.views
    state.loading = false
  },
})
