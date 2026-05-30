import { controller, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-sparkline.sui'
import { linePath, parseNums, toneColor } from '../chart-util'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const hasData = signal(false)
    const strokePath = signal('')
    const areaPath = signal('')
    const lineColor = signal('var(--primary)')
    return { hasData, strokePath, areaPath, lineColor }
  },

  mount(state, _host) {
    const stop = effect(() => {
      const nums = parseNums(state.data)
      const w = state.width
      const h = state.height
      const color = toneColor(state.tone)
      state.lineColor = color

      if (nums.length < 2) {
        state.hasData = false
        state.strokePath = ''
        state.areaPath = ''
        return
      }

      const { d, points } = linePath(nums, w, h, 2)
      state.strokePath = d

      if (state.fill) {
        const first = points[0]!
        const last = points[points.length - 1]!
        state.areaPath = `${d} L${last[0].toFixed(2)},${h} L${first[0].toFixed(2)},${h} Z`
      } else {
        state.areaPath = ''
      }
      state.hasData = true
    })

    return () => stop()
  },
})
