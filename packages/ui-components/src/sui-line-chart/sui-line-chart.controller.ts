import { controller, effect, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-line-chart.sui'
import { linePath, parseJson, toneColor } from '../chart-util'

interface Series {
  name: string
  data: number[]
  tone?: string
}

interface Pt {
  id: string
  cx: number
  cy: number
}

interface SeriesPath {
  name: string
  strokeD: string
  color: string
  pts: Pt[]
}

interface GridLine {
  id: string
  y: number
}

export default controller<Scope>({
  scope: () => {
    const seriesPaths = signal<SeriesPath[]>([])
    const gridLines = signal<GridLine[]>([])
    return { seriesPaths, gridLines }
  },

  mount(state, _host) {
    const scope = state as unknown as Scope
    const seriesPaths = scope.seriesPaths as unknown as Signal<SeriesPath[]>
    const gridLines = scope.gridLines as unknown as Signal<GridLine[]>

    const stop = effect(() => {
      const allSeries = parseJson<Series[]>(scope.series() as string, [])
      const w = scope.width() as number
      const h = scope.height() as number
      const flat = allSeries.flatMap((s) => s.data)
      const min = flat.length ? Math.min(...flat) : 0
      const max = flat.length ? Math.max(...flat) : 1
      const range = max - min || 1
      const innerH = h - 4

      gridLines.set(
        (scope.grid() as boolean)
          ? [0.25, 0.5, 0.75].map((f, i) => ({ id: `g${i}`, y: 2 + innerH * f }))
          : [],
      )

      const paths: SeriesPath[] = []
      for (const s of allSeries) {
        if (s.data.length < 2) continue
        const normalized = s.data.map((v) => ((v - min) / range) * 100)
        const { d, points } = linePath(normalized, w, h, 2)
        if (!d) continue
        const color = toneColor(s.tone)
        paths.push({
          name: s.name,
          strokeD: d,
          color,
          pts: points.map(([cx, cy], i) => ({ id: `${s.name}-${i}`, cx, cy })),
        })
      }
      seriesPaths.set(paths)
    })

    return () => stop()
  },
})
