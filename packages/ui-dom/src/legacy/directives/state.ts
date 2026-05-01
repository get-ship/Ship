// <state> directive — evaluate the initial-value expression against
// the current scope and produce a Signal that the template can read
// and bind:value to.

import { state, type Signal } from '@ship/ui-runtime'
import type { StateDecl } from '../parse'

/** Build a Signal from a `<state>` declaration. */
export function mountState(
  decl: StateDecl,
  scope: Record<string, unknown>,
): Signal<unknown> {
  const initial = decl.value(scope)
  return state(initial) as Signal<unknown>
}
