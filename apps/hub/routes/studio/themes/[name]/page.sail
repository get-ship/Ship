<controller>
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
// `route.params.name()` is a reactive handle provided by `@ship/sail-runtime`
// via `ctx.route`; the param route is registered by `deck routes` codegen at
// build time. An unknown name resolves to no theme → the not-found surface.
import { controller, signal, derived, effect, onCleanup } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
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
import type { Scope } from './page.sail'

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

export default controller(({ route }: ViewScope<Scope>) => {

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

    
    const setCat = (id: string) => { activeCat = id }
    const setToken = (name: string, value: string) => {
      overrides = { ...overrides, [name]: value }
    }
    const resetAll = (s) => { s.overrides = {} }
    const setActive = (s) => {
      const id = s.themeId
      if (!id || typeof document === 'undefined') return
      applyTheme(id)
      persistTheme(id)
      s.activeId = id
    }
    return {
      // themeId / overrides / activeCat / activeId must be exposed: the
      // handlers mutate or read them through the scope proxy.
      themeName, themeId, theme, activeCat, categories, groups,
      previewStyle, overrideCount, isActive, swatches, kindLabel, weights,
      overrides, activeId, setCat, setToken, resetAll, setActive }
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <article data-test="theme-editor">

      <a href="/studio/themes" data-test="theme-detail-back">
        <icon name="chevron-left" size="xs"></icon>
        Themes
      </a>

      <if cond={!theme()}>
        <empty-state icon="search-x" variant="default" size="md" data-test="theme-not-found">
          <span slot="title">No such theme</span>
          <span>{'No theme with id "' + themeName() + '" was found.'}</span>
          <button slot="action" variant="outline" size="sm" href="/studio/themes">Back to themes</button>
        </empty-state>
      <else>

      <header>
        <span aria-hidden="true">
          <for as="c" of={swatches()} key={c}>
            <span style={'background:' + c}></span>
          </for>
        </span>
        <div>
          <h1 data-test="theme-name">{theme().label}</h1>
          <div>
            <code>{theme().id}</code>
            <span aria-hidden="true"></span>
            <span data-kind={kindLabel()}>{kindLabel()}</span>
            <span aria-hidden="true"></span>
            <span>{overrideCount()} token override{overrideCount() === 1 ? '' : 's'}</span>
          </div>
          <div aria-hidden="true"></div>
        </div>
        <div>
          <if cond={isActive()}>
            <button variant="outline" size="sm" disabled={true} data-test="theme-active">
              <icon slot="leading" name="check" size="xs"></icon>Active
            </button>
            <button variant="ghost" size="sm" on:click={clearActive} data-test="theme-clear">Clear active</button>
          <else>
            <button tone="primary" variant="solid" size="sm" on:click={setActive} data-test="set-active">
              <icon slot="leading" name="check" size="xs"></icon>Set active
            </button>
          </else></if>
        </div>
      </header>

      <div>

        <!-- Token editor -->
        <section>
          <div>
            <span>Tokens</span>
            <span>· {overrideCount()} overridden</span>
            <if cond={overrideCount() > 0}>
              <span><button type="button" on:click={resetAll}>Reset to defaults</button></span>
            </if>
          </div>
          <div role="tablist">
            <for as="c" of={categories()} key={c.id}>
              <button type="button" data-active={c.active}
                on:click={() => setCat(c.id)}>{c.label}</button>
            </for>
          </div>
          <div>
            <for as="g" of={groups()} key={g.id}>
              <div>{g.label}</div>
              <for as="t" of={g.tokens} key={t.name}>
                <div data-type={t.type}>
                  <span>
                    <code>{t.name}</code>
                    <if cond={t.overridden}><span title="overridden">●</span></if>
                  </span>
                  <if cond={t.type === 'color'}>
                    <label style={'background:' + t.value}>
                      <input type="color" value={t.value} aria-label={t.name}
                        on:input={(e) => setToken(t.name, e.target.value)} />
                    </label>
                    <input value={t.value} aria-label={t.name}
                      on:input={(e) => setToken(t.name, e.target.value)} />
                  <else-if cond={t.type === 'number'}>
                    <span aria-hidden="true"></span>
                    <select aria-label={t.name}
                      on:change={(e) => setToken(t.name, e.target.value)}>
                      <for as="w" of={weights()} key={w}>
                        <option value={w} selected={t.value === w}>{w}</option>
                      </for>
                    </select>
                  </else-if><else>
                    <span aria-hidden="true"></span>
                    <input value={t.value} aria-label={t.name}
                      on:input={(e) => setToken(t.name, e.target.value)} />
                  </else></if>
                </div>
              </for>
            </for>
          </div>
        </section>

        <!-- Live preview (scoped — never your active theme) -->
        <section>
          <div><span>Live preview</span><span>· updates as you type</span></div>
          <div data-theme={theme().id} style={previewStyle()}>
            <h3>Heading sets the tone</h3>
            <div>
              <button>Primary</button>
              <button>Outline</button>
              <button>Ghost</button>
            </div>
            <div>
              <strong>Card sits on the surface</strong>
              <p>Body text reads against the foreground token; muted text uses muted-foreground.</p>
            </div>
            <div>Search agents, skills, MCPs…</div>
            <nav>
              <a>Dashboard</a><a>Agents</a><a>Pipelines</a><a>Settings</a>
            </nav>
            <div>
              <div><div>8</div><div>agents</div></div>
              <div><div>14</div><div>skills</div></div>
              <div><div>5</div><div>MCP</div></div>
            </div>
            <div>
              <div>Success — token saved</div>
              <div>Error — invalid value</div>
            </div>
          </div>
        </section>

      </div>

      <section>
        <h2>Source</h2>
        <div><code>[data-theme="{theme().id}"]</code> in the global stylesheet</div>
      </section>

      </else></if>

    </article>
  </main>
</div>
