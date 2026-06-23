// The hub's daemon clients — mirroring the shell's split (apps/shell/src/ops.ts),
// keyed to the daemon's live-data contract (apps/shipd/src/ship_runtime/
// live_resolvers.rs):
//
//   - `ship` = the `@ship/sdk-client` LIVE reader. Ops with a registered live
//     mux resolver — `agents.summaries`, `collections.list`, `git.overview`,
//     `account.state` — stream over SSE as `LiveSource`s:
//     `live(() => ship.agents.summaries())`. Use `ship` ONLY for those ops; an
//     op with no resolver (skills.*, events.list) 404s the subscribe.
//   - `ops` = the typed Connect request/response client. Everything unary:
//     `skills.summaries`/`detail` (a `{ json }` envelope — decode at the call
//     site), `settings.*`, `space.*`, and the write mutations. Throws
//     `ConnectError` on failure (no `resp.ok`).
//
// One of each, shared across every chrome controller — no DI, no per-view
// factory, NO daemon-base wiring: both clients default their transport via the
// SDK's `resolveDaemonBase()` (same-origin under dev; the Tauri loopback origin
// when the host plants `window.__SHIP_DAEMON_URL__`). The app hand-rolls no
// `main.ts` daemon bootstrap — the framework owns reachability.
// `@ship/sdk-client` is importmap-resolved (daemon-served `/_deck/sdk-client.js`);
// `@ship/sdk/connect` is inlined.
//
// Theme transport: `setThemeTransport` is wired here as a module-level side
// effect, mirroring apps/shell/src/ops.ts. Save/load back to the daemon's
// RUNTIME settings table so the theme choice persists across sessions.
// Tier-4 localStorage is still the synchronous cache the boot script reads,
// so the Connect round-trip never blocks paint.
// Register the SUI primitive pack as a module side effect. The chrome pack
// (`@ship/chrome`) and the app-local command-bar components COMPOSE generic
// `<sui-*>` primitives (sui-dialog, sui-icon, sui-input, sui-portal, …) but do
// not register them — by contract the HOST does, via `import
// '@ship/sui-primitives'` (see packages/components/ship/chrome/src/index.ts).
// The shell does this in apps/shell/src/main.ts; the hub has no main.ts, so its
// single host-owned entry module (this clientsModule, imported by entry.gen.ts)
// owns the registration. Without it the command palette's <sui-dialog> stays an
// undefined element — the bar flips to `open` but never paints — and Studio
// card icons render as blank boxes.
import '@ship/sui-primitives'
import { createClient } from '@ship/sdk-client'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
import { setThemeTransport, type ThemeId } from '@ship/tokens/themes'
import { navigate } from '@ship/deck/router'
import { registerKeymap } from '../chrome/keymap'

/** Live SSE reader — only for ops with a registered live resolver. */
export const ship = createClient()

/** Typed unary request/response client — skills, settings, space, mutations. */
export const ops = createShipConnectClient()

// ---------------------------------------------------------------------------
// RUNTIME settings helpers (verbatim from apps/shell/src/ops.ts).
// Used below for the theme transport and available to any hub module that
// imports this file.
// ---------------------------------------------------------------------------

/** Read one RUNTIME setting, parsed from its JSON bytes. `null` when unset. */
export async function readSetting<T = unknown>(key: string): Promise<T | null> {
  try {
    const setting = await ops.settings.get({ key, scope: SettingScope.RUNTIME })
    if (setting.valueJson.length === 0) return null
    return JSON.parse(new TextDecoder().decode(setting.valueJson)) as T
  } catch {
    return null
  }
}

/** Write one RUNTIME setting as JSON bytes. */
export async function writeSetting(key: string, value: unknown): Promise<void> {
  await ops.settings.set({
    key,
    valueJson: new TextEncoder().encode(JSON.stringify(value)),
    scope: SettingScope.RUNTIME,
  })
}

// ---------------------------------------------------------------------------
// Theme transport (mirrors apps/shell/src/ops.ts setThemeTransport block).
// Wired once at module load — fire-and-forget writes ride the RUNTIME settings
// table (machine-local launch defaults) so the theme survives a daemon restart
// but stays per-machine. A failed write surfaces a toast (sync failure ≠ apply
// failure: the theme is already visible via the localStorage cache).
// ---------------------------------------------------------------------------
const THEME_SETTING_KEY = 'shell.theme'
setThemeTransport({
  save(id: ThemeId) {
    void writeSetting(THEME_SETTING_KEY, id).catch((err: unknown) => {
      console.error('[theme] failed to persist theme to daemon settings', err)
      document.dispatchEvent(new CustomEvent('sui:notify', {
        detail: {
          tone: 'warning',
          title: 'Theme not synced',
          description: 'Applied locally, but saving it to the daemon failed — it may not persist across sessions.',
        },
      }))
    })
  },
  async load(): Promise<string | null> {
    return await readSetting<string>(THEME_SETTING_KEY)
  },
})

// ---------------------------------------------------------------------------
// Global keymap (apps/hub/src/chrome/keymap.ts). The hub has no main.ts, so
// this clientsModule is the single host-owned entry seam (imported by
// entry.gen.ts) that owns app-level keydown glue — same role it plays for the
// `@ship/sui-primitives` registration above. The keymap is state-free; it
// takes the router's `navigate` so the keymap module stays decoupled.
// ---------------------------------------------------------------------------
registerKeymap({ navigate })
