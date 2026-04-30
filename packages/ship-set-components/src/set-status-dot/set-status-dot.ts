// <set-status-dot> — first-shipped ported primitive. Reference port
// for every primitive that follows.

import { defineComponent } from '../define'
import template from './set-status-dot.html.set'

/** Register the `<set-status-dot>` custom element. Idempotent. */
export function registerSetStatusDot(): void {
  defineComponent(template)
}
