import { controller, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
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

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const bars = signal<Bar[]>([])
    const isEmpty = signal(true)
    return { bars, isEmpty }
  },

  mount(state, _host) {
    const stop = effect(() => {
      const nums = parseNums(state.data)
      if (nums.length === 0) {
        state.bars = []
        state.isEmpty = true
        return
      }
      const labels = parseLabels(state.labels)
      const w = state.width
      const h = state.height
      const rawMax = state.max
      const maxVal = rawMax > 0 ? rawMax : Math.max(...nums, 0)
      const safeMax = maxVal <= 0 ? 1 : maxVal
      const color = toneColor(state.tone)
      const gap = 4
      const radius = 3
      const horizontal = state.orientation === 'horizontal'
      const n = nums.length

      state.bars = nums.map((v, i) => {
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
        })
      state.isEmpty = false
    })

    return () => stop()
  },
})
