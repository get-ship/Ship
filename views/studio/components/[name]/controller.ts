import { controller, signal, effect } from '@ship/sui-runtime'
import { defineRoutes } from '@ship/sui-runtime/route'
import type { Scope } from './page.sui'
import { loadCatalog, type CatalogEntry } from '../catalog'
import {
  buildCells,
  deriveAxes,
  mountMatrixCells,
  type MatrixAxis,
} from './matrix'
import { SLOT_COMPOSITIONS, slotLabel } from './slot-compositions'
import { buildSnippet } from './snippet'
import { readSearchParams, writeSearchParams, highlightActiveCell } from './deep-link'

type StudioFilterKind = 'all' | 'sui' | 'view'

// Canonical access path is /studio/components/:name (the shell route) and
// /sui/studio/components/:name (view-host standalone, e.g. dev-panel popouts).
defineRoutes([
  { pattern: '/studio/components/:name' },
  { pattern: '/sui/studio/components/:name' },
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

// Components shipped as their own separate bundle at
// `/_sui/sui-components/<tag>.js`. The daemon normally injects these into the
// host page by static-scanning view templates for the tag, but the
// studio creates host nodes via `document.createElement(tag)` at
// runtime — the daemon can't see those, so the bundle module never
// runs and the host never upgrades. Eagerly fetch the bundle when the
// studio is about to render one of these tags. The other separate-bundle
// tags (panel-dock, nav-top, command-bar, rail, terminal, launcher,
// devtools) live on STUDIO_HIDE_TAGS in catalog.ts so they don't
// reach this code path.
const ISLAND_TAGS = new Set<string>([
  'sui-code-editor',
  'sui-markdown-editor',
])
const islandsLoaded = new Set<string>()
function ensureIsland(tag: string): void {
  if (!ISLAND_TAGS.has(tag) || islandsLoaded.has(tag)) return
  if (customElements.get(tag)) {
    islandsLoaded.add(tag)
    return
  }
  islandsLoaded.add(tag)
  // Fire-and-forget. The host node is already in the DOM; once the
  // module evaluates it calls customElements.define and the existing
  // element upgrades in place — no need to re-create the host.
  import(/* @vite-ignore */ `/_sui/sui-components/${tag}.js`).catch(() => {
    // Leave the host un-upgraded on failure so the studio visibly
    // fails for the offending tag rather than silently pretending the
    // component is present.
    islandsLoaded.delete(tag)
  })
}

export default controller<Scope>({
  scope: () => ({
    allCatalog: signal<CatalogEntry[]>([]),
    loading: signal(true),
    propValues: signal<Record<string, unknown>>({}),
    _dispose: signal<(() => void) | null>(null),
    copyFeedback: signal(false),
    // Authoritative subroute slug — seeded from `location.pathname` at
    // mount() rather than `route.params.name()`. The shell pre-registers
    // the dynamic pattern as `/ship-ui-studio/:component` (see
    // apps/shell/src/main.ts), so the view's own `defineRoutes` call
    // above lands in a different runtime instance and the shell's
    // shared `route` proxy never seeds `params.name`. Reading the
    // pathname sidesteps that mismatch entirely — every catalog click
    // full-remounts this view, so we don't need reactive route params
    // to track nav.
    currentName: signal<string>(''),
  }),

  derived: {
    entry: ({ allCatalog, currentName }) => {
      const name = currentName()
      if (!name) return null
      return (allCatalog() as CatalogEntry[]).find(e => e.id === name) ?? null
    },
    hasProps: ({ entry }) => {
      const e = entry() as CatalogEntry | null
      return e !== null && (e.props ?? []).length > 0
    },
    matrixAxes: ({ entry }) => {
      const e = entry() as CatalogEntry | null
      // Skip the matrix for view-kind entries (iframe previews) and any
      // sui-kind component without enum props.
      if (!e || e.kind !== 'sui') return [] as MatrixAxis[]
      return deriveAxes(e.props ?? [])
    },
    matrixCells: ({ matrixAxes }) => buildCells(matrixAxes() as MatrixAxis[]),
    matrixAxesLabel: ({ matrixAxes }) => {
      const axes = matrixAxes() as MatrixAxis[]
      if (axes.length === 0) return ''
      return axes.map(a => a.name).join(' × ')
    },
  },

  on: {
    setProp: (state, propName: string, value: unknown) => {
      state.propValues = {
        ...state.propValues,
        [propName]: value,
      }
    },
    copyCode: (state, entry: CatalogEntry | null) => {
      if (!entry) return
      const snippet = buildSnippet(
        entry.tag,
        state.propValues,
        entry.props ?? [],
        SLOT_COMPOSITIONS[entry.tag],
      )
      void navigator.clipboard.writeText(snippet)
      state.copyFeedback = true
      setTimeout(() => { state.copyFeedback = false }, 1500)
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
    state.currentName = name
    const entry = all.find(e => e.id === name)

    if (!entry) {
      state.loading = false
      return
    }

    // Seed prop defaults. For size props, prefer the largest enum variant
    // so small components (badge, kbd, status-dot) are legible in the preview.
    const initVals: Record<string, unknown> = {}
    for (const prop of entry.props ?? []) {
      if (prop.name === 'size' && prop.values && prop.values.length > 0) {
        initVals[prop.name] = prop.values[prop.values.length - 1]
      } else if (prop.default !== undefined) {
        initVals[prop.name] = prop.default
      }
    }
    // URL ?key=value overrides defaults so shared links land on the right state.
    const fromUrl = readSearchParams(location.search, entry.props ?? [])
    Object.assign(initVals, fromUrl)
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
    ensureIsland(entry.tag)
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
      node.replaceChildren(document.createTextNode(slotLabel(entry.tag)))
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
      for (const sel of el.querySelectorAll<HTMLSelectElement>('.prop-select')) {
        const propName = sel.id.replace(/^prop-/, '')
        const v = state.propValues[propName]
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
    const props = entry.props ?? []
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

    // Reactively sync propValues → element attributes + URL search params,
    // and re-highlight the matching matrix cell so the deep-link stays
    // visible as the user clicks around the prop panel. The render-slot
    // div carries the `morph` directive so any controller-driven re-render
    // patches in place rather than wholesale-replacing the live preview
    // node — keeps focus / scroll / custom-element state stable while
    // the prop panel toggles attributes on the host.
    const stopEffect = effect(() => {
      const vals = state.propValues
      applyProps(vals)
      writeSearchParams(vals, props)
      highlightActiveCell(el, vals)
    })

    // Populate state-matrix cells. Independent of `state.propValues` —
    // the prop panel only drives the single preview above.
    mountMatrixCells(el, entry.tag, (tag) => SLOT_COMPOSITIONS[tag])
    highlightActiveCell(el, state.propValues)

    // Leave the node attached — next mount reuses it (same tag) or
    // swaps it (different tag). Just stop the prior reactive binding
    // so attribute writes don't leak across components.
    state._dispose = () => { stopEffect() }
  },

  unmount: (state) => {
    state._dispose?.()
  },
})
