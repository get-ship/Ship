// The hub's singleton Connect client. Every chrome controller imports
// `ship` from this file — there is no `ops.ts` wrapper, no DI, no
// factory call. The bundler externalizes the importmap-resolved
// `@ship/sdk/connect` so a single client (and single stub registry,
// single user-context) is shared across every chrome controller.
//
// `baseUrl: ''` is intentional: in the dev loop shipd serves the SPA
// at the same origin as the API (vite proxies /api + /_deck +
// /<service>/<method> to shipd's loopback port). In the proxy-less
// Tauri webview the host plants `window.__SHIP_DAEMON_URL__` with the
// loopback origin and the SDK reads that to switch transports —
// same code, both surfaces.
import { createShipConnectClient } from '@ship/sdk/connect'

export const ship = createShipConnectClient({ baseUrl: '' })
