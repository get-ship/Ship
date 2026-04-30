import { defineComponent } from '../define'
import template from './set-spinner.html.set'

/** Register `<set-spinner>`. Idempotent. */
export function registerSetSpinner(): void {
  defineComponent(template)
}
