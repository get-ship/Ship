import { defineComponent } from '../define'
import template from './set-chip.html.set'

/** Register `<set-chip>`. Idempotent. */
export function registerSetChip(): void {
  defineComponent(template)
}
