import { DECK_PREFIX } from '@ship/sdk-client'
import { chrome, controller, signal, derived, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-components.sui'

// The dock's Components tab inspects the workspace SUI catalog —
// `/_deck/catalog/workspace` returns every `<sui-*>` the monorepo
// knows about (packaged pack components + non-packaged shell/view
const SUI_CATALOG_URL = `${DECK_PREFIX}/catalog/workspace`

interface CemAttr { name: string; description?: string; type?: { text?: string }; default?: string }
interface CatalogEntryWire {
  tagName: string
  description?: string
  attributes?: CemAttr[]
  slots?: Array<{ name: string; description?: string }>
}

interface Entry {
  tag: string
  description: string
  firstDesc: string
  snippet: string
  slotsLabel: string
  attributes: CemAttr[]
  slots: Array<{ name: string; description?: string }>
}

function flattenCatalog(entries: CatalogEntryWire[]): Entry[] {
  const out: Entry[] = []
  for (const e of entries ?? []) {
    if (!e.tagName) continue
    const description = (e.description ?? '').trim()
    const attributes = e.attributes ?? []
    const slots = e.slots ?? []
    const snipAttrs = attributes.slice(0, 3)
      .filter(a => !a.default || a.default === 'false')
      .map(a => (a.type?.text ?? '').includes('boolean') ? ` ${a.name}` : ` ${a.name}="…"`)
      .join('')
    const hasDefault = slots.some(sl => !sl.name || sl.name === '')
    const snippet = hasDefault
      ? `<${e.tagName}${snipAttrs}>\n  \n</${e.tagName}>`
      : `<${e.tagName}${snipAttrs}></${e.tagName}>`
    out.push({
      tag: e.tagName,
      description,
      firstDesc: description.split('\n')[0] ?? '',
      snippet,
      slotsLabel: slots.map(sl => sl.name || '(default)').join(', '),
      attributes,
      slots,
    })
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag))
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const entries = signal<Entry[]>([])
    const filter = signal('')
    const shown = derived(() => {
      const all = entries()
      const needle = filter().toLowerCase()
      return needle
        ? all.filter(e => e.tag.toLowerCase().includes(needle) || e.description.toLowerCase().includes(needle))
        : all
    })
    const bodyEl = ref<HTMLDivElement>(null)
    return { entries, filter, shown, bodyEl }
  },

  mount(state, host) {
    let loaded = false

    async function load(): Promise<void> {
      if (loaded) return
      loaded = true
      try {
        const resp = await fetch(SUI_CATALOG_URL, { credentials: 'same-origin' })
        if (!resp.ok) return
        const doc = await resp.json() as { entries?: CatalogEntryWire[] }
        state.entries = flattenCatalog(doc.entries ?? [])
      } catch { /* best-effort */ }
    }

    // Copy click delegation — needs host for notify dispatch
    const bodyEl = state.bodyEl
    if (bodyEl) {
      bodyEl.addEventListener('click', (e: MouseEvent) => {
        const btn = (e.target as Element).closest<HTMLButtonElement>('.copy[data-tag]')
        if (!btn) return
        e.preventDefault()
        e.stopPropagation()
        const tag = btn.dataset.tag ?? ''
        const entry = state.entries.find(x => x.tag === tag)
        if (!entry) return
        void navigator.clipboard.writeText(entry.snippet).then(
          () => chrome.notify({ tone: 'success', title: 'Copied' }),
          () => chrome.notify({ tone: 'warning', title: 'Copy failed' }),
        )
      })
    }

    Object.defineProperty(host, 'refresh', {
      value: () => { loaded = false; state.entries = []; void load() },
      configurable: true,
    })

    void load()
  },
})
