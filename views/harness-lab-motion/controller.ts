import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Tone = 'primary' | 'success' | 'warning' | 'info' | 'accent'
interface Chip { id: number; label: string; tone: Tone }

const TONES: Tone[] = ['primary', 'success', 'warning', 'info', 'accent']
const WORDS = ['fade', 'slide', 'scale', 'morph', 'spring', 'flip', 'stagger', 'ease']
let nextChip = 0
let nextGrid = 0

const mkChip = (): Chip => ({
  id: ++nextChip,
  label: WORDS[nextChip % WORDS.length] + ' ' + nextChip,
  tone: TONES[nextChip % TONES.length],
})

export default controller<Scope>({
  scope: () => {
    const chips = signal<Chip[]>(Array.from({ length: 5 }, mkChip))
    const grid = signal<number[]>(Array.from({ length: 200 }, (_, i) => i + 1))
    const thrashing = signal(false)
    const iter = signal(0)
    const currentTab = signal<'a' | 'b' | 'c'>('a')
    const prefersReduced = signal(
      typeof window !== 'undefined' &&
      window.matchMedia &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches,
    )
    let timer: ReturnType<typeof setInterval> | null = null

    return {
      chips, grid, thrashing, iter, currentTab, prefersReduced,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      addOne: (): void => { chips.set([...chips(), mkChip()]) },
      addTen: (): void => { chips.set([...chips(), ...Array.from({ length: 10 }, mkChip)]) },
      removeAll: (): void => { chips.set([]) },
      removeChip: (id: number): void => { chips.set(chips().filter((c) => c.id !== id)) },
      shuffle: (): void => {
        const arr = [...chips()]
        for (let i = arr.length - 1; i > 0; i--) {
          const j = Math.floor(Math.random() * (i + 1));
          [arr[i], arr[j]] = [arr[j], arr[i]]
        }
        chips.set(arr)
      },

      thrashOnce: (): void => {
        nextGrid += 200
        grid.set(Array.from({ length: 200 }, (_, i) => nextGrid + i))
        iter.set(iter() + 1)
      },
      toggleThrash: (): void => {
        if (thrashing()) {
          thrashing.set(false)
          if (timer) { clearInterval(timer); timer = null }
        } else {
          thrashing.set(true)
          timer = setInterval(() => {
            nextGrid += 200
            grid.set(Array.from({ length: 200 }, (_, i) => nextGrid + i))
            iter.set(iter() + 1)
          }, 16)
        }
      },

      setTab: (t: 'a' | 'b' | 'c'): void => currentTab.set(t),
    }
  },

  derived: {
    tabs: () => [
      { id: 'a', label: 'Alpha' },
      { id: 'b', label: 'Beta' },
      { id: 'c', label: 'Gamma' },
    ],
  },
})
