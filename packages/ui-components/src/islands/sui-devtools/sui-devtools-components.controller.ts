import { controller, signal, derived, ref } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-components.sui'

interface CemAttr { name: string; description?: string; type?: { text?: string }; default?: string }
interface CemDecl { kind: string; name: string; tagName?: string; customElement?: boolean; description?: string; attributes?: CemAttr[]; slots?: Array<{ name: string; description?: string }> }
interface CemModule { kind: string; path: string; declarations?: CemDecl[] }

interface Entry {
  tag: string
  description: string
  firstDesc: string
  snippet: string
  slotsLabel: string
  attributes: CemAttr[]
  slots: Array<{ name: string; description?: string }>
}

function flattenCem(modules: CemModule[]): Entry[] {
  const out: Entry[] = []
  for (const m of modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName) continue
      const description = (d.description ?? '').trim()
      const attributes = d.attributes ?? []
      const slots = d.slots ?? []
      const snipAttrs = attributes.slice(0, 3)
        .filter(a => !a.default || a.default === 'false')
        .map(a => (a.type?.text ?? '').includes('boolean') ? ` ${a.name}` : ` ${a.name}="…"`)
        .join('')
      const hasDefault = slots.some(sl => !sl.name || sl.name === '')
      const snippet = hasDefault
        ? `<${d.tagName}${snipAttrs}>\n  \n</${d.tagName}>`
        : `<${d.tagName}${snipAttrs}></${d.tagName}>`
      out.push({
        tag: d.tagName,
        description,
        firstDesc: description.split('\n')[0] ?? '',
        snippet,
        slotsLabel: slots.map(sl => sl.name || '(default)').join(', '),
        attributes,
        slots,
      })
    }
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag))
}

export default controller<Scope>({
  scope: () => {
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
    const s = state as Record<string, unknown>
    let loaded = false

    async function load(): Promise<void> {
      if (loaded) return
      loaded = true
      try {
        const resp = await fetch('/api/assets/custom-elements.json', { credentials: 'same-origin' })
        if (!resp.ok) return
        const cem = await resp.json() as { modules: CemModule[] }
        s['entries'] = flattenCem(cem.modules)
      } catch { /* best-effort */ }
    }

    // Copy click delegation — needs host for notify dispatch
    const bodyRefFn = s['bodyEl']
    const bodyEl = (typeof bodyRefFn === 'function' ? (bodyRefFn as () => HTMLElement | null)() : null)
    if (bodyEl) {
      bodyEl.addEventListener('click', (e: MouseEvent) => {
        const btn = (e.target as Element).closest<HTMLButtonElement>('.copy[data-tag]')
        if (!btn) return
        e.preventDefault()
        e.stopPropagation()
        const tag = btn.dataset.tag ?? ''
        const entry = (s['entries'] as Entry[]).find(x => x.tag === tag)
        if (!entry) return
        void navigator.clipboard.writeText(entry.snippet).then(
          () => host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: 'Copied' }, bubbles: true, composed: true })),
          () => host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'warning', title: 'Copy failed' }, bubbles: true, composed: true })),
        )
      })
    }

    Object.defineProperty(host, 'refresh', {
      value: () => { loaded = false; s['entries'] = []; void load() },
      configurable: true,
    })

    void load()
  },
})
