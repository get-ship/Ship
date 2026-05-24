// Themes studio section — the theme gallery + picker. Themes are a client-side
// catalog (the THEMES registry in tokens/themes.ts); applying one flips
// `data-theme` on <html> + persists to localStorage, the same path the nav
// theme-picker uses. The design-system designer owns the theme *values*; this
// view is the pick/preview UX.
import { controller, signal, onCleanup } from '@ship/sui-runtime'
import {
  THEMES,
  readStoredTheme,
  type ThemeId,
} from '@ship/ui-components/tokens/themes'
import type { Scope } from './page.sui'

type FilterId = 'all' | 'light' | 'dark'

// `navigate` is injected onto the proxy by the runtime and isn't a scope key.
type Nav = { navigate: (path: string) => void }

export default controller<Scope>()({
  scope: () => {
    const active = signal<ThemeId>(readStoredTheme())
    const q = signal('')
    const filter = signal<FilterId>('all')
    // Stay in sync when the theme is changed elsewhere (nav picker, etc.).
    // onCleanup enrolls teardown on the mount scope — no effect wrapper needed
    // since the listener doesn't depend on reactive state.
    if (typeof window !== 'undefined') {
      const onChange = (e: Event) => {
        const id = (e as CustomEvent<{ id?: string }>).detail?.id
        if (id) active.set(id as ThemeId)
      }
      window.addEventListener('ship:theme:change', onChange)
      onCleanup(() => window.removeEventListener('ship:theme:change', onChange))
    }
    return { active, q, filter }
  },
  derived: {
    themes: ({ q, filter, active }) => {
      // Derived scope-args arrive as signal getters — call them.
      const term = ((typeof q === 'function' ? q() : q) as string).toLowerCase()
      const f = (typeof filter === 'function' ? filter() : filter) as FilterId
      const cur = (typeof active === 'function' ? active() : active) as ThemeId
      return THEMES.filter((t) => f === 'all' || (f === 'dark' ? t.dark : !t.dark))
        .filter(
          (t) =>
            !term ||
            t.label.toLowerCase().includes(term) ||
            t.id.toLowerCase().includes(term),
        )
        .map((t) => ({
          id: t.id,
          label: t.label,
          dark: t.dark,
          swatches: t.swatches,
          isActive: t.id === cur,
        }))
    },
    filters: () => [
      { id: 'all', label: 'All', count: THEMES.length },
      { id: 'light', label: 'Light', count: THEMES.filter((t) => !t.dark).length },
      { id: 'dark', label: 'Dark', count: THEMES.filter((t) => t.dark).length },
    ],
    activeLabel: ({ active }) => {
      const cur = (typeof active === 'function' ? active() : active) as ThemeId
      return THEMES.find((t) => t.id === cur)?.label ?? cur
    },
  },
  on: {
    // Clicking a theme opens its editor — it does NOT swap your live theme
    // (browsing shouldn't lose your active theme). Applying is an explicit
    // "Set active" action inside the editor.
    open: (s, id: string) => {
      (s as unknown as Nav).navigate(`/studio/themes/${id}`)
    },
    onSearchInput: (s, e: Event) => {
      s.q = (e.target as HTMLInputElement).value ?? ''
    },
    clearSearch: (s) => {
      s.q = ''
    },
    setFilter: (s, id: string) => {
      s.filter = id as FilterId
    },
  },
})
