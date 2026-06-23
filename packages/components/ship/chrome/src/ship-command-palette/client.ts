// Daemon clients for the <ship-command-palette> family (palette · panel · chat).
//
// The pack owns its data plumbing; the host wires nothing. Two handles,
// mirroring the daemon's live-data contract:
//
//   - `ship` = the `@ship/sdk-client` LIVE reader. Used here for the kernel
//     event stream (`ship.onNamespace('agent.session.', …)`) that drives the
//     ai-mode session lifecycle.
//   - `ops`  = the typed Connect request/response client. Everything unary:
//     `agents.*`, `search.search`, `space.active`, `settings.get`.
//
// Both default their transport via the SDK (same-origin under the web SPA;
// the Tauri loopback origin when the host plants `window.__SHIP_DAEMON_URL__`).
// Cookies carry the session, so there is no token plumbing.
import { createClient } from '@ship/sdk-client'
import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'

/** Live SSE reader — only for ops with a registered live resolver. */
export const ship = createClient()

/** Typed unary request/response client — agents, search, space, settings. */
export const ops = createShipConnectClient()

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
