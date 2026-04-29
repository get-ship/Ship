// state() — semantic alias for `signal`, plus the `debounce` derivation.
//
// Authors call `state('')` for ephemeral local state inside view setup
// or component setup blocks. It's the same engine as `signal` — the
// two names exist so reading a setup block tells you intent at a
// glance: `signal` is the lower-level primitive, `state` is the thing
// the user is binding to.
//
// `debounce(input, ms)` returns a derived signal that lags `input` by
// `ms` milliseconds — each new write to `input` resets the timer and
// the output only updates after `ms` of stillness.

import { effect, signal, untrack, type Signal } from './signal'

export { signal as state } from './signal'
export type { Signal as State } from './signal'

/**
 * A signal that lags `input` by `ms` milliseconds. Each write to
 * `input` restarts the timer; the returned signal only updates after
 * `ms` have elapsed without further writes.
 *
 * Useful for search-as-you-type, where you want to debounce the input
 * before firing a query (`live(() => ship.search(query()))` reading
 * the debounced signal will only re-run once the user stops typing).
 *
 * Implementation detail: the returned signal is reactive and can be
 * subscribed to like any other — `derived` and `effect` work as you'd
 * expect.
 *
 * @param input The source signal to debounce.
 * @param ms Milliseconds of stillness required before the output updates.
 * @returns A signal whose value lags `input`.
 *
 * @example
 * ```ts
 * const query = state('')
 * const debounced = debounce(query, 250)
 * effect(() => console.log('search:', debounced()))
 * query.set('h'); query.set('he'); query.set('hel') // only 'hel' fires
 * ```
 *
 * @see state for the semantic alias of `signal`.
 */
export function debounce<T>(input: Signal<T>, ms: number): Signal<T> {
  const out = signal(untrack(() => input()))
  let timer: ReturnType<typeof setTimeout> | null = null

  effect(() => {
    const value = input()
    if (timer !== null) clearTimeout(timer)
    timer = setTimeout(() => {
      timer = null
      out.set(value)
    }, ms)
    return () => {
      if (timer !== null) {
        clearTimeout(timer)
        timer = null
      }
    }
  })

  return out
}
