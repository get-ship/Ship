// Theme editor — `/studio/themes/:name`.
//
// The catalog is the static `THEMES` registry (`@ship/tokens/themes`); this
// view is the full token editor + scoped preview for one entry. There are no
// Connect ops — every value is read from the design-token CSS at runtime.
//
//  • The live preview is SCOPED — a container with data-theme={id} plus the
//    in-flight overrides as inline CSS vars. Editing never bleeds your edits
//    onto the rest of the app; only the preview pane reflects them.
//  • themes.css scopes tokens to :root[data-theme], so reading a theme's real
//    values means applying it to <html>. The probe below does that on mount,
//    then onCleanup restores the persisted active theme — so browsing the
//    editor never quietly swaps the user's theme. "Set active" is the only
//    thing that commits a change.
//  • Tokens are grouped by category (tabs), each row typed (color → swatch +
//    picker, number → weight select, others → text input) with an override dot.
//
// `route.params.name()` is a reactive handle provided by `@ship/sui-runtime`
// via `ctx.route`; the param route is registered by `deck routes` codegen at
// build time. An unknown name resolves to no theme → the not-found surface.
import { controller, signal, derived, effect, onCleanup } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  THEMES,
  DEFAULT_THEME,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type Theme,
  type ThemeId,
} from '@ship/tokens/themes'
import { LAYERS, ALL_TOKENS, type TokenType } from './token-catalog'
import type { Scope } from './page.sui'

const THEME_BY_ID: Record<string, Theme> = Object.fromEntries(
  THEMES.map((t) => [t.id, t]),
)

const CATEGORIES = [{ id: 'all', label: 'All' }, ...LAYERS.map((l) => ({ id: l.id, label: l.label }))]
// Common font-weight options for the number control.
const WEIGHTS = ['300', '400', '500', '600', '700', '800']

interface Row {
  name: string
  type: TokenType
  value: string
  overridden: boolean
}
interface Group {
  id: string
  label: string
  tokens: Row[]
}

export default controller({
  scope: ({ route }: ViewScope<Scope>) => {
    const themeName = derived(() => route.params.name() ?? '')
    const themeId = derived<ThemeId | null>(() => {
      const id = themeName()
      return isValidTheme(id) ? id : null
    })
    const theme = derived<Theme | null>(() => {
      const id = themeId()
      return id ? (THEME_BY_ID[id] ?? null) : null
    })

    const activeCat = signal('all')
    // Base token values for this theme, read from the scoped probe at mount.
    const base = signal<Record<string, string>>({})
    // In-flight edits, layered over the base.
    const overrides = signal<Record<string, string>>({})
    // The persisted active theme. Seeded from storage; set active / clear
    // active update it. Drives the "Active" indicator and the onCleanup
    // restore, so the preview-time applyTheme below never sticks.
    const activeId = signal<ThemeId>(readStoredTheme())

    // Apply the theme to <html> (preview), read its resolved tokens into base,
    // and restore the persisted active theme on leave. Edits live in
    // `overrides` and only paint the preview pane, so the rest of the app keeps
    // the unedited theme while you work.
    effect(() => {
      const id = themeId()
      if (typeof document === 'undefined' || !id) return
      applyTheme(id)
      const cs = getComputedStyle(document.documentElement)
      const map: Record<string, string> = {}
      for (const t of ALL_TOKENS) map[t.name] = cs.getPropertyValue(t.name).trim()
      base.set(map)
    })
    onCleanup(() => {
      if (typeof document === 'undefined') return
      applyTheme(activeId())
    })

    const resolved = derived<Record<string, string>>(() => {
      const b = base() as Record<string, string>
      const o = overrides() as Record<string, string>
      const out: Record<string, string> = {}
      for (const t of ALL_TOKENS) out[t.name] = o[t.name] ?? b[t.name] ?? ''
      return out
    })

    const groups = derived<Group[]>(() => {
      const cat = activeCat() as string
      const r = resolved() as Record<string, string>
      const o = overrides() as Record<string, string>
      return LAYERS.filter((l) => cat === 'all' || l.id === cat).map((l) => ({
        id: l.id,
        label: l.label,
        tokens: l.tokens.map((t) => ({
          name: t.name,
          type: t.type,
          value: r[t.name] ?? '',
          overridden: t.name in o,
        })),
      }))
    })

    const categories = derived(() =>
      CATEGORIES.map((c) => ({ ...c, active: c.id === (activeCat() as string) })),
    )
    // Scoped preview: inline CSS vars for the in-flight overrides, layered on
    // the container's data-theme. Empty when nothing's overridden yet.
    const previewStyle = derived(() =>
      Object.entries(overrides() as Record<string, string>)
        .map(([k, v]) => `${k}: ${v}`)
        .join('; '),
    )
    const overrideCount = derived(() => Object.keys(overrides() as Record<string, string>).length)
    const isActive = derived(() => {
      const t = theme()
      return t !== null && t.id === activeId()
    })
    const swatches = derived<string[]>(() => (theme()?.swatches ?? []) as string[])
    const kindLabel = derived(() => (theme()?.dark ? 'dark' : 'light'))
    const weights = derived(() => WEIGHTS)

    return {
      // themeId / overrides / activeCat / activeId must be exposed: the
      // handlers mutate or read them through the scope proxy.
      themeName, themeId, theme, activeCat, categories, groups,
      previewStyle, overrideCount, isActive, swatches, kindLabel, weights,
      overrides, activeId,
    }
  },

  on: {
    setCat: (s, id: string) => { s.activeCat = id },
    setToken: (s, name: string, value: string) => {
      s.overrides = { ...s.overrides, [name]: value }
    },
    resetAll: (s) => { s.overrides = {} },
    setActive: (s) => {
      const id = s.themeId
      if (!id || typeof document === 'undefined') return
      applyTheme(id)
      persistTheme(id)
      s.activeId = id
    },
    // No "off" theme exists; clearing re-applies the registry default — the
    // same fallback first paint uses (readStoredTheme → DEFAULT_THEME).
    clearActive: (s) => {
      if (typeof document === 'undefined') return
      applyTheme(DEFAULT_THEME)
      persistTheme(DEFAULT_THEME)
      s.activeId = DEFAULT_THEME
    },
  },

})
