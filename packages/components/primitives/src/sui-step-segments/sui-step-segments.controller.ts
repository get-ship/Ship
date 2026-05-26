import { controller, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-step-segments.sui'

const KNOWN = new Set(['done', 'failed', 'pending', 'running'])

interface Segment {
  id: string
  state: string
}

function parseValues(raw: string): string[] {
  return raw
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter((s) => s.length > 0)
    .map((s) => (KNOWN.has(s) ? s : 'pending'))
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({
    segments: signal<Segment[]>([]),
    ariaLabel: signal<string | null>(null),
  }),

  mount(state, _host) {
    const stop = effect(() => {
      const states = parseValues(state.values || '')
      state.segments = states.map((s, i) => ({ id: `s${i}`, state: s }))
      state.ariaLabel = states.length === 0
        ? null
        : `${states.length} step outcome${states.length === 1 ? '' : 's'}`
    })

    return () => stop()
  },
})
