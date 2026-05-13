import { controller, derived } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-theme-toggle.sui'

const STORAGE_KEY = 'ship-theme'

type Theme = 'light' | 'dark' | 'auto'

function resolveInitial(current: Theme): 'light' | 'dark' {
  if (current === 'light' || current === 'dark') return current
  if (typeof window === 'undefined') return 'light'
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'light' || saved === 'dark') return saved
  } catch {
    /* storage may be unavailable */
  }
  const prefersDark = window.matchMedia?.('(prefers-color-scheme: dark)')?.matches
  return prefersDark ? 'dark' : 'light'
}

function applyTheme(theme: 'light' | 'dark'): void {
  if (typeof document === 'undefined') return
  document.documentElement.classList.toggle('dark', theme === 'dark')
}

export default controller<Scope>({
  scope: (state, host) => {
    const s = state as unknown as {
      theme: Signal<Theme>
    }
    const isDark = derived<boolean>(() => s.theme() === 'dark')

    function toggle(): void {
      const next: 'light' | 'dark' = s.theme() === 'dark' ? 'light' : 'dark'
      s.theme.set(next)
      try {
        localStorage.setItem(STORAGE_KEY, next)
      } catch {
        /* quota / private mode */
      }
      applyTheme(next)
      host.dispatchEvent(
        new CustomEvent('sui-theme-change', {
          detail: { theme: next },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { isDark, toggle }
  },

  mount(state) {
    const scope = state as unknown as {
      theme: Signal<Theme>
    }
    const resolved = resolveInitial(scope.theme())
    if (resolved !== scope.theme()) scope.theme.set(resolved)
    applyTheme(resolved)
  },
})
