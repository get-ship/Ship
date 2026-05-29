import { controller, signal, effect, derived } from '@ship/sui-runtime'
import { defineRoutes } from '@ship/sui-runtime/route'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { loadCatalog, type CatalogEntry, type PropEntry } from '../catalog'
import {
  buildCells,
  deriveAxes,
  mountMatrixCells,
  type MatrixAxis,
} from './matrix'
import { slotLabel } from './slot-compositions'
import { buildSnippet } from './snippet'
import { readSearchParams, writeSearchParams, highlightActiveCell } from './deep-link'


const BOOL_TYPES = new Set(['boolean', 'Boolean', 'bool'])
const isBoolType = (t: string | undefined) => t !== undefined && BOOL_TYPES.has(t)

// One playground knob, classified for the template's typed control set:
// enum → segmented buttons, bool → checkbox, string → text input.
interface Knob {
  name: string
  kind: 'enum' | 'bool' | 'string'
  values: string[]
}

function classifyKnob(p: PropEntry): Knob {
  if (Array.isArray(p.values) && p.values.length > 0)
    return { name: p.name, kind: 'enum', values: p.values }
  if (isBoolType(p.type)) return { name: p.name, kind: 'bool', values: [] }
  return { name: p.name, kind: 'string', values: [] }
}

// Canonical access path is /studio/components/:name (the shell route) and
// /sui/studio/components/:name (view-host standalone, e.g. dev-panel popouts).
defineRoutes([
  { pattern: '/studio/components/:name' },
  { pattern: '/sui/studio/components/:name' },
])

// Shared across mounts: one iframe + one preview node at module scope. Holding
// them caps DOM iframe count at 1 (the SSE connection survives navigations) and
// lets the sui-kind preview reuse the same host element across same-tag mounts.
let sharedFrame: HTMLIFrameElement | null = null
let sharedNode: HTMLElement | null = null
let sharedNodeTag: string | null = null

// Components shipped as their own bundle at `/_sui/sui-components/<tag>.js`. The
// daemon injects these by static-scanning view templates for the tag, but the
// studio creates host nodes via createElement at runtime — the daemon can't see
// those, so eagerly fetch the bundle when about to render one of these tags.
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
  import(/* @vite-ignore */ `/_sui/sui-components/${tag}.js`).catch(() => {
    islandsLoaded.delete(tag)
  })
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const allCatalog = signal<CatalogEntry[]>([])
    const loading = signal(true)
    const propValues = signal<Record<string, unknown>>({})
    const _dispose = signal<(() => void) | null>(null)
    const copyFeedback = signal(false)
    // Authoritative subroute slug — seeded from `location.pathname` at
    // mount() rather than `route.params.name()`. The shell pre-registers
    // the dynamic pattern, so the view's own `defineRoutes` lands in a
    // different runtime instance and the shell's shared `route` proxy
    // never seeds `params.name`. Reading the pathname sidesteps that.
    const currentName = signal<string>('')

    const entry = derived(() => {
      const name = currentName()
      if (!name) return null
      return allCatalog().find(e => e.id === name) ?? null
    })
    const hasProps = derived(() => {
      const e = entry()
      return e !== null && (e.props ?? []).length > 0
    })
    // Knobs for the playground, classified into enum / bool / string.
    const knobs = derived(() => {
      const e = entry()
      if (!e) return [] as Knob[]
      return (e.props ?? []).map(classifyKnob)
    })
    const slotList = derived(() => {
      const e = entry()
      return (e?.slots ?? []) as { name?: string; description?: string }[]
    })
    const metaLabel = derived(() => {
      const e = entry()
      if (!e) return ''
      const p = (e.props ?? []).length
      const s = (e.slots ?? []).length
      const parts: string[] = [`${p} prop${p === 1 ? '' : 's'}`]
      if (s > 0) parts.push(`${s} slot${s === 1 ? '' : 's'}`)
      return parts.join(' · ')
    })
    // Live source snippet, recomputed as the knobs change.
    const snippet = derived(() => {
      const e = entry()
      if (!e) return ''
      return buildSnippet(e.tag, propValues(), e.props ?? [], e.examples?.[0]?.slots)
    })
    const matrixAxes = derived(() => {
      const e = entry()
      if (!e || e.kind !== 'sui') return [] as MatrixAxis[]
      return deriveAxes(e.props ?? [])
    })
    const matrixCells = derived(() => buildCells(matrixAxes()))
    const matrixAxesLabel = derived(() => {
      const axes = matrixAxes()
      if (axes.length === 0) return ''
      return axes.map(a => a.name).join(' × ')
    })
    return {
      allCatalog, loading, propValues, _dispose, copyFeedback, currentName,
      entry, hasProps, knobs, slotList, metaLabel, snippet, matrixAxes, matrixCells, matrixAxesLabel,
    }
  },

  on: {
    setProp: (state, propName: string, value: unknown) => {
      state.propValues = { ...state.propValues, [propName]: value }
    },
    copyCode: (state, entry: CatalogEntry | null) => {
      if (!entry) return
      const snippet = buildSnippet(
        entry.tag,
        state.propValues,
        entry.props ?? [],
        entry.examples?.[0]?.slots,
      )
      void navigator.clipboard.writeText(snippet)
      state.copyFeedback = true
      setTimeout(() => { state.copyFeedback = false }, 1500)
    },
  },

  // mount may be async; the runtime awaits it. Mount's return type is
  // sync-only; cast through unknown to satisfy the inferred section contract.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  mount: (async (state: any, el: HTMLElement) => {
    const all = await loadCatalog()
    state.allCatalog = all

    const segments = location.pathname.split('/').filter(Boolean)
    const name = segments[segments.length - 1] ?? ''
    state.currentName = name
    const entry = all.find(e => e.id === name)

    if (!entry) {
      state.loading = false
      return
    }

    // Seed prop defaults. For size props, prefer the largest enum variant
    // so small components (badge, kbd, status-dot) are legible.
    const initVals: Record<string, unknown> = {}
    for (const prop of entry.props ?? []) {
      if (prop.name === 'size' && prop.values && prop.values.length > 0) {
        initVals[prop.name] = prop.values[prop.values.length - 1]
      } else if (prop.default !== undefined) {
        initVals[prop.name] = prop.default
      }
    }
    const fromUrl = readSearchParams(location.search, entry.props ?? [])
    Object.assign(initVals, fromUrl)
    state.propValues = initVals
    state.loading = false

    // Wait one tick for the template to re-render with loading=false.
    await new Promise<void>(r => setTimeout(r, 0))

    const slot = el.querySelector('[data-render-slot]') as HTMLElement | null
    if (!slot) return

    if (entry.kind === 'view') {
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
      state._dispose = () => {}
      return
    }

    if (sharedFrame?.parentElement) sharedFrame.parentElement.removeChild(sharedFrame)

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
    node.style.transform = ''
    node.style.transformOrigin = ''
    const compose = entry.examples?.[0]?.slots
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
    const ZOOM_TAGS = new Set(['sui-status-dot', 'sui-badge', 'sui-chip', 'sui-spinner'])
    if (ZOOM_TAGS.has(entry.tag)) {
      node.style.transform = 'scale(3)'
      node.style.transformOrigin = 'center'
    }

    // Apply prop values to the rendered host. Boolean attrs follow HTML
    // semantics (presence ≡ true) — strip entirely when falsy. String
    // props: remove on null/empty so the component keeps its declared
    // default.
    const props = entry.props ?? []
    const propTypes = new Map(props.map(p => [p.name, p.type]))
    const applyProps = (vals: Record<string, unknown>) => {
      for (const p of props) {
        const v = vals[p.name]
        const t = propTypes.get(p.name)
        if (isBoolType(t)) {
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
    // and re-highlight the matching matrix cell. The render-slot carries
    // `morph` so controller-driven re-renders patch in place.
    const stopEffect = effect(() => {
      const vals = state.propValues
      applyProps(vals)
      writeSearchParams(vals, props)
      highlightActiveCell(el, vals)
    })

    mountMatrixCells(el, entry.tag, () => entry.examples?.[0]?.slots)
    highlightActiveCell(el, state.propValues)

    state._dispose = () => { stopEffect() }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  }) as unknown as any,

  unmount: (state) => {
    state._dispose?.()
  },
})
