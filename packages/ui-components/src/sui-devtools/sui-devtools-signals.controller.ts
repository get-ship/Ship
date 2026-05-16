import { controller, signal, derived } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-signals.sui'

// The signal tree reads from `window.__suiCurrentScope`, published by
// apps/shell2/src/main.ts after every successful mountRoute. The handle's
// scope is a record of signal getters + plain values + framework helpers
// (`ship`, `route`, `navigate`); we surface the signal/value entries and
// hide the helpers (callable framework objects without `__peek`).

// Keys exposed by the runtime in the public scope that aren't useful in
// the devtools signal tree — framework helpers, not view state.
const FRAMEWORK_KEYS = new Set(['ship', 'route', 'navigate'])

interface SignalEntry { key: string; value: string }

type MaybeSignal = ((...args: unknown[]) => unknown) & { __peek?: () => unknown }

function isSignalGetter(v: unknown): v is MaybeSignal {
  return typeof v === 'function' && typeof (v as MaybeSignal).__peek === 'function'
}

function collectSignals(): Record<string, unknown> {
  const scope = (window as unknown as { __suiCurrentScope?: Record<string, unknown> | null }).__suiCurrentScope
  if (!scope) return {}
  const out: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(scope)) {
    if (FRAMEWORK_KEYS.has(k)) continue
    if (isSignalGetter(v)) {
      // Use __peek() — reading via the tracked getter inside a non-reactive
      // context still works but `__peek` is the canonical non-tracking
      // read and avoids polluting any ambient observer.
      try { out[k] = v.__peek!() } catch { /* skip uncallable */ }
      continue
    }
    if (typeof v === 'function') continue // method on the scope, not state
    out[k] = v
  }
  return out
}

function renderValue(v: unknown): string {
  if (v === undefined) return 'undefined'
  if (typeof v === 'string') return v.length > 80 ? `"${v.slice(0, 77)}…"` : `"${v}"`
  if (typeof v === 'function') return '<fn>'
  try { return JSON.stringify(v) } catch { return String(v) }
}

export default controller<Scope>({
  scope: () => {
    const sigs = signal<Record<string, unknown>>({})
    const filter = signal('')
    const allCount = derived(() => Object.keys(sigs()).length)
    const shownEntries = derived<SignalEntry[]>(() => {
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

    function sample(): void { s['sigs'] = collectSignals() }

    function start(): void {
      if (timer) return
      sample()
      timer = setInterval(sample, 500)
    }

    function stop(): void {
      if (!timer) return
      clearInterval(timer)
      timer = null
    }

    const onDockActivate = (ev: Event) => {
      const tab = (ev as CustomEvent<{ tab: string }>).detail?.tab
      if (tab === 'signals') start()
      else stop()
    }

    // Re-sample immediately on view navigation (handle replaced).
    const onScopeChange = () => sample()

    const dock = host.closest('sui-panel-dock') as HTMLElement | null
    dock?.addEventListener('sui:dock:activate', onDockActivate)
    document.addEventListener('sui:scope:change', onScopeChange)
    if (dock?.getAttribute('active-tab') === 'signals') start()

    Object.defineProperty(host, 'refresh', { value: () => sample(), configurable: true })

    return () => {
      stop()
      dock?.removeEventListener('sui:dock:activate', onDockActivate)
      document.removeEventListener('sui:scope:change', onScopeChange)
    }
  },
})
