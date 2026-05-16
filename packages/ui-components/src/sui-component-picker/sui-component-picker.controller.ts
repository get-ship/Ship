import { controller, signal, derived, ref, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-component-picker.sui'
import { isPickerHidden } from '../picker-hidden-tags'
import { SLOT_COMPOSITIONS } from '../picker-slot-compositions'

// Default text label per atomic component when no composition is
// declared. Keyed by tag for components whose default-slot doesn't
// accept arbitrary text or needs specific attrs.
const DEFAULT_ATTRS: Record<string, Record<string, string>> = {
  'sui-icon':      { name: 'package',  size: 'md' },
  'sui-app-icon':  { name: 'dashboard', size: 'md' },
  'sui-avatar':    { initials: 'AB' },
  'sui-badge':     {},
  'sui-button':    {},
  'sui-status-dot':{ tone: 'success' },
  'sui-skeleton':  { width: '60px', height: '12px' },
  'sui-stat':      { label: 'Stat', value: '42' },
  'sui-progress':  { value: '60' },
  'sui-spinner':   {},
  'sui-kbd':       {},
  'sui-tag':       {},
  'sui-chip':      {},
  'sui-divider':   {},
  'sui-separator': {},
}

const DEFAULT_TEXT: Record<string, string> = {
  'sui-badge': 'NEW',
  'sui-button': 'Click me',
  'sui-tag': 'tag',
  'sui-chip': 'chip',
  'sui-kbd': '⌘K',
  'sui-link': 'link',
  'sui-alert': 'Heads up — this is an alert.',
  'sui-error-state': 'Something went wrong',
  'sui-empty-state': 'No items yet',
  'sui-loading': 'Loading…',
}

// Parse a slot composition HTML blob into a DocumentFragment via
// DOMParser. Lint-clean (no innerHTML writes) and the parsed
// document is detached so no mutations happen on the live tree.
function parseFragment(html: string): DocumentFragment {
  const doc = new DOMParser().parseFromString(`<body>${html}</body>`, 'text/html')
  const frag = document.createDocumentFragment()
  const body = doc.body
  while (body.firstChild) frag.appendChild(body.firstChild)
  return frag
}

function populatePreview(container: HTMLElement, tag: string): void {
  // Clear prior render so re-renders don't double-stack.
  while (container.firstChild) container.removeChild(container.firstChild)
  if (!customElements.get(tag)) {
    // Element not registered yet — fall back to the tag name as text.
    const label = document.createElement('code')
    label.textContent = `<${tag}>`
    label.style.opacity = '0.6'
    container.appendChild(label)
    return
  }
  const el = document.createElement(tag)
  const attrs = DEFAULT_ATTRS[tag] ?? {}
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  const comp = SLOT_COMPOSITIONS[tag]
  if (comp) {
    el.appendChild(parseFragment(comp))
  } else {
    el.textContent = DEFAULT_TEXT[tag] ?? 'Example'
  }
  container.appendChild(el)
}

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
      // Hide chrome / container tags — they don't render standalone
      // and confuse users browsing for primitives to insert.
      if (isPickerHidden(d.tagName)) continue
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

    // Live preview population. Watches the shadow root for any
    // `.tile-preview[data-tag]` element appearing/disappearing and
    // populates with a real <tag> instance. Survives template
    // re-renders driven by filter/search changes.
    const root = host.shadowRoot
    if (root) {
      const populate = (): void => {
        for (const el of root.querySelectorAll<HTMLElement>('.tile-preview[data-tag]')) {
          if (el.dataset['previewed'] === '1') continue
          const tag = el.dataset['tag']
          if (!tag) continue
          populatePreview(el, tag)
          el.dataset['previewed'] = '1'
        }
      }
      const obs = new MutationObserver(populate)
      obs.observe(root, { childList: true, subtree: true })
      // Initial pass after the first paint so any tiles already
      // present (modal mode renders immediately) get filled.
      requestAnimationFrame(populate)
    }

    void load()
  },
})
