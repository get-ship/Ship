import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './sui-va-activity.sui'

interface Entry {
  id: string
  kind: 'annotation' | 'compiled' | 'errored'
  label: string
  detail: string
  ts: number
  tsFmt: string
}

const MAX_ENTRIES = 50

function formatTs(ms: number): string {
  const d = new Date(ms)
  const hh = String(d.getHours()).padStart(2, '0')
  const mm = String(d.getMinutes()).padStart(2, '0')
  const ss = String(d.getSeconds()).padStart(2, '0')
  return `${hh}:${mm}:${ss}`
}

export default controller<Scope>({
  scope: () => {
    const entries = signal<Entry[]>([])
    return { entries }
  },

  mount(state, host) {
    const s = state as unknown as {
      entries: ReturnType<typeof signal<Entry[]>>
      view: () => string
    }

    let counter = 0

    function push(entry: Omit<Entry, 'id' | 'tsFmt'>): void {
      counter += 1
      const e: Entry = {
        ...entry,
        id: `va-${counter}`,
        tsFmt: formatTs(entry.ts),
      }
      const next = [e, ...s.entries()].slice(0, MAX_ENTRIES)
      s.entries.set(next)
    }

    function matchesView(detailView: string | undefined): boolean {
      const v = s.view()
      if (!v) return true
      if (!detailView) return true
      return detailView === v
    }

    function onAnnotation(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string; selector?: string; label?: string; ts?: number } | undefined
      if (!d || !matchesView(d.view)) return
      push({
        kind: 'annotation',
        label: d.label ?? 'Annotation',
        detail: `selector: ${d.selector ?? '?'}`,
        ts: d.ts ?? Date.now(),
      })
    }
    function onCompiled(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string; ms?: number; ts?: number } | undefined
      if (!d || !matchesView(d.view)) return
      push({
        kind: 'compiled',
        label: 'View compiled',
        detail: d.ms != null ? `${d.ms}ms` : '',
        ts: d.ts ?? Date.now(),
      })
    }
    function onErrored(e: Event): void {
      const d = (e as CustomEvent).detail as { view?: string; errors?: Array<{ message: string }>; ts?: number } | undefined
      if (!d || !matchesView(d.view)) return
      push({
        kind: 'errored',
        label: 'Compile failed',
        detail: (d.errors ?? []).map((er) => er.message).join('\n'),
        ts: d.ts ?? Date.now(),
      })
    }

    document.addEventListener('sui:view:annotation', onAnnotation)
    document.addEventListener('sui:view:compiled', onCompiled)
    document.addEventListener('sui:view:errored', onErrored)

    Object.defineProperty(host, 'clear', {
      configurable: true,
      value: () => s.entries.set([]),
    })

    return () => {
      document.removeEventListener('sui:view:annotation', onAnnotation)
      document.removeEventListener('sui:view:compiled', onCompiled)
      document.removeEventListener('sui:view:errored', onErrored)
    }
  },
})
