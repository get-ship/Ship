import { defineComponent } from '../define'
import template from './set-card.html.set'

/** Register `<set-card>`. Idempotent. */
export function registerSetCard(): void {
  defineComponent(template)
}
