import { defineComponent } from '../define'
import template from './set-kbd.html.set'

/** Register `<set-kbd>`. Idempotent. */
export function registerSetKbd(): void {
  defineComponent(template)
}
