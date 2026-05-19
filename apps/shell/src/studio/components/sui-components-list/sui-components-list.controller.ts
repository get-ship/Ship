import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './sui-components-list.sui'
import {
  loadComponents,
  groupByCategory,
  type ComponentEntry,
  type CategoryGroup,
} from '../catalog'

type PkgFilter = 'all' | string

export interface Pill { id: PkgFilter; label: string; count: number }

// Project an npm package name to the short pill label, e.g.
// `@ship/primitives` → `primitives`. Unscoped names pass through.
function pkgLabel(pkg: string | undefined): string {
  if (!pkg) return 'other'
  const slash = pkg.lastIndexOf('/')
  return slash >= 0 ? pkg.slice(slash + 1) : pkg
}

export default controller<Scope>({
  scope: () => ({
    entries: signal<ComponentEntry[]>([]),
    loading: signal(true),
    query: signal(''),
    pkg: signal<PkgFilter>('all'),
  }),

  derived: {
    totalCount: ({ entries }) => (entries() as ComponentEntry[]).length,

    // Package pills: All + one per distinct owning package, with counts.
    pills: ({ entries }): Pill[] => {
      const all = entries() as ComponentEntry[]
      const counts = new Map<string, number>()
      for (const e of all) {
        const id = pkgLabel(e.package)
        counts.set(id, (counts.get(id) ?? 0) + 1)
      }
      const out: Pill[] = [{ id: 'all', label: 'All', count: all.length }]
      for (const [id, count] of [...counts].sort((a, b) => b[1] - a[1])) {
        out.push({ id, label: id, count })
      }
      return out
    },

    groups: ({ entries, query, pkg }): CategoryGroup[] => {
      const q = query().trim().toLowerCase()
      const filter = pkg()
      const filtered = (entries() as ComponentEntry[]).filter((e) => {
        if (filter !== 'all' && pkgLabel(e.package) !== filter) return false
        if (!q) return true
        return (
          e.tag.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q)
        )
      })
      return groupByCategory(filtered)
    },
  },

  on: {
    open: (_s, tag: string) => {
      chrome.navigate(`/studio/components/${tag}`)
    },
    setPkg: (state, id: PkgFilter) => {
      state.pkg = id
    },
    onSearch: (state, e: Event) => {
      state.query = (e.target as HTMLInputElement).value
    },
  },

  // Non-async so the return type stays `void` (MountSection rejects a
  // returned Promise). Fire the catalog fetch and assign on resolve.
  mount: (state) => {
    void loadComponents().then((entries) => {
      state.entries = entries
      state.loading = false
    })
  },
})
