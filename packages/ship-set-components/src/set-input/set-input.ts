import { defineComponent } from '../define'
import template from './set-input.html.set'

/** Register `<set-input>`. Idempotent. */
export function registerSetInput(): void {
  defineComponent(template)
}
