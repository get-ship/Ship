import { controller, effect, defineRoutes } from '@ship/ui-runtime'
import type { Scope } from './index.sui'
import { loadCatalog, type CatalogEntry, type PropEntry } from '../partials/catalog'

// View can be hosted at `/ship-ui-studio/:name` (shell SPA) or
// `/sui/ship-ui-studio/:name` (standalone daemon URL). Register both so
// `route.params.name` resolves regardless of mount context.
defineRoutes([
  { pattern: '/ship-ui-studio/:name' },
  { pattern: '/sui/ship-ui-studio/:name' },
])

export default controller<Scope>({
  derived: {
    entry: ({ allCatalog, route }) => {
      const name = route.params.name()
      if (!name) return null
      return (allCatalog() as CatalogEntry[]).find(e => e.id === name) ?? null
    },
    hasProps: ({ entry }) => {
      const e = (entry() as CatalogEntry | null)
      return e !== null && (e.props ?? []).length > 0
    },
  },

  on: {
    setProp: (state, propName: string, value: unknown) => {
      state.propValues = {
        ...(state.propValues as Record<string, unknown>),
        [propName]: value,
      }
    },
  },

  mount: async (state, el) => {
    const catalog = await loadCatalog()
    const all = [...catalog.lit, ...catalog.sui, ...catalog.views]
    state.allCatalog = all

    // Derive name from pathname since route signals are seeded from
    // defineRoutes above, but mount runs before the first reactive tick.
    const segments = location.pathname.split('/').filter(Boolean)
    const name = segments[segments.length - 1] ?? ''
    const entry = all.find(e => e.id === name)

    if (!entry) {
      state.loading = false
      return
    }

    // Seed prop defaults
    const initVals: Record<string, unknown> = {}
    for (const prop of (entry.props ?? []) as PropEntry[]) {
      if (prop.default !== undefined) initVals[prop.name] = prop.default
    }
    state.propValues = initVals
    state.loading = false

    // Wait one tick for the template to re-render with loading=false
    await new Promise<void>(r => setTimeout(r, 0))

    const slot = el.querySelector('[data-render-slot]') as HTMLElement | null
    if (!slot) return

    if (entry.kind === 'view') {
      slot.innerHTML = `<div class="detail-view-hint"><p>SUI view preview: <code>${entry.id}</code></p></div>`
      const clear = () => { slot.innerHTML = '' }
      state._dispose = clear
      return
    }

    const node = document.createElement(entry.tag)
    node.className = 'detail-rendered-component'
    slot.appendChild(node)

    // Apply initial prop values
    const applyProps = (vals: Record<string, unknown>) => {
      for (const [k, v] of Object.entries(vals)) {
        if (v !== undefined && v !== '') node.setAttribute(k, String(v))
      }
    }
    applyProps(initVals)

    // Reactively sync propValues → element attributes
    const stopEffect = effect(() => {
      const vals = state.propValues as Record<string, unknown>
      applyProps(vals)
    })

    state._dispose = () => { stopEffect(); node.remove() }
  },

  unmount: (state) => {
    const fn = state._dispose as (() => void) | undefined
    fn?.()
  },
})
