<controller>
// <ship-space-shell> controller — reads the space config (scope=SPACE) and
// drives the shell's config-driven region signals + identity.
//
// Data plumbing is pack-owned (same model as @ship/chrome's controllers): a
// host-agnostic same-origin Connect client, cookies carry the session, no
// token plumbing, the host wires nothing. The settings reads are READ-ONLY
// (C0) — the shell never writes a space setting.
//
// The five config keys (settings, scope=SPACE):
//   glyph       string   space identity icon name (lucide)
//   accent      string   accent color override (applied as --accent)
//   theme       string   ThemeId; applied via applyTheme (no-op if unset)
//   navigation  enum     'rail' | 'top' | 'none'   → which nav region
//   dock        enum     'left' | 'centered' | 'right' | 'off' → dock region

import { controller, signal, derived } from '@ship/sail-runtime'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
import { applyTheme } from '@ship/tokens/themes'
import type { Scope } from './ship-space-shell.sail'

type Nav = 'rail' | 'top' | 'none'
type Dock = 'left' | 'centered' | 'right' | 'off'

interface SpaceConfig {
  glyph: string
  accent: string
  theme: string
  navigation: Nav
  dock: Dock
}

const DEFAULTS: SpaceConfig = {
  glyph: '',
  accent: '',
  theme: '',
  navigation: 'rail',
  dock: 'off',
}

// Same-origin Connect client — unary reads (settings.get is request/response,
// not the live SSE reader).
const ops = createShipConnectClient({ baseUrl: '' })

const isNav = (v: unknown): v is Nav => v === 'rail' || v === 'top' || v === 'none'
const isDock = (v: unknown): v is Dock =>
  v === 'left' || v === 'centered' || v === 'right' || v === 'off'

/** Read one SPACE-scoped setting, parsed from its JSON bytes. `null` when
 *  unset or unreachable — boot continues on the defaults. */
async function readSetting<T>(key: string): Promise<T | null> {
  try {
    const setting = await ops.settings.get({ key, scope: SettingScope.SPACE })
    if (setting.valueJson.length === 0) return null
    return JSON.parse(new TextDecoder().decode(setting.valueJson)) as T
  } catch {
    return null
  }
}

async function loadConfig(): Promise<SpaceConfig> {
  const [glyph, accent, theme, navigation, dock] = await Promise.all([
    readSetting<string>('glyph'),
    readSetting<string>('accent'),
    readSetting<string>('theme'),
    readSetting<string>('navigation'),
    readSetting<string>('dock'),
  ])
  return {
    glyph: glyph ?? DEFAULTS.glyph,
    accent: accent ?? DEFAULTS.accent,
    theme: theme ?? DEFAULTS.theme,
    navigation: isNav(navigation) ? navigation : DEFAULTS.navigation,
    dock: isDock(dock) ? dock : DEFAULTS.dock,
  }
}

/** Apply the space identity: theme (only when set, so an unset space keeps
 *  the persisted theme) + accent override on the document root. */
function applyIdentity(cfg: SpaceConfig): void {
  if (cfg.theme) applyTheme(cfg.theme)
  if (cfg.accent) document.documentElement.style.setProperty('--accent', cfg.accent)
}

export default controller(() => {

    const navigation = signal<Nav>(DEFAULTS.navigation)
    const dock = signal<Dock>(DEFAULTS.dock)
    const glyph = signal<string>(DEFAULTS.glyph)
    const editMode = signal<boolean>(false)

    const showTop = derived<boolean>(() => navigation() === 'top')
    const showRail = derived<boolean>(() => navigation() === 'rail')
    const showDock = derived<boolean>(() => dock() !== 'off')
    const dockSide = derived<string>(() => (dock() === 'off' ? '' : dock()))

    return { navigation, dock, glyph, editMode, showTop, showRail, showDock, dockSide }
})
</controller>

<!--
  <ship-space-shell> — the spaces app's config-driven shell.

  This is the LAYOUT chrome for the spaces app: `routes/_layout.sui` names
  this tag, so the bundler binds `{ pathPrefix: '/', tag: 'ship-space-shell' }`
  and the runtime wraps every space route in this element. It is the spaces
  analogue of `<ship-app-chrome>` (the hub's root chrome) — but where the hub
  chrome is fixed, this shell composes its regions FROM CONFIG.

  ── Slot composition (config-driven) ──────────────────────────────────────
  The controller reads the space's settings (scope=SPACE) and flips the
  region signals. Each chrome region is rendered ONLY when its config says so;
  inside each region a NAMED SLOT is the clean fill point for the real chrome
  component (today @ship/chrome; a later relocation moves these to
  @ship/shared). The shell decides PRESENCE; the slotted component fills it.

    navigation = 'top'   → renders the top-nav region   (slot="top-nav")
    navigation = 'rail'  → renders the rail region      (slot="rail")
    navigation = 'none'  → neither
    dock != 'off'        → renders the dock region       (slot="dock"),
                           aligned left | centered | right via data-dock

  ── View outlet ───────────────────────────────────────────────────────────
  `<slot name="route">` IS the view outlet. `mountRouter` (@ship/deck/router)
  appends a `<div slot="route">` to this element and mounts the active route
  (from `routes.gen.ts` modules) into it; this named slot projects it.

  ── Platform layer (always-on, Ship-provided) ─────────────────────────────
  `<ship-command-palette>` is mounted unconditionally — NOT config-gated. It
  is the cmdk surface AND the unified search (its `find` mode drives
  `ops.search.search`). Every space gets it for free.

  ── Edit mode ──────────────────────────────────────────────────────────────
  `editMode()` (a signal toggled by the `ship:space:edit-mode` document event)
  gates an EMPTY overlay slot — the hook only. No WYSIWYG content today.

  ── Identity ────────────────────────────────────────────────────────────────
  The controller applies the space `theme` (applyTheme) + `accent` (a runtime
  `--accent` override on the document root). The `glyph` (icon name) is the
  space's identity mark, rendered top-left when set.
-->

<!-- style block removed: use sx / external CSS -->

<if cond={glyph()}>
<div data-test="space-glyph" aria-hidden="true">
  <icon name={glyph()} size="md"></icon>
</div>
</if>

<if cond={showTop()}>
<header data-test="space-top-nav" role="banner">
  <slot name="top-nav"></slot>
</header>
</if>

<div>
  <if cond={showRail()}>
  <aside
    data-test="space-rail"
    role="navigation"
    aria-label="Space navigation"
  >
    <slot name="rail"></slot>
  </aside>
  </if>

  <main data-test="space-view-outlet">
    <slot name="route"></slot>
  </main>
</div>

<if cond={showDock()}>
<footer
  data-dock={dockSide()}
  data-test="space-dock"
>
  <slot name="dock"></slot>
</footer>
</if>

<ship-command-palette data-test="space-cmdk"></ship-command-palette>

<if cond={editMode()}>
<div data-test="space-edit-overlay">
  <!-- edit overlay mounts here -->
  <slot name="edit-overlay"></slot>
</div>
</if>
