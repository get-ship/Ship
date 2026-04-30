import { defineComponent } from '../define'
import template from './set-tab.html.set'

/** Register `<set-tab>`. Idempotent. */
export function registerSetTab(): void {
  defineComponent(template)
}
