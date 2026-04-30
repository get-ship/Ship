import { defineComponent } from '../define'
import template from './set-empty-state.html.set'

/** Register `<set-empty-state>`. Idempotent. */
export function registerSetEmptyState(): void {
  defineComponent(template)
}
