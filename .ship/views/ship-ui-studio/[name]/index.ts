import { controller, effect, defineRoutes } from '@ship/ui-runtime'
import type { Scope } from './index.sui'
import { loadCatalog, type CatalogEntry, type PropEntry } from '../partials/catalog'

// Until shell 2.0 ports the studio in, the canonical access path is
// /sui/ship-ui-studio/:name (view-host static mount). Both patterns
// are registered so route.params.name resolves under either prefix.
// When shell 2.0 lands, the /sui/ prefix entry can drop.
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
    currentName: ({ route }) => route.params.name() ?? '',
    filteredCatalog: ({ allCatalog, filterKind, filterText }) => {
      const kind = filterKind()
      const text = filterText().toLowerCase()
      return (allCatalog() as CatalogEntry[]).filter(e => {
        const kindMatch = kind === 'all' || e.kind === kind
        const textMatch = !text || e.id.toLowerCase().includes(text)
        return kindMatch && textMatch
      })
    },
    totalCount: ({ allCatalog }) => (allCatalog() as CatalogEntry[]).length,
    filteredCount: ({ filteredCatalog }) => (filteredCatalog() as CatalogEntry[]).length,
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

    // Seed prop defaults. For size props, prefer the largest enum variant
    // so small components (badge, kbd, status-dot) are legible in the preview.
    const initVals: Record<string, unknown> = {}
    for (const prop of (entry.props ?? []) as PropEntry[]) {
      if (prop.name === 'size' && prop.values && prop.values.length > 0) {
        initVals[prop.name] = prop.values[prop.values.length - 1]
      } else if (prop.default !== undefined) {
        initVals[prop.name] = prop.default
      }
    }
    state.propValues = initVals
    state.loading = false

    // Wait one tick for the template to re-render with loading=false
    await new Promise<void>(r => setTimeout(r, 0))

    const slot = el.querySelector('[data-render-slot]') as HTMLElement | null
    if (!slot) return

    if (entry.kind === 'view') {
      // Render the actual view via the daemon's standalone /sui/<id>
      // route, sandboxed in an iframe so its reactive scope stays
      // isolated from the studio. Same-origin so authentication
      // cookies flow; ship-ui-studio inside ship-ui-studio works
      // because each iframe is its own document.
      const frame = document.createElement('iframe')
      frame.className = 'detail-view-frame'
      frame.src = `/sui/${entry.id}`
      frame.title = `${entry.id} preview`
      frame.loading = 'eager'
      slot.appendChild(frame)
      state._dispose = () => { frame.remove() }
      return
    }

    const node = document.createElement(entry.tag)
    node.className = 'detail-rendered-component'
    // Seed the default slot with a real text node BEFORE the host
    // observes its slot occupancy. This avoids icon-only collapse on
    // tags like sui-button that reflect-empty into `data-icon-only`.
    node.appendChild(document.createTextNode('example'))
    slot.appendChild(node)

    // Components with intrinsic size < 64px are scaled up for legibility.
    const ZOOM_TAGS = new Set(['sui-status-dot', 'sui-kbd', 'sui-badge', 'sui-chip', 'sui-spinner'])
    if (ZOOM_TAGS.has(entry.tag)) {
      node.style.transform = 'scale(3)'
      node.style.transformOrigin = 'center'
    }

    // <select> ignores its `value` attribute — the matching <option>
    // must be marked `selected`, or the select.value property must be
    // assigned post-render. The studio's prop panel uses native
    // <select> so author markup stays simple; sync the JS-property
    // here once the DOM exists.
    const syncSelects = () => {
      const root = el as ParentNode
      for (const sel of root.querySelectorAll<HTMLSelectElement>('.prop-select')) {
        const propName = sel.id.replace(/^prop-/, '')
        const v = (state.propValues as Record<string, unknown>)[propName]
        if (v != null) sel.value = String(v)
      }
    }
    syncSelects()

    // Apply prop values to the rendered host. Boolean attributes follow
    // HTML semantics: presence ≡ true, regardless of value — so
    // `disabled="false"` would still mark the button disabled. Strip
    // boolean attrs entirely when the value is falsy. For string props,
    // setAttribute(k, '') deliberately keeps the attribute (some
    // components reflect emptiness). For unset / `undefined` props, the
    // attribute is removed so the component falls back to its declared
    // default.
    const props = (entry.props ?? []) as PropEntry[]
    const propTypes = new Map(props.map(p => [p.name, p.type]))
    const applyProps = (vals: Record<string, unknown>) => {
      for (const p of props) {
        const v = vals[p.name]
        const t = propTypes.get(p.name)
        const isBool = t === 'boolean' || t === 'Boolean' || t === 'bool'
        if (isBool) {
          const truthy = v === true || v === 'true'
          if (truthy) node.setAttribute(p.name, '')
          else node.removeAttribute(p.name)
        } else {
          if (v == null || v === '') node.removeAttribute(p.name)
          else node.setAttribute(p.name, String(v))
        }
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
