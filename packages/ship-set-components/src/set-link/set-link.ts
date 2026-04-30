import { defineComponent } from '../define'
import template from './set-link.html.set'

/** Register `<set-link>`. Idempotent. */
export function registerSetLink(): void {
  defineComponent(template)
}
