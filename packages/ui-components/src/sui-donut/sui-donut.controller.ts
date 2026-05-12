import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-donut.sui'
import { arcPath, parseJson, toneColor } from '../chart-util'

interface Segment {
  id: string
  d: string
  color: string
  title: string
}

export default controller<Scope>({
  scope: () => ({
    segments: signal<Segment[]>([]),
  }),

  mount(state, _host) {
    const scope = state as unknown as Scope
    const segments = scope.segments as unknown as Signal<Segment[]>

    const stop = effect(() => {
      const raw = parseJson<Array<{ label: string; value: number; tone?: string }>>(
        scope.data() as string,
        [],
      )
      const total = raw.reduce((acc, s) => acc + Math.max(0, s.value), 0)
      if (total <= 0) {
        segments.set([])
        return
      }
      const sz = scope.size() as number
      const t = scope.thickness() as number
      const r = (sz - t) / 2
      const cx = sz / 2
      const cy = sz / 2
      const TWO_PI = 2 * Math.PI
      let angle = -Math.PI / 2

      segments.set(
        raw
          .filter((seg) => seg.value > 0)
          .map((seg, i) => {
            const rawSweep = (seg.value / total) * TWO_PI
            // Cap just below full circle to avoid coincident-endpoint arc degeneracy.
            const sweep = rawSweep >= TWO_PI - 0.001 ? TWO_PI - 0.001 : rawSweep
            const d = arcPath(cx, cy, r, angle, angle + sweep)
            angle += rawSweep
            return {
              id: `s${i}`,
              d,
              color: toneColor(seg.tone),
              title: `${seg.label}: ${seg.value}`,
            }
          }),
      )
    })

    return () => stop()
  },
})
