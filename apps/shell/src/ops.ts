// Shared per-domain Connect client for the shell (#72).
//
// One same-origin client: the daemon's auth middleware reads the session
// cookie, so no token plumbing. Controllers that used to `fetch('/api/v1/…')`
// directly call `ops.<domain>.<method>()` instead — typed, gated server-side,
// off the REST surface. Mutations route through the one gate; reads are C0.
//
// Connect throws `ConnectError` on failure (no `resp.ok` to check) — call
// sites wrap in try/catch where they previously branched on the status code.

import { createShipConnectClient } from '@ship/sdk/connect'

/** Same-origin daemon op surface. Base URL '' → relative requests + cookies. */
export const ops = createShipConnectClient({ baseUrl: '' })
