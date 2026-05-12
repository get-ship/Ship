import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-sparkline.sui'
import { linePath, parseNums, toneColor } from '../chart-util'

export default controller<Scope>({
  scope: () => {
    const hasData = signal(false)
    const strokePath = signal('')
    const areaPath = signal('')
    const lineColor = signal('var(--primary)')
    return { hasData, strokePath, areaPath, lineColor }
  },

  mount(state, _host) {
    const scope = state as unknown as Scope
    const hasData = scope.hasData as unknown as Signal<boolean>
    const strokePath = scope.strokePath as unknown as Signal<string>
    const areaPath = scope.areaPath as unknown as Signal<string>
    const lineColor = scope.lineColor as unknown as Signal<string>

    const stop = effect(() => {
      const nums = parseNums(scope.data() as string)
      const w = scope.width() as number
      const h = scope.height() as number
      const color = toneColor(scope.tone() as string)
      lineColor.set(color)

      if (nums.length < 2) {
        hasData.set(false)
        strokePath.set('')
        areaPath.set('')
        return
      }

      const { d, points } = linePath(nums, w, h, 2)
      strokePath.set(d)

      if (scope.fill() as boolean) {
        const first = points[0]!
        const last = points[points.length - 1]!
        areaPath.set(`${d} L${last[0].toFixed(2)},${h} L${first[0].toFixed(2)},${h} Z`)
      } else {
        areaPath.set('')
      }
      hasData.set(true)
    })

    return () => stop()
  },
})
