// Shared per-domain Connect client for the shell (#72).
//
// One same-origin client: the daemon's auth middleware reads the session
// cookie, so no token plumbing. Controllers that used to `fetch('/api/v1/…')`
// directly call `ops.<domain>.<method>()` instead — typed, gated server-side,
// off the REST surface. Mutations route through the one gate; reads are C0.
//
// Connect throws `ConnectError` on failure (no `resp.ok` to check) — call
// sites wrap in try/catch where they previously branched on the status code.

import { createShipConnectClient, SettingScope } from '@ship/sdk/connect'
import { createClient } from '@ship/sdk-client'

/** Same-origin daemon op surface. Base URL '' → relative requests + cookies. */
export const ops = createShipConnectClient({ baseUrl: '' })

/**
 * Shell preferences (`cmdk.send_key`, `shell.rail.pinned`, `shell.agent.active`)
 * live in the daemon's RUNTIME settings table — the machine-local launch
 * defaults that the per-space/account overlays sit on top of. Every shell read
 * and write targets that one scope.
 *
 * `SettingsService` carries values as JSON-encoded bytes (`Setting.valueJson`),
 * so these helpers own the encode/decode + scope so call sites stay typed and
 * never reach for `TextEncoder`/`TextDecoder` themselves.
 */

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

/**
 * Shared `@ship/sdk-client` instance for chrome controllers that have no
 * `ctx.ship` (they live in apps/shell, not daemon-served views). Its live
 * reads stream over Connect; mesh/canvas (no per-domain service yet) fall back
 * to the legacy expr-SSE channel internally.
 */
export const ship = createClient()
