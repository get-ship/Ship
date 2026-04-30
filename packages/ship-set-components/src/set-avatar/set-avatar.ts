import { defineComponent } from '../define'
import template from './set-avatar.html.set'

/** Register `<set-avatar>`. Idempotent. */
export function registerSetAvatar(): void {
  defineComponent(template)
}
