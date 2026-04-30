import { defineComponent } from '../define'
import template from './set-progress.html.set'

/** Register `<set-progress>`. Idempotent. */
export function registerSetProgress(): void {
  defineComponent(template)
}
