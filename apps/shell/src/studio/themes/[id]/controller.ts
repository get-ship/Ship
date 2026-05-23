// Theme editor — faithful to .artifacts/studio-design-v3/themes/detail.html.
//
// Design intent that the old port missed:
//  • The live preview is SCOPED — a container with data-theme={id} plus the
//    in-flight overrides as inline CSS vars. Editing never swaps the user's
//    global theme; "Set active" is the only thing that does.
//  • Tokens are grouped by category (tabs), each row typed (color → swatch +
//    picker, length/number/font → appropriate control), with an override dot.
import { controller, signal, derived, effect, onCleanup } from '@ship/sui-runtime'
import type { MutableState } from '@ship/sui-runtime'
import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type ThemeId,
} from '@ship/ui-components/tokens/themes'
import { LAYERS, ALL_TOKENS, type TokenType } from './token-catalog'
import type { Scope } from './page.sui'

// `on:` handlers receive the writable state proxy; annotate explicitly
// since the OnSection union blocks contextual inference. `overrides`
// emits as `unknown` (emitter gap) so narrow it through `SExt`.
type S = MutableState<Scope>
type SExt = S & { overrides: Record<string, string> }

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

export default controller<Scope>({
  scope: ({ route }) => {
    const themeId = derived<ThemeId>(() => {
      const id = route.params.id()
      return isValidTheme(id) ? id : readStoredTheme()
    })
    const theme = derived(() => THEMES.find((t) => t.id === themeId()) ?? null)
    const activeCat = signal('all')
    // Base token values for this theme, read from a scoped probe at mount.
    const base = signal<Record<string, string>>({})
    // In-flight edits, layered over the base.
    const overrides = signal<Record<string, string>>({})
    const committed = signal(false)

    // themes.css scopes tokens to :root[data-theme], so a theme's values only
    // resolve when applied to <html>. Apply it (preview), read its tokens, and
    // restore the user's theme on leave unless they Set active. Edits live in
    // `overrides` and only paint the preview pane, so the rest of the app keeps
    // the unedited theme while you work.
    const previousActive = readStoredTheme()
    effect(() => {
      const id = themeId()
      if (typeof document === 'undefined') return
      applyTheme(id)
      const cs = getComputedStyle(document.documentElement)
      const map: Record<string, string> = {}
      for (const t of ALL_TOKENS) map[t.name] = cs.getPropertyValue(t.name).trim()
      base.set(map)
    })
    onCleanup(() => {
      if (!committed()) applyTheme(previousActive)
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
    const isActive = derived(() => committed() || readStoredTheme() === themeId())
    const swatches = derived<string[]>(() => (theme()?.swatches ?? []) as string[])
    const kindLabel = derived(() => (theme()?.dark ? 'dark' : 'light'))
    const weights = derived(() => WEIGHTS)

    return {
      // base/overrides/committed must be exposed so mount() can seed base +
      // track commit, and the handlers can mutate via the scope proxy.
      base, overrides, committed,
      themeId, theme, activeCat, categories, groups, resolved,
      previewStyle, overrideCount, isActive, swatches, kindLabel, weights,
    }
  },

  on: {
    setCat: (s: S, id: string) => { s.activeCat = id },
    setToken: (s: S, name: string, value: string) => {
      const st = s as SExt
      st.overrides = { ...st.overrides, [name]: value }
    },
    resetAll: (s: S) => { (s as SExt).overrides = {} },
    setActive: (s: S) => {
      applyTheme(s.themeId as ThemeId)
      persistTheme(s.themeId as ThemeId)
      s.committed = true
    },
  },

})
