import { defineComponent } from '../define'
import template from './set-stat.html.set'

/** Register `<set-stat>`. Idempotent. */
export function registerSetStat(): void {
  defineComponent(template)
}
