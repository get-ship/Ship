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
// factory. `baseUrl: ''` is same-origin (dev: shipd serves SPA + API on one
// port; Tauri: the host plants `window.__SHIP_DAEMON_URL__` and the SDK
// switches transports). `@ship/sdk-client` is importmap-resolved
// (daemon-served `/_deck/sdk-client.js`); `@ship/sdk/connect` is inlined.
import { createClient } from '@ship/sdk-client'
import { createShipConnectClient } from '@ship/sdk/connect'

/** Live SSE reader — only for ops with a registered live resolver. */
export const ship = createClient()

/** Typed unary request/response client — skills, settings, space, mutations. */
export const ops = createShipConnectClient({ baseUrl: '' })
