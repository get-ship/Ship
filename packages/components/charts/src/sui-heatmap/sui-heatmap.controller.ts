import { controller, effect, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-heatmap.sui'
import { parseJson, parseLabels, parseNums, toneColor } from '../chart-util'

interface Cell {
  id: string
  x: number
  y: number
  w: number
  h: number
  fill: string
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({
    cells: signal<Cell[]>([]),
    svgW: signal(0),
    svgH: signal(0),
  }),

  mount(state, _host) {
    const stop = effect(() => {
      const rowLabels = parseLabels(state.rows)
      const colLabels = parseLabels(state.cols)
      const cs = state.cellSize
      const gp = state.gap
      const rawData = state.data

      // Resolve grid: prefer JSON 2D array, fall back to 1D comma-separated numbers.
      const jsonResult = parseJson<number[][]>(rawData, [])
      const is2d =
        Array.isArray(jsonResult) &&
        jsonResult.length > 0 &&
        Array.isArray(jsonResult[0])
      let grid: number[][]
      if (is2d) {
        grid = jsonResult
      } else {
        const flat = parseNums(rawData)
        const nc = colLabels.length || 1
        const nr = rowLabels.length || (flat.length > 0 ? Math.ceil(flat.length / nc) : 0)
        grid = Array.from({ length: nr }, (_, r) => flat.slice(r * nc, (r + 1) * nc))
      }

      const nrows = grid.length
      const ncols = nrows > 0 ? Math.max(...grid.map((r) => r.length)) : 0

      // Normalize values across all cells for color mixing.
      const allVals = grid.flatMap((r) => r.filter((v) => typeof v === 'number'))
      const vMin = allVals.length ? Math.min(...allVals) : 0
      const vMax = allVals.length ? Math.max(...allVals) : 1
      const vRange = vMax - vMin

      const toneVar = toneColor(state.tone)
      const step = cs + gp

      state.svgW = ncols > 0 ? ncols * cs + (ncols - 1) * gp : 0
      state.svgH = nrows > 0 ? nrows * cs + (nrows - 1) * gp : 0

      const result: Cell[] = []
      for (let ri = 0; ri < nrows; ri++) {
        const row = grid[ri] ?? []
        for (let ci = 0; ci < ncols; ci++) {
          const v = row[ci] ?? 0
          // When all values are equal, show full saturation if non-zero, none if zero.
          const normalized = vRange === 0 ? (vMin > 0 ? 1 : 0) : (v - vMin) / vRange
          const pct = (normalized * 100).toFixed(1)
          const fill =
            normalized > 0
              ? `fill:color-mix(in oklch,${toneVar} ${pct}%,transparent)`
              : ''
          result.push({
            id: `c${ri}-${ci}`,
            x: ci * step,
            y: ri * step,
            w: cs,
            h: cs,
            fill,
          })
        }
      }
      state.cells = result
    })

    return () => stop()
  },
})
