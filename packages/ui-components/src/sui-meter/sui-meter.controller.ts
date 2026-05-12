import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-meter.sui'
import { parseJson, toneColor } from '../chart-util'

interface Bar {
  id: string
  style: string
  title: string
}

interface SegInput {
  tone: string
  count: number
  label?: string
}

export default controller<Scope>({
  scope: () => ({
    bars: signal<Bar[]>([]),
  }),

  mount(state, _host) {
    const scope = state as unknown as Scope
    const bars = scope.bars as unknown as Signal<Bar[]>

    const stop = effect(() => {
      const segs = parseJson<SegInput[]>(scope.segments() as string, [])
      const sum = segs.reduce((a, s) => a + Math.max(0, s.count), 0)
      const rawTotal = scope.total() as number
      const denom = rawTotal > 0 ? rawTotal : sum
      const safeDenom = denom <= 0 ? 1 : denom

      bars.set(
        segs
          .filter((s) => s.count > 0)
          .map((s, i) => {
            const pct = (Math.max(0, s.count) / safeDenom) * 100
            return {
              id: `b${i}`,
              style: `width:${pct.toFixed(3)}%;background:${toneColor(s.tone)};`,
              title: `${s.label ?? s.tone}: ${s.count}`,
            }
          }),
      )
    })

    return () => stop()
  },
})
