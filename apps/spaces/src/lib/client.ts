// The spaces app's daemon clients + host registrations — the host-owned entry
// seam `entry.gen.ts` imports (declared in `deck.jsonc#entry.clientsModule`).
// Mirrors the hub's `src/lib/client.ts` split, minus the hub-only theme
// transport + keymap (the shell applies its own per-space theme).
//
//   - `ship` = the `@ship/sdk-client` LIVE reader. Ops with a registered live
//     mux resolver stream over SSE as `LiveSource`s. Use for those ops only.
//   - `ops`  = the typed Connect request/response client (unary). Settings,
//     search, mutations. Throws `ConnectError` on failure.
//
// Both default their transport via the SDK's `resolveDaemonBase()` (same-origin
// under dev; the Tauri loopback origin when the host plants
// `window.__SHIP_DAEMON_URL__`). The framework owns reachability.
//
// `import '@ship/sail-primitives'` registers the generic `<sui-*>` pack as a
// module side effect. By contract the HOST registers primitives; the chrome
// the shell composes (`<ship-command-palette>` and its `<sui-dialog>` /
// `<sui-icon>` / `<sui-input>` internals) only COMPOSES them. Without this the
// command palette's dialog stays an undefined element.
import '@ship/sail-primitives'
import { createClient } from '@ship/sdk-client'
import { createShipConnectClient } from '@ship/sdk/connect'

/** Live SSE reader — only for ops with a registered live resolver. */
export const ship = createClient()

/** Typed unary request/response client — settings, search, mutations. */
export const ops = createShipConnectClient()
