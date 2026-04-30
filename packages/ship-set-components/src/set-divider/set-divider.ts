import { defineComponent } from '../define'
import template from './set-divider.html.set'

/** Register `<set-divider>`. Idempotent. */
export function registerSetDivider(): void {
  defineComponent(template)
}
