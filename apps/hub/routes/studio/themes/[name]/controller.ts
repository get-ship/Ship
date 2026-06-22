// Theme detail view — `/studio/themes/:name`.
//
// The theme catalog is the same `THEMES` registry the index view reads
// (`@ship/tokens/themes`); this view is the preview + apply editor for
// one entry. Applying does NOT navigate — it just flips `data-theme`
// on <html> and persists via the host's theme transport (wired in
// apps/hub/src/lib/client.ts), the same path the index view and the
// nav theme-picker use.
//
// Note: a `route.params.name()` reactive handle is provided by
// `@ship/sui-runtime` via `ctx.route`. Reading it the first time
// creates the signal and seeds it from the current pathname — no
// separate `defineRoutes` call needed (the param route is registered
// by `deck routes` codegen at build time).

import { controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  THEMES,
  DEFAULT_THEME,
  applyTheme,
  isValidTheme,
  readStoredTheme,
  type Theme,
  type ThemeId,
} from '@ship/tokens/themes'
import type { Scope } from './page.sui'

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
)

type Banner = { ok: boolean; msg: string } | null

const SWATCH_LABELS: ReadonlyArray<readonly [string, string]> = [
  ['background', 'Background'],
  ['card', 'Card'],
  ['primary', 'Primary'],
  ['accent', 'Accent'],
]

export default controller({
  scope: ({ route }: ViewScope<Scope>) => {
    const themeName = derived(() => route.params.name() ?? '')
    // Active theme is reactive — the index view mutates this signal
    // when the user picks a different theme there, and we want this
    // page's "is active" chip to follow live. The host's theme
    // transport (wired in apps/hub/src/lib/client.ts) dispatches
    // `ship:theme:change` after every apply, so a single listener is
    // enough to stay in sync.
    const activeId = signal<ThemeId>(readStoredTheme())
    if (typeof window !== 'undefined') {
      const onChange = (e: Event) => {
        const id = (e as CustomEvent<{ id?: string }>).detail?.id
        if (id) activeId.set(id as ThemeId)
      }
      window.addEventListener('ship:theme:change', onChange)
    }
    const theme = derived<Theme | null>(() => {
      const id = themeName()
      return id ? (THEME_BY_ID[id] ?? null) : null
    })
    const isActive = derived(() => theme()?.id === activeId())
    const swatches = derived<{ label: string; value: string }[]>(() => {
      const t = theme()
      if (!t) return []
      return SWATCH_LABELS.map(([, label], i) => ({
        label,
        value: t.swatches[i] ?? t.swatches[0] ?? '',
      }))
    })
    const applyBanner = signal<Banner>(null)

    return { themeName, theme, isActive, swatches, activeId, applyBanner }
  },

  on: {
    setActive: (s) => {
      const t = s.theme
      if (!t) return
      if (typeof window === 'undefined') return
      if (!isValidTheme(t.id)) {
        s.applyBanner = { ok: false, msg: 'Unknown theme id.' }
        return
      }
      try {
        applyTheme(t.id)
        s.activeId = t.id
        s.applyBanner = { ok: true, msg: 'Theme applied.' }
      } catch (err) {
        s.applyBanner = { ok: false, msg: (err as Error).message ?? 'Apply failed.' }
      }
    },
    clearActive: (s) => {
      if (typeof window === 'undefined') return
      // The catalog has no "off" theme — the closest "clear" is
      // re-applying the registry default. This is the same fallback
      // the index view uses for first paint (`readStoredTheme` →
      // DEFAULT_THEME).
      try {
        applyTheme(DEFAULT_THEME)
        s.activeId = DEFAULT_THEME
        s.applyBanner = { ok: true, msg: 'Reset to default.' }
      } catch (err) {
        s.applyBanner = { ok: false, msg: (err as Error).message ?? 'Reset failed.' }
      }
    },
  },
})
