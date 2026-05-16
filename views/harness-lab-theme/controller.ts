import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface Swatch { name: string; color: string }

const SWATCHES: readonly Swatch[] = [
  { name: 'sky',     color: 'oklch(0.62 0.18 240)' },
  { name: 'emerald', color: 'oklch(0.66 0.16 152)' },
  { name: 'amber',   color: 'oklch(0.78 0.16 80)'  },
  { name: 'rose',    color: 'oklch(0.65 0.21 18)'  },
  { name: 'violet',  color: 'oklch(0.58 0.22 295)' },
]

export default controller<Scope>({
  scope: () => {
    const primary = signal('')
    const radius = signal(8)
    const demo = signal('')

    return {
      primary, radius, demo,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },

      setPrimary: (color: string): void => {
        primary.set(color)
        chrome.theme.set('--primary', color)
      },
      setRadius: (px: number): void => {
        radius.set(px)
        chrome.theme.set('--radius-md', `${px}px`)
      },
      clearAll: (): void => {
        primary.set('')
        radius.set(8)
        chrome.theme.clear('--primary')
        chrome.theme.clear('--radius-md')
      },
    }
  },

  derived: {
    swatches: () => SWATCHES,
  },
})
