import { defineComponent } from '../define'
import template from './set-button.html.set'

/** Register `<set-button>`. Idempotent. */
export function registerSetButton(): void {
  defineComponent(template)
}
