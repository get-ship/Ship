// <data> directive — resolves a top-level data declaration into a
// scope value. `live` mode wraps the source in `live()` for the
// loading/ok/error lifecycle; one-shot mode evaluates the source
// synchronously and stores the result.
//
// The source thunk receives `(ship, route, $scope)`. Reading any
// reactive signal inside the thunk (typically `$scope.<key>`) is
// tracked by `live()`'s effect — when the dep changes, the prior
// SSE subscription disposes and a new one opens. That is how
// reactive `<data>` args work: authors write
// `<data source="ship.agents.get({id: $scope.selectedId()})" live />`
// and the channel re-binds automatically when `selectedId` changes.

import { live, type LiveSignal } from '@ship-set/runtime'
import type { DataDecl } from '../parse'

/** Mount a single `<data>` declaration; returns its scope value. */
export function mountData(
  decl: DataDecl,
  ship: unknown,
  route: Record<string, string>,
  scope: Record<string, unknown>,
): unknown {
  if (decl.live) {
    return live(() => decl.source(ship, route, scope)) as LiveSignal<unknown>
  }
  return decl.source(ship, route, scope)
}
