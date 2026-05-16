import { controller, signal, derived, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-component-picker.sui'

const FALLBACK_CATALOG = '/api/assets/sui-custom-elements.json'

interface CemAttr { name: string; description?: string; type?: { text?: string }; default?: string }
interface CemDecl {
  kind: string
  name: string
  tagName?: string
  customElement?: boolean
  description?: string
  attributes?: CemAttr[]
  slots?: Array<{ name: string; description?: string }>
}
interface CemModule { kind: string; path: string; declarations?: CemDecl[] }

export interface PickerEntry {
  tag: string
  description: string
  firstDesc: string
  /** Lucide icon name. Inferred from tag → category mapping; falls back
   *  to `package` when nothing matches. The dock's components-tab uses
   *  the same heuristic so the picker UI stays consistent with it. */
  icon: string
  category: string
}

const CATEGORY_ICON: Record<string, string> = {
  layout: 'layout',
  data: 'database',
  form: 'square-pen',
  feedback: 'message-circle',
  navigation: 'compass',
  overlay: 'layers',
  display: 'eye',
  chart: 'bar-chart-3',
  text: 'type',
  media: 'image',
}

function categoryOf(tag: string): string {
  if (/(card|stack|grid|panel|section|divider|separator|toolbar|page)/.test(tag)) return 'layout'
  if (/(input|textarea|checkbox|radio|select|slider|switch|toggle|field|label|form|file|button)/.test(tag)) return 'form'
  if (/(table|list|tile|tree|json|stat|metachip|kbd|code|markdown|diff|history)/.test(tag)) return 'data'
  if (/(alert|toast|notif|empty|loading|skeleton|progress|spinner|status)/.test(tag)) return 'feedback'
  if (/(nav|rail|breadcrumb|menu|tabs|tab|stepper|launcher|dock)/.test(tag)) return 'navigation'
  if (/(dialog|drawer|popover|tooltip|hover|context-menu|dropdown|portal)/.test(tag)) return 'overlay'
  if (/(chart|donut|gauge|sparkline|heatmap|meter|line|area|bar)/.test(tag)) return 'chart'
  if (/(icon|avatar|app-icon|theme|annotation|edit-overlay)/.test(tag)) return 'display'
  return 'display'
}

function iconFor(tag: string): string {
  return CATEGORY_ICON[categoryOf(tag)] ?? 'package'
}

function flattenCem(modules: CemModule[]): PickerEntry[] {
  const out: PickerEntry[] = []
  const seen = new Set<string>()
  for (const m of modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName) continue
      if (seen.has(d.tagName)) continue
      seen.add(d.tagName)
      const description = (d.description ?? '').trim()
      out.push({
        tag: d.tagName,
        description,
        firstDesc: description.split('\n')[0] ?? '',
        icon: iconFor(d.tagName),
        category: categoryOf(d.tagName),
      })
    }
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag))
}

function fuzzyMatch(needle: string, hay: string): boolean {
  if (!needle) return true
  const n = needle.toLowerCase()
  const h = hay.toLowerCase()
  if (h.includes(n)) return true
  // Subsequence match: every char of needle appears in order somewhere.
  let i = 0
  for (const c of h) {
    if (c === n[i]) {
      i += 1
      if (i === n.length) return true
    }
  }
  return false
}

interface HostWithProps extends HTMLElement {
  open?: boolean
}

export default controller<Scope>({
  scope: (state, host) => {
    const entries = signal<PickerEntry[]>([])
    const filter = signal('')
    const queue = signal<string[]>([])
    const gridEl = ref<HTMLDivElement>(null)
    const s = state as unknown as {
      categoryFilter: () => string
      selectMode: () => 'single' | 'multi'
      mode: () => 'modal' | 'inline'
    }
    const shown = derived(() => {
      const all = entries()
      const needle = filter().trim()
      const cat = s.categoryFilter().trim().toLowerCase()
      return all.filter((e) => {
        if (cat && e.category !== cat) return false
        if (!needle) return true
        return fuzzyMatch(needle, e.tag) || fuzzyMatch(needle, e.description)
      })
    })
    const isSelected = (tag: string): boolean => queue().includes(tag)

    const h = host as HostWithProps

    function emitPicked(tags: string[]): void {
      host.dispatchEvent(new CustomEvent('sui:component:picked', {
        detail: { tags }, bubbles: true, composed: true,
      }))
    }

    function onTileClick(tag: string): void {
      if (s.selectMode() === 'multi') {
        const q = queue()
        if (q.includes(tag)) queue.set(q.filter((t) => t !== tag))
        else queue.set([...q, tag])
        return
      }
      emitPicked([tag])
      if (s.mode() === 'modal') { h.open = false }
    }

    function commit(): void {
      const tags = queue()
      if (tags.length === 0) return
      emitPicked([...tags])
      queue.set([])
      if (s.mode() === 'modal') h.open = false
    }

    function onDialogClose(): void { h.open = false }

    return {
      entries, filter, queue, gridEl, shown, isSelected,
      onTileClick, commit, onDialogClose,
    }
  },

  mount(state, host) {
    const s = state as unknown as {
      entries: ReturnType<typeof signal<PickerEntry[]>>
      filter: ReturnType<typeof signal<string>>
      catalogUrl: () => string
    }
    const h = host as HostWithProps
    let loaded = false
    let inflight: Promise<void> | null = null

    async function load(): Promise<void> {
      if (loaded) return
      if (inflight) return inflight
      inflight = (async () => {
        try {
          const url = s.catalogUrl() || FALLBACK_CATALOG
          const resp = await fetch(url, { credentials: 'same-origin' })
          if (!resp.ok) return
          const cem = await resp.json() as { modules?: CemModule[] }
          s.entries.set(flattenCem(cem.modules ?? []))
          loaded = true
        } catch { /* best-effort */ }
        finally { inflight = null }
      })()
      return inflight
    }

    // Imperative API.
    Object.defineProperty(host, 'open', {
      configurable: true,
      get: () => h.hasAttribute('open'),
      set: (v: boolean) => {
        if (v) h.setAttribute('open', '')
        else h.removeAttribute('open')
      },
    })
    Object.defineProperty(host, 'refresh', {
      configurable: true,
      value: () => { loaded = false; s.entries.set([]); void load() },
    })
    Object.defineProperty(host, 'close', {
      configurable: true,
      value: () => { h.open = false },
    })

    void load()
  },
})
