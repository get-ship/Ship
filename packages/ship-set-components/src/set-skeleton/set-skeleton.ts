import { defineComponent } from '../define'
import template from './set-skeleton.html.set'

/** Register `<set-skeleton>`. Idempotent. */
export function registerSetSkeleton(): void {
  defineComponent(template)
}
