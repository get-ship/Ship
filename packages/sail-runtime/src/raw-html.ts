// `{@html expr}` binding. Replaces the anchor span's innerHTML with
// the expression's value on each dependency change.
//
// **Author is responsible for sanitization.** This binding never
// escapes the value — that's the whole point — and is the only
// supported XSS surface in the runtime.

import { effect, type Signal } from './signal'
import type { Disposer } from './bind'

/**
 * Replace `el.innerHTML` with the value of `get()` on every dependency
 * change. **Author is responsible for sanitization** — this binding
 * never escapes the value and is the only intended XSS surface in the
 * runtime.
 *
 * Pass extra signals via `deps` when the expression depends on values
 * that aren't read inside `get` (e.g. `get` is a pre-rendered string
 * but should still re-run when some toggle flips).
 *
 * Compiler target: `{@html expr}` template syntax.
 *
 * @param el Element whose `innerHTML` is rewritten.
 * @param get Reactive read of the HTML string.
 * @param deps Extra signals to subscribe to (default `[]`).
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const html = signal('<em>hi</em>')
 * bindRawHtml(span, () => html())
 * ```
 *
 * @see bindText for safe, escaped interpolation.
 */
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
