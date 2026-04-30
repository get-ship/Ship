import { defineComponent } from '../define'
import template from './set-mark.html.set'

/** Register `<set-mark>`. Idempotent. */
export function registerSetMark(): void {
  defineComponent(template)
}
