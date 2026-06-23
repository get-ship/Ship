// spaces.ts — SDK surface for the spaces scaffold.
//
// One method today: `ship.spaces.config({ id })` — fetches the runtime
// per-space config from the daemon. The daemon resolves the id against
// its space registry; today the launch surface returns an empty
// `routes` array (the spaces scaffold's `routes.gen.ts` is the static
// half; the runtime merge reads this response for the dynamic half).
//
// Kept as a hand-owned file (not generated) because the route payload
// shape is small and the call site (apps/spaces/src/main.ts) imports
// the runtime-merge shape, not the full typed SDK surface.

import { fetchBody } from './http'

/** A single runtime user-defined route. The spaces scaffold merges
 *  these with its static `DYNAMIC_ROUTE_PATTERNS` at boot; the runtime
 *  treats both halves identically. */
export interface SpaceRuntimeRoute {
  /** Path pattern, e.g. `/agents/:id`. */
  pattern: string
  /** Optional layout component tag (e.g. `sui-studio-chrome`). */
  layout?: string
}

/** The runtime per-space config. Mirrors what the daemon returns from
 *  `GET /_deck/spaces/{id}/config` — a future field goes here, not in
 *  generated.ts (the route payload is hand-owned, not specta-derived). */
export interface SpaceConfig {
  /** Runtime user-defined routes — merged with the spaces scaffold's
   *  static `routes.gen.ts` at boot. Empty for the launch surface. */
  routes: SpaceRuntimeRoute[]
}

/** Fetch the per-space config from the daemon. The launch surface
 *  returns an empty `routes` array; the call is here so spaces authors
 *  can wire runtime routes without touching the spaces scaffold.
 *
 *  Catches network failures and returns `null` — boot continues with
 *  the static half alone. A misconfigured daemon is not a fatal error
 *  for the spaces window; the operator sees the absent-route behavior
 *  in the chrome (the daemon's logs flag the 404). */
export async function readSpaceConfig(
  baseUrl: string,
  args: { id: string },
): Promise<SpaceConfig | null> {
  const path = `/_deck/spaces/${encodeURIComponent(args.id)}/config`
  try {
    return await fetchBody<SpaceConfig>(baseUrl, path, 'json')
  } catch {
    return null
  }
}
