// `{@html expr}` binding. Replaces the anchor span's innerHTML with
// the expression's value on each dependency change.
//
// **Author is responsible for sanitization.** This binding never
// escapes the value — that's the whole point — and is the only
// supported XSS surface in the runtime.

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'

export function bindRawHtml(
  el: Element,
  get: () => string,
  deps: Signal<unknown>[] = [],
): Disposer {
  return effect(() => {
    // touch deps for the dependency tracker
    for (const d of deps) d()
    el.innerHTML = String(get() ?? '')
  })
}
