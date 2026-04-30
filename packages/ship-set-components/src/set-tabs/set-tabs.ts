import { defineComponent } from '../define'
import template from './set-tabs.html.set'

/** Register `<set-tabs>`. Idempotent. */
export function registerSetTabs(): void {
  defineComponent(template)
}
