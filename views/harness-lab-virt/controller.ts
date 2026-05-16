import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Tone = 'primary' | 'success' | 'warning' | 'info' | 'accent'
type Density = 'compact' | 'normal' | 'roomy'

const TONES: Tone[] = ['primary', 'success', 'warning', 'info', 'accent']
const ROW_H_BY_DENSITY: Record<Density, number> = { compact: 28, normal: 44, roomy: 64 }
const BUFFER = 8

interface Row { id: number; tone: Tone; seed: number }

const seed = (n: number): Row[] =>
  Array.from({ length: n }, (_, i) => ({
    id: i + 1,
    tone: TONES[i % TONES.length],
    seed: ((i * 9301 + 49297) % 233280) / 233280,
  }))

export default controller<Scope>({
  scope: () => {
    const all = signal<Row[]>(seed(10000))
    const scrollTop = signal(0)
    const viewportH = signal(640)
    const density = signal<Density>('normal')
    const mutating = signal(false)
    const mutationCount = signal(0)
    let timer: ReturnType<typeof setInterval> | null = null
    let scrollEl: HTMLElement | null = null

    const rowH = derived(() => ROW_H_BY_DENSITY[density()])
    const startIdx = derived(() => Math.max(0, Math.floor(scrollTop() / rowH()) - BUFFER))
    const visibleCount = derived(() => Math.ceil(viewportH() / rowH()) + BUFFER * 2)
    const endIdx = derived(() => Math.min(all().length, startIdx() + visibleCount()))

    const windowed = derived(() => {
      const list = all()
      const s = startIdx()
      const e = endIdx()
      return list.slice(s, e).map((r) => ({
        ...r,
        label: 'row-' + r.id.toString().padStart(6, '0'),
        metric: (r.seed * 1000).toFixed(1),
        barPct: Math.round(r.seed * 100),
      }))
    })

    return {
      all, scrollTop, viewportH, density, mutating, mutationCount,
      rowH, startIdx, visibleCount, endIdx, windowed,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      setCount: (n: number): void => {
        all.set(seed(n))
        if (scrollEl) scrollEl.scrollTop = 0
      },
      setDensity: (d: Density): void => density.set(d),
      jump: (frac: number): void => {
        if (!scrollEl) return
        scrollEl.scrollTop = Math.floor(scrollEl.scrollHeight * frac)
      },
      toggleMutate: (): void => {
        if (mutating()) {
          mutating.set(false)
          if (timer) { clearInterval(timer); timer = null }
        } else {
          mutating.set(true)
          mutationCount.set(0)
          timer = setInterval(() => {
            const list = all()
            const i = Math.floor(Math.random() * list.length)
            const next = list.slice()
            next[i] = { ...next[i], seed: Math.random() }
            all.set(next)
            mutationCount.set(mutationCount() + 1)
          }, 16)
        }
      },

      bindScrollEl: (el: HTMLElement): void => { scrollEl = el },
    }
  },

  derived: {
    total:      ({ all }) => all().length,
    windowSize: ({ visibleCount }) => visibleCount(),
    topPad:     ({ startIdx, rowH }) => startIdx() * rowH(),
    bottomPad:  ({ all, endIdx, rowH }) => Math.max(0, (all().length - endIdx()) * rowH()),
  },

  mount: (state, el) => {
    const sc = el.querySelector('.hv-scroll') as HTMLElement | null
    if (!sc) return
    type ScopeState = {
      bindScrollEl: (el: HTMLElement) => void
      scrollTop: number
      viewportH: number
    }
    const s = state as unknown as ScopeState
    s.bindScrollEl(sc)
    s.viewportH = sc.clientHeight
    let raf = 0
    const onScroll = (): void => {
      if (raf) return
      raf = requestAnimationFrame(() => {
        raf = 0
        s.scrollTop = sc.scrollTop
      })
    }
    sc.addEventListener('scroll', onScroll, { passive: true })
    const ro = new ResizeObserver(() => { s.viewportH = sc.clientHeight })
    ro.observe(sc)
    return () => {
      sc.removeEventListener('scroll', onScroll)
      if (raf) cancelAnimationFrame(raf)
      ro.disconnect()
    }
  },
})
