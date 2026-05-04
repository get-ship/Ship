import { controller, signal, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-signals.sui'

function collectSignals(): Record<string, unknown> {
  const ds = (document as unknown as { _ds?: { signals?: unknown } })._ds
  if (ds?.signals) {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(ds.signals as Record<string, unknown>))
      out[k] = (v as { value?: unknown })?.value ?? v
    return out
  }
  const alt = (window as unknown as { ds?: { signals?: unknown } }).ds
    ?? (globalThis as unknown as { __ds?: { signals?: unknown } }).__ds
  if (alt?.signals) {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(alt.signals as Record<string, unknown>))
      out[k] = (v as { value?: unknown })?.value ?? v
    return out
  }
  const out: Record<string, unknown> = {}
  document.querySelectorAll('[data-signals]').forEach((el) => {
    const raw = el.getAttribute('data-signals')
    if (!raw) return
    try { Object.assign(out, new Function(`return (${raw})`)()) } catch { /* ignore */ }
  })
  const view = document.getElementById('ship-view-root')
  if (view) {
    const walk = (node: Element): void => {
      for (const attr of Array.from(node.attributes)) {
        if (attr.name.startsWith('data-signals:')) {
          const name = attr.name.slice('data-signals:'.length)
          try { out[name] = new Function(`return (${attr.value})`)() } catch { out[name] = attr.value }
        }
      }
      for (const child of Array.from(node.children)) walk(child)
    }
    walk(view)
  }
  return out
}

function renderValue(v: unknown): string {
  if (typeof v === 'string') return v.length > 80 ? `"${v.slice(0, 77)}…"` : `"${v}"`
  try { return JSON.stringify(v) } catch { return String(v) }
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const filterInput = scope.filterInput() as HTMLInputElement
    const countEl = scope.countEl() as HTMLElement
    const bodyEl = scope.bodyEl() as HTMLElement

    const sigs = signal<Record<string, unknown>>({})
    const filter = signal('')

    let timer: ReturnType<typeof setInterval> | null = null

    const onDockActivate = (ev: Event) => {
      const tab = (ev as CustomEvent<{ tab: string }>).detail?.tab
      if (tab === 'signals') _start()
      else _stop()
    }

    const dock = host.closest('sui-panel-dock') as HTMLElement | null
    dock?.addEventListener('sui:dock:activate', onDockActivate)
    if (dock?.getAttribute('active-tab') === 'signals') _start()

    function _start(): void {
      if (timer) return
      _sample()
      timer = setInterval(_sample, 500)
    }

    function _stop(): void {
      if (!timer) return
      clearInterval(timer)
      timer = null
    }

    function _sample(): void { sigs.set(collectSignals()) }

    filterInput.addEventListener('input', () => filter.set(filterInput.value))

    const stopFx = effect(() => {
      const s = sigs()
      const f = filter()
      const keys = Object.keys(s).sort()
      const needle = f.toLowerCase()
      const shown = needle ? keys.filter(k => k.toLowerCase().includes(needle)) : keys
      countEl.textContent = `${shown.length} / ${keys.length}`
      if (shown.length === 0) {
        bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="${needle ? 'search' : 'radio'}"></sui-icon><span slot="title">${needle ? 'No matches' : 'No signals'}</span>${needle ? 'No signals match the current filter.' : 'Live events will appear here when actors emit.'}</sui-empty-state>`
      } else {
        bodyEl.innerHTML = shown.map(k =>
          `<div class="row"><span class="k">${esc(k)}</span><span class="v">${esc(renderValue(s[k]))}</span></div>`
        ).join('')
      }
    })

    /** Public poke for the dock to force an immediate sample. */
    Object.defineProperty(host, 'refresh', { value: () => _sample(), configurable: true })

    return () => {
      stopFx()
      _stop()
      dock?.removeEventListener('sui:dock:activate', onDockActivate)
    }
  },
})
