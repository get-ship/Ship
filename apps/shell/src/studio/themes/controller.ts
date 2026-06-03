// Themes studio section — the theme gallery + picker. Themes are a client-side
// catalog (the THEMES registry in tokens/themes.ts); applying one flips
// `data-theme` on <html> + persists to localStorage, the same path the nav
// theme-picker uses. The design-system designer owns the theme *values*; this
// view is the pick/preview UX.
import { controller, signal, onCleanup, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  THEMES,
  readStoredTheme,
  type ThemeId,
} from '@ship/tokens/themes'
import type { Scope } from './page.sui'

type FilterId = 'all' | 'light' | 'dark'

// `navigate` is injected onto the proxy by the runtime and isn't a scope key.

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
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
    const themes = derived(() => {
      const term = q().toLowerCase()
      const f = filter()
      const cur = active()
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
    })
    const filters = derived(() => [
      { id: 'all', label: 'All', count: THEMES.length },
      { id: 'light', label: 'Light', count: THEMES.filter((t) => !t.dark).length },
      { id: 'dark', label: 'Dark', count: THEMES.filter((t) => t.dark).length },
    ])
    const activeLabel = derived(() => THEMES.find((t) => t.id === active())?.label ?? active())
    return { active, q, filter, themes, filters, activeLabel }
  },
  on: {
    // Clicking a theme opens its editor — it does NOT swap your live theme
    // (browsing shouldn't lose your active theme). Applying is an explicit
    // "Set active" action inside the editor.
    open: (s, id: string) => {
      s.navigate(`/studio/themes/${id}`)
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
