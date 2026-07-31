<controller>
// Themes studio section — the theme gallery + picker. Themes are a client-side
// catalog (the THEMES registry in tokens/themes.ts); applying one flips
// `data-theme` on <html> + persists to localStorage, the same path the nav
// theme-picker uses. The design-system designer owns the theme *values*; this
// view is the pick/preview UX.
//
// Hub note: this controller is byte-identical to the shell's. Themes uses NO
// Connect ops — only @ship/tokens/themes. The setThemeTransport wiring
// (save/load via ops.settings RUNTIME scope) is in apps/hub/src/lib/client.ts,
// the hub's bootstrap seam, mirroring apps/shell/src/ops.ts.
import { controller, signal, onCleanup, derived } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import {
  THEMES,
  readStoredTheme,
  type ThemeId,
} from '@ship/tokens/themes'
import type { Scope } from './page.sail'

type FilterId = 'all' | 'light' | 'dark'

// `navigate` is injected onto the proxy by the runtime and isn't a scope key.

export default controller((_ctx: ViewScope<Scope>) => {

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
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <div>
      <div>

        <header>
          <div>
            <h1>Themes</h1>
            <p>
              <strong>{themes().length}</strong> theme{themes().length === 1 ? '' : 's'}
              <span aria-hidden="true"></span>
              active: <span data-test="active-theme">{activeLabel()}</span>
            </p>
            <div aria-hidden="true"></div>
          </div>
        </header>

        <div>
          <div>
            <icon name="search" size="sm" tone="muted"></icon>
            <input type="text" placeholder="Search themes…" aria-label="Search themes"
              spellcheck="false" data-test="search" value={q()} on:input={onSearchInput} />
            <if cond={q()}>
              <button type="button" aria-label="Clear search" on:click={clearSearch}>
                <icon name="x" size="xs"></icon>
              </button>
            </if>
          </div>
          <toggle-group variant="segmented" type="single" value={filter()} size="sm" aria-label="Filter by mode"
            on:sail-change={(e) => setFilter(e.detail.value)}>
            <for as="f" of={filters()} key={f.id}>
              <toggle value={f.id} data-test="filter">{f.label}</toggle>
            </for>
          </toggle-group>
        </div>

        <if cond={themes().length === 0}>
          <empty-state icon="search-x" variant="default" size="sm" data-test="empty">
            <span slot="title">No matching themes</span>
            Nothing matches your search.
            <button slot="action" variant="outline" size="sm" on:click={clearSearch}>Clear search</button>
          </empty-state>
        <else>
          <div data-test="themes-grid">
            <for as="t" of={themes()} key={t.id}>
              <button type="button" data-test="theme-card"
                data-id={t.id} data-active={t.isActive}
                on:click={() => open(t.id)}>
                <div aria-hidden="true">
                  <for as="c" of={t.swatches} key={c}>
                    <span style={'background:' + c}></span>
                  </for>
                </div>
                <div>
                  <div>
                    <span>{t.label}</span>
                    <if cond={t.isActive}>
                      <chip tone="success" variant="soft" size="sm" data-test="active-chip">
                        <status-dot slot="leading" tone="success" size="sm" label="Active"></status-dot>
                        active
                      </chip>
                    </if>
                  </div>
                  <div>
                    <badge tone="muted" variant="outline" size="sm">{t.dark ? 'Dark' : 'Light'}</badge>
                    <code>{t.id}</code>
                  </div>
                </div>
              </button>
            </for>
          </div>
        </else></if>

      </div>
    </div>
  </main>
</div>
