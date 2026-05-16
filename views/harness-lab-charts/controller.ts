import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const WINDOW = 30

export default controller<Scope>({
  scope: () => {
    const points = signal<number[]>(Array.from({ length: WINDOW }, (_, i) => 50 + Math.sin(i / 3) * 30))
    const points2 = signal<number[]>(Array.from({ length: WINDOW }, (_, i) => 40 + Math.cos(i / 2) * 20))
    const tick = signal(0)
    const hz = signal(10)
    const streaming = signal(false)
    let timer: ReturnType<typeof setInterval> | null = null

    const startTimer = (): void => {
      if (timer) clearInterval(timer)
      timer = setInterval(() => {
        const cur = points()
        const cur2 = points2()
        const t = tick() + 1
        tick.set(t)
        const next = [...cur.slice(1), Math.max(0, Math.min(100, cur[cur.length - 1] + (Math.random() - 0.5) * 12))]
        const next2 = [...cur2.slice(1), Math.max(0, Math.min(100, cur2[cur2.length - 1] + (Math.random() - 0.5) * 18))]
        points.set(next)
        points2.set(next2)
      }, Math.max(16, Math.floor(1000 / hz())))
    }

    return {
      points, points2, tick, hz, streaming,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      toggle: (): void => {
        if (streaming()) {
          streaming.set(false)
          if (timer) { clearInterval(timer); timer = null }
        } else {
          streaming.set(true)
          startTimer()
        }
      },
      setHz: (n: number): void => {
        hz.set(n)
        if (streaming()) startTimer()
      },
      reset: (): void => {
        points.set(Array.from({ length: WINDOW }, () => 50))
        points2.set(Array.from({ length: WINDOW }, () => 50))
        tick.set(0)
      },
    }
  },

  derived: {
    csv:        ({ points }) => points().map((n) => n.toFixed(0)).join(','),
    csvWindow:  ({ points }) => points().slice(-12).map((n) => n.toFixed(0)).join(','),
    count:      ({ points }) => points().length,
    lastVal:    ({ points }) => Math.round(points()[points().length - 1]),
    seriesJson: ({ points, points2 }) =>
      JSON.stringify([{ data: points() }, { data: points2() }]),
  },
})
