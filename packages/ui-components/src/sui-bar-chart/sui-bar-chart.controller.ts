import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-bar-chart.sui'
import { parseLabels, parseNums, toneColor } from '../chart-util'

interface Bar {
  id: string
  x: number
  y: number
  w: number
  h: number
  rx: number
  ry: number
  color: string
  title: string
}

export default controller<Scope>({
  scope: () => {
    const bars = signal<Bar[]>([])
    const isEmpty = signal(true)
    return { bars, isEmpty }
  },

  mount(state, _host) {
    const scope = state as unknown as Scope
    const bars = scope.bars as unknown as Signal<Bar[]>
    const isEmpty = scope.isEmpty as unknown as Signal<boolean>

    const stop = effect(() => {
      const nums = parseNums(scope.data() as string)
      if (nums.length === 0) {
        bars.set([])
        isEmpty.set(true)
        return
      }
      const labels = parseLabels(scope.labels() as string)
      const w = scope.width() as number
      const h = scope.height() as number
      const rawMax = scope.max() as number
      const maxVal = rawMax > 0 ? rawMax : Math.max(...nums, 0)
      const safeMax = maxVal <= 0 ? 1 : maxVal
      const color = toneColor(scope.tone() as string)
      const gap = 4
      const radius = 3
      const horizontal = (scope.orientation() as string) === 'horizontal'
      const n = nums.length

      bars.set(
        nums.map((v, i) => {
          const ratio = Math.max(0, v) / safeMax
          let x: number, y: number, barW: number, barH: number
          if (horizontal) {
            barH = (h - gap * (n - 1)) / n
            y = i * (barH + gap)
            barW = ratio * w
            x = 0
          } else {
            barW = (w - gap * (n - 1)) / n
            x = i * (barW + gap)
            barH = ratio * h
            y = h - barH
          }
          return {
            id: `b${i}`,
            x,
            y,
            w: Math.max(0, barW),
            h: Math.max(0, barH),
            rx: radius,
            ry: radius,
            color,
            title: `${labels[i] ?? String(v)}: ${v}`,
          }
        }),
      )
      isEmpty.set(false)
    })

    return () => stop()
  },
})
