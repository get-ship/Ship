import { defineComponent } from '../define'
import template from './set-badge.html.set'

/** Register `<set-badge>`. Idempotent. */
export function registerSetBadge(): void {
  defineComponent(template)
}
