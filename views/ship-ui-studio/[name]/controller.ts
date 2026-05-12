import { controller, signal, effect } from '@ship/sui-runtime'
import { defineRoutes } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'
import { loadCatalog, type CatalogEntry, type PropEntry } from '../catalog'

type StudioFilterKind = 'all' | 'sui' | 'view'

// Until shell 2.0 ports the studio in, the canonical access path is
// /sui/ship-ui-studio/:name (view-host static mount). Both patterns
// are registered so route.params.name resolves under either prefix.
// When shell 2.0 lands, the /sui/ prefix entry can drop.
defineRoutes([
  { pattern: '/ship-ui-studio/:name' },
  { pattern: '/sui/ship-ui-studio/:name' },
])

// Shared across mounts. Each rapid catalog click previously created a
// new iframe — the prior one's dispose ran async after re-mount, so
// in-flight iframes (+ their SSE/reactive scopes) compounded until
// the tab froze. Holding one frame at module scope and only setting
// .src on it keeps the SSE connection alive across navigations and
// caps DOM iframe count at 1. Same idea for the sui-kind preview
// node: reuse when the tag matches, recreate only on tag change.
let sharedFrame: HTMLIFrameElement | null = null
let sharedNode: HTMLElement | null = null
let sharedNodeTag: string | null = null

// Default-slot fillers for compound components — atomic ones render
// fine with just an "example" text node, but compounds need real
// child elements to show their layout (sui-rail empty looks like a
// blank rectangle). Keyed by host tag.
const SLOT_COMPOSITIONS: Record<string, string> = {
  'sui-rail': `
    <sui-rail-section label="Section A">
      <sui-rail-item data-view="counter">Counter</sui-rail-item>
      <sui-rail-item data-view="agents" active>Agents</sui-rail-item>
    </sui-rail-section>
    <sui-rail-section label="Section B">
      <sui-rail-item data-view="git">Git</sui-rail-item>
    </sui-rail-section>
  `,
  'sui-tabs': `
    <sui-tab name="overview" active>Overview</sui-tab>
    <sui-tab name="details">Details</sui-tab>
    <sui-tab name="logs">Logs</sui-tab>
  `,
  'sui-dialog': `
    <span slot="header">Confirm action</span>
    <p>Are you sure you want to continue? This affects 3 items.</p>
    <span slot="footer">
      <sui-button variant="outline">Cancel</sui-button>
      <sui-button variant="primary">Continue</sui-button>
    </span>
  `,
  'sui-alert-dialog': `
    <span slot="title">Delete workspace?</span>
    <p>This permanently removes the workspace and its history.</p>
  `,
  'sui-card': `
    <span slot="header">Card title</span>
    <p>Card body content. Use cards to group related information.</p>
    <span slot="footer">
      <sui-button variant="outline" size="sm">Action</sui-button>
    </span>
  `,
  'sui-accordion': `
    <sui-accordion-item value="one">
      <span slot="trigger">Section one</span>
      <p>First section body.</p>
    </sui-accordion-item>
    <sui-accordion-item value="two">
      <span slot="trigger">Section two</span>
      <p>Second section body.</p>
    </sui-accordion-item>
  `,
  'sui-popover': `
    <sui-button slot="trigger">Open popover</sui-button>
    <p>Popover panel content.</p>
  `,
  'sui-dropdown-menu': `
    <sui-button slot="trigger">Open menu</sui-button>
    <button role="menuitem" data-value="rename">Rename</button>
    <button role="menuitem" data-value="duplicate">Duplicate</button>
    <button role="menuitem" data-value="delete">Delete</button>
  `,
}

export default controller<Scope>({
  scope: () => ({
    allCatalog: signal<CatalogEntry[]>([]),
    loading: signal(true),
    propValues: signal<Record<string, unknown>>({}),
    _dispose: signal<(() => void) | null>(null),
    filterKind: signal<StudioFilterKind>('all'),
    filterText: signal(''),
  }),

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
      if (sharedNode?.parentElement) sharedNode.parentElement.removeChild(sharedNode)
      if (!sharedFrame) {
        sharedFrame = document.createElement('iframe')
        sharedFrame.className = 'detail-view-frame'
        sharedFrame.loading = 'eager'
      }
      if (sharedFrame.parentElement !== slot) {
        sharedFrame.parentElement?.removeChild(sharedFrame)
        slot.appendChild(sharedFrame)
      }
      sharedFrame.title = `${entry.id} preview`
      const nextHref = new URL(`/sui/${entry.id}`, location.href).href
      if (sharedFrame.src !== nextHref) sharedFrame.src = `/sui/${entry.id}`
      // No teardown — the iframe (and its SSE connection) lives at
      // module scope so the next view-kind mount reuses it. When the
      // studio itself unmounts the browser drops the page; nothing
      // to clean up here.
      state._dispose = () => {}
      return
    }

    // Detach iframe if we're coming from a view-kind preview.
    if (sharedFrame?.parentElement) sharedFrame.parentElement.removeChild(sharedFrame)

    // Reuse one host node per tag across mounts. Tag change ⇒ swap;
    // otherwise the effect() below mutates attrs on the same element.
    if (sharedNode && sharedNodeTag !== entry.tag) {
      sharedNode.remove()
      sharedNode = null
      sharedNodeTag = null
    }
    if (!sharedNode) {
      sharedNode = document.createElement(entry.tag)
      sharedNode.className = 'detail-rendered-component'
      sharedNodeTag = entry.tag
    }
    const node = sharedNode
    // Reset transform (cleared in case the prior mount applied ZOOM)
    node.style.transform = ''
    node.style.transformOrigin = ''
    // Seed the default slot. Compound components get a typical-use
    // composition parsed into a DocumentFragment; the parser upgrades
    // any <sui-*> elements as nodes are adopted into the live document
    // on appendChild. Atomic components get a plain "example" text
    // node BEFORE the host observes slot occupancy (avoids icon-only
    // collapse on tags like sui-button that reflect-empty into
    // data-icon-only). Using a fragment instead of .innerHTML keeps
    // controller-purity happy and preserves the host's own attributes.
    const compose = SLOT_COMPOSITIONS[entry.tag]
    if (compose) {
      const range = document.createRange()
      range.selectNodeContents(node)
      const frag = range.createContextualFragment(compose)
      node.replaceChildren(frag)
    } else {
      node.replaceChildren(document.createTextNode('example'))
    }
    if (node.parentElement !== slot) {
      node.parentElement?.removeChild(node)
      slot.appendChild(node)
    }

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

    // Leave the node attached — next mount reuses it (same tag) or
    // swaps it (different tag). Just stop the prior reactive binding
    // so attribute writes don't leak across components.
    state._dispose = () => { stopEffect() }
  },

  unmount: (state) => {
    const fn = state._dispose as (() => void) | undefined
    fn?.()
  },
})
