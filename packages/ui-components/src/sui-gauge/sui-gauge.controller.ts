import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-gauge.sui'
import { arcPath, toneColor } from '../chart-util'

const DEG = Math.PI / 180
// 270° arc: start at 135°, end at 405° (= 45° + 360°, clockwise past bottom gap).
const START_ANGLE = 135 * DEG
const END_ANGLE = 405 * DEG

export default controller<Scope>({
  scope: () => ({
    trackD: signal(''),
    valueD: signal(''),
    arcColor: signal('var(--primary)'),
  }),

  mount(state, _host) {
    const scope = state as unknown as Scope
    const trackD = scope.trackD as unknown as Signal<string>
    const valueD = scope.valueD as unknown as Signal<string>
    const arcColor = scope.arcColor as unknown as Signal<string>

    const stop = effect(() => {
      const sz = scope.size() as number
      const t = scope.thickness() as number
      const r = (sz - t) / 2 - 2
      const cx = sz / 2
      const cy = sz / 2

      trackD.set(arcPath(cx, cy, r, START_ANGLE, END_ANGLE))

      const val = scope.value() as number
      const min = scope.min() as number
      const max = scope.max() as number
      const ratio = Math.max(0, Math.min(1, (val - min) / (max - min || 1)))
      const valueEndAngle = START_ANGLE + (END_ANGLE - START_ANGLE) * ratio
      valueD.set(ratio > 0 ? arcPath(cx, cy, r, START_ANGLE, valueEndAngle) : '')

      const explicitTone = scope.tone() as string
      const low = scope.lowThreshold() as number
      const high = scope.highThreshold() as number
      let resolved = explicitTone
      if (!resolved) {
        if (low > 0 && val < low) resolved = 'destructive'
        else if (high > 0 && val < high) resolved = 'warning'
        else if (low > 0 || high > 0) resolved = 'success'
        else resolved = 'primary'
      }
      arcColor.set(toneColor(resolved))
    })

    return () => stop()
  },
})
