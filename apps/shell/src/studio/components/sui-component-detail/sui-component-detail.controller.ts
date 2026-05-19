import { controller, signal, effect, ref, chrome, route } from '@ship/sui-runtime'
import type { Scope } from './sui-component-detail.sui'
import {
  loadComponents,
  findEntry,
  type ComponentEntry,
  type PropEntry,
} from '../catalog'
import {
  deriveAxes,
  buildCells,
  mountMatrixCells,
  highlightActiveCell,
  type MatrixAxis,
  type MatrixCell,
} from '../matrix'
import { SLOT_COMPOSITIONS } from '../slot-compositions'
import { buildSnippet } from '../snippet'
import { seedDefaults, createLiveNode, applyProps, isBoolType } from '../render'

// Lazy global route — deferred past module eval (SSR-safe). The tag id of
// the component being inspected is the `:sub` segment of
// `/studio/components/<tag>`.
let _route: ReturnType<typeof route> | null = null
const rt = (): ReturnType<typeof route> => (_route ??= route())
const currentTag = (): string => rt().params.sub() ?? ''

export type KnobKind = 'enum' | 'bool' | 'text'
export interface Knob {
  name: string
  kind: KnobKind
  values: string[]
  placeholder: string
}

export function knobFor(p: PropEntry): Knob {
  const kind: KnobKind = (p.values && p.values.length > 0)
    ? 'enum'
    : isBoolType(p.type) ? 'bool' : 'text'
  return { name: p.name, kind, values: p.values ?? [], placeholder: p.default ?? '' }
}

function axesLabel(axes: readonly MatrixAxis[]): string {
  return axes.map((a) => a.name).join(' × ')
}

export default controller<Scope>({
  scope: () => {
    const entries = signal<ComponentEntry[]>([])
    const loading = signal(true)
    const propValues = signal<Record<string, unknown>>({})
    const copyFeedback = signal(false)
    const matrixEl = ref<HTMLElement>(null)
    const stageEl = ref<HTMLElement>(null)

    const entryFor = (): ComponentEntry | null =>
      findEntry(entries() as ComponentEntry[], currentTag())

    // Re-seed knob defaults whenever the resolved entry changes (the same
    // detail element is reused when navigating tag → tag).
    effect(() => {
      const e = entryFor()
      if (e) propValues.set(seedDefaults(e.props))
    })

    // (Re)build the single live preview node. It persists across knob
    // edits — the prop effect below mutates its attributes in place — and
    // swaps only when the tag changes.
    let liveNode: HTMLElement | null = null
    let liveTag: string | null = null
    effect(() => {
      const e = entryFor()
      const stage = stageEl()
      if (!e || !stage) return
      if (liveTag !== e.tag) {
        liveNode = createLiveNode(e.tag)
        liveTag = e.tag
      }
      if (liveNode && liveNode.parentElement !== stage) stage.replaceChildren(liveNode)
    })

    // Fill matrix hosts after the template renders the cell divs. Reading
    // entryFor() registers the dependency; the {#for} hosts materialize in
    // the same render pass, so defer a tick before instantiating.
    effect(() => {
      const e = entryFor()
      const host = matrixEl()
      if (!e || !host) return
      setTimeout(() => {
        mountMatrixCells(host, e.tag, (t) => SLOT_COMPOSITIONS[t])
        highlightActiveCell(host, propValues())
      }, 0)
    })

    // Apply knob values → live node + re-highlight the matching cell.
    effect(() => {
      const vals = propValues()
      const e = entryFor()
      if (liveNode && e) applyProps(liveNode, vals, e.props)
      const host = matrixEl()
      if (host) highlightActiveCell(host, vals)
    })

    return {
      entries, loading, propValues, copyFeedback, matrixEl, stageEl,

      back: () => chrome.navigate('/studio/components'),
      setProp: (name: string, value: unknown) => {
        propValues.set({ ...propValues(), [name]: value })
      },
      resetKnobs: () => {
        const e = entryFor()
        if (e) propValues.set(seedDefaults(e.props))
      },
      copyCode: () => {
        const e = entryFor()
        if (!e) return
        const code = buildSnippet(e.tag, propValues(), e.props, SLOT_COMPOSITIONS[e.tag])
        void navigator.clipboard?.writeText(code)
        copyFeedback.set(true)
        setTimeout(() => copyFeedback.set(false), 1500)
      },
    }
  },

  derived: {
    currentTag: () => currentTag(),
    entry: ({ entries }) => findEntry(entries() as ComponentEntry[], currentTag()),
    knobs: ({ entries }): Knob[] => {
      const e = findEntry(entries() as ComponentEntry[], currentTag())
      return (e?.props ?? []).map(knobFor)
    },
    matrixCells: ({ entries }): MatrixCell[] => {
      const e = findEntry(entries() as ComponentEntry[], currentTag())
      return e ? buildCells(deriveAxes(e.props)) : []
    },
    matrixAxesLabel: ({ entries }): string => {
      const e = findEntry(entries() as ComponentEntry[], currentTag())
      return e ? axesLabel(deriveAxes(e.props)) : ''
    },
    snippet: ({ entries, propValues }): string => {
      const e = findEntry(entries() as ComponentEntry[], currentTag())
      if (!e) return ''
      return buildSnippet(e.tag, propValues(), e.props, SLOT_COMPOSITIONS[e.tag])
    },
    copyLabel: ({ copyFeedback }) => (copyFeedback() ? 'Copied' : 'Copy'),
  },

  // Non-async so the return type stays `void` (MountSection rejects a
  // returned Promise). The scope effects react once `entries` lands.
  mount: (state) => {
    void loadComponents().then((entries) => {
      state.entries = entries
      state.loading = false
    })
  },
})
