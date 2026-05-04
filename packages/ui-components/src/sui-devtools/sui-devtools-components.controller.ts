import { controller, signal, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-components.sui'

interface CemAttr { name: string; description?: string; type?: { text?: string }; default?: string }
interface CemDecl { kind: string; name: string; tagName?: string; customElement?: boolean; description?: string; attributes?: CemAttr[]; slots?: Array<{ name: string; description?: string }> }
interface CemModule { kind: string; path: string; declarations?: CemDecl[] }

interface Entry {
  tag: string
  description: string
  attributes: CemAttr[]
  slots: Array<{ name: string; description?: string }>
}

function flattenCem(modules: CemModule[]): Entry[] {
  const out: Entry[] = []
  for (const m of modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName) continue
      out.push({ tag: d.tagName, description: (d.description ?? '').trim(), attributes: d.attributes ?? [], slots: d.slots ?? [] })
    }
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag))
}

function snippet(e: Entry): string {
  const attrs = e.attributes.slice(0, 3)
    .filter(a => !a.default || a.default === 'false')
    .map(a => (a.type?.text ?? '').includes('boolean') ? ` ${a.name}` : ` ${a.name}="…"`)
    .join('')
  const hasDefault = e.slots.some(s => !s.name || s.name === '')
  return hasDefault ? `<${e.tag}${attrs}>\n  \n</${e.tag}>` : `<${e.tag}${attrs}></${e.tag}>`
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function entryHtml(e: Entry): string {
  const snip = snippet(e)
  const first = e.description.split('\n')[0] ?? ''
  const attrsHtml = e.attributes.length > 0
    ? `<div><strong>Attributes</strong><dl class="attrs">${e.attributes.map(a => `<dt>${esc(a.name)}${a.default ? `=${esc(a.default)}` : ''}</dt><dd>${esc(a.description ?? '')}</dd>`).join('')}</dl></div>`
    : ''
  const slotsHtml = e.slots.length > 0
    ? `<div><strong>Slots</strong>: ${e.slots.map(s => esc(s.name || '(default)')).join(', ')}</div>`
    : ''
  return `<details><summary><span class="tag">&lt;${esc(e.tag)}&gt;</span><span class="desc">${esc(first)}</span><button class="copy" data-snip="${esc(snip)}" title="Copy snippet">Copy</button></summary><div class="panel"><pre>${esc(snip)}</pre>${attrsHtml}${slotsHtml}</div></details>`
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const filterInput = scope.filterInput() as HTMLInputElement
    const countEl = scope.countEl() as HTMLElement
    const bodyEl = scope.bodyEl() as HTMLElement

    const entries = signal<Entry[]>([])
    const filter = signal('')
    let loaded = false

    async function load(): Promise<void> {
      if (loaded) return
      loaded = true
      try {
        const resp = await fetch('/api/assets/custom-elements.json', { credentials: 'same-origin' })
        if (!resp.ok) { bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">fetch failed: ${resp.status}</div>`; return }
        const cem = await resp.json() as { modules: CemModule[] }
        entries.set(flattenCem(cem.modules))
      } catch (e) {
        bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">${esc((e as Error).message)}</div>`
      }
    }

    filterInput.addEventListener('input', () => filter.set(filterInput.value))

    // Click-delegate copy buttons
    bodyEl.addEventListener('click', (e: MouseEvent) => {
      const btn = (e.target as Element).closest<HTMLButtonElement>('.copy')
      if (!btn) return
      e.preventDefault()
      e.stopPropagation()
      const snip = btn.dataset.snip ?? ''
      navigator.clipboard.writeText(snip).then(
        () => host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: 'Copied' }, bubbles: true, composed: true })),
        () => host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'warning', title: 'Copy failed' }, bubbles: true, composed: true })),
      )
    })

    const stopFx = effect(() => {
      const all = entries()
      const f = filter()
      if (all.length === 0) {
        bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="package"></sui-icon><span slot="title">Loading components…</span>Fetching the component catalog from the daemon.</sui-empty-state>`
        countEl.textContent = ''
        return
      }
      const needle = f.toLowerCase()
      const shown = needle ? all.filter(e => e.tag.toLowerCase().includes(needle) || e.description.toLowerCase().includes(needle)) : all
      countEl.textContent = `${shown.length} / ${all.length}`
      if (shown.length === 0) {
        bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="search"></sui-icon><span slot="title">No matches</span>No components match the current filter.</sui-empty-state>`
      } else {
        bodyEl.innerHTML = shown.map(entryHtml).join('')
      }
    })

    Object.defineProperty(host, 'refresh', {
      value: () => { loaded = false; entries.set([]); void load() },
      configurable: true,
    })

    void load()

    return () => stopFx()
  },
})
