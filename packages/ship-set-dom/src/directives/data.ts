// <data> directive — resolves a top-level data declaration into a
// scope value. `live` mode wraps the source in `live()` for the
// loading/ok/error lifecycle; one-shot mode evaluates the source
// synchronously and stores the result.

import { live, type LiveSignal } from '@ship-set/runtime'
import type { DataDecl } from '../parse'

/** Mount a single `<data>` declaration; returns its scope value. */
export function mountData(
  decl: DataDecl,
  ship: unknown,
  route: Record<string, string>,
): unknown {
  if (decl.live) {
    return live(() => decl.source(ship, route)) as LiveSignal<unknown>
  }
  return decl.source(ship, route)
}
