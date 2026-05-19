import { controller, effect, signal } from '@ship/sui-runtime'
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
    const stop = effect(() => {
      const sz = state.size()
      const t = state.thickness()
      const r = (sz - t) / 2 - 2
      const cx = sz / 2
      const cy = sz / 2

      state.trackD.set(arcPath(cx, cy, r, START_ANGLE, END_ANGLE))

      const val = state.value()
      const min = state.min()
      const max = state.max()
      const ratio = Math.max(0, Math.min(1, (val - min) / (max - min || 1)))
      const valueEndAngle = START_ANGLE + (END_ANGLE - START_ANGLE) * ratio
      state.valueD.set(ratio > 0 ? arcPath(cx, cy, r, START_ANGLE, valueEndAngle) : '')

      const explicitTone = state.tone()
      const low = state.lowThreshold()
      const high = state.highThreshold()
      let resolved: string = explicitTone
      if (!resolved) {
        if (low > 0 && val < low) resolved = 'destructive'
        else if (high > 0 && val < high) resolved = 'warning'
        else if (low > 0 || high > 0) resolved = 'success'
        else resolved = 'primary'
      }
      state.arcColor.set(toneColor(resolved))
    })

    return () => stop()
  },
})
