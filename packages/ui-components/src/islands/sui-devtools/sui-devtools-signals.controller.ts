import { controller, signal, derived } from '@ship/ui-runtime'
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

export default controller<Scope>({
  scope: () => {
    const sigs = signal<Record<string, unknown>>({})
    const filter = signal('')
    const allCount = derived(() => Object.keys(sigs()).length)
    const shownEntries = derived(() => {
      const s = sigs()
      const needle = filter().toLowerCase()
      const keys = Object.keys(s).sort()
      const filtered = needle ? keys.filter(k => k.toLowerCase().includes(needle)) : keys
      return filtered.map(k => ({ key: k, value: renderValue(s[k]) }))
    })
    return { sigs, filter, allCount, shownEntries }
  },

  mount(state, host) {
    const s = state as Record<string, unknown>
    let timer: ReturnType<typeof setInterval> | null = null

    function _sample(): void { s['sigs'] = collectSignals() }

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

    const onDockActivate = (ev: Event) => {
      const tab = (ev as CustomEvent<{ tab: string }>).detail?.tab
      if (tab === 'signals') _start()
      else _stop()
    }

    const dock = host.closest('sui-panel-dock') as HTMLElement | null
    dock?.addEventListener('sui:dock:activate', onDockActivate)
    if (dock?.getAttribute('active-tab') === 'signals') _start()

    Object.defineProperty(host, 'refresh', { value: () => _sample(), configurable: true })

    return () => {
      _stop()
      dock?.removeEventListener('sui:dock:activate', onDockActivate)
    }
  },
})
