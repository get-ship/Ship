// live() — function-form data binding with a tagged-union lifecycle.
//
// `data()` (in src/data.ts) takes the options-shape `{ initial,
// subscribe }` and is the lower-level primitive the patch layer wires
// into. `live()` is the higher-level form: pass a thunk that returns
// the SDK call result, get back a signal whose value walks the
// loading → ok | error states. Tracked dep changes inside the thunk
// re-fire the source.
//
// In Phase 1 the Promise-return case is the focus; the SSE/patch-routed
// path will reuse data()'s machinery in Phase 3 (see TODO inside).

import { effect, signal, type Signal } from './signal'

/** The lifecycle of a `LiveSignal<T>` value. */
export type LiveState<T> =
  | { kind: 'loading' }
  | { kind: 'error'; error: Error }
  | { kind: 'ok'; value: T }

/** A signal that carries the loading/ok/error lifecycle of a live source. */
export type LiveSignal<T> = Signal<LiveState<T>>

/** Options for `live()`. */
export interface LiveOpts {
  /**
   * Called when the source rejects (or throws synchronously). Runs
   * after the signal has transitioned to `{ kind: 'error', error }`.
   */
  onError?: (err: unknown) => void
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))

/**
 * Run `source` inside a tracked reactive scope; expose the result as a
 * `LiveSignal<T>` whose value walks `loading → ok | error`.
 *
 * Behavior:
 *
 * - Initial value is `{ kind: 'loading' }`.
 * - `source` runs eagerly. If it returns a Promise (or a sync value
 *   that we wrap), the signal updates to `{ kind: 'ok', value }` on
 *   resolve or `{ kind: 'error', error }` on reject.
 * - Any tracked signal read inside `source` becomes a dep. When a dep
 *   changes, the signal flips back to `loading`, the previous in-flight
 *   resolution is invalidated (token-based, like `bindAwait`), and
 *   `source` runs again.
 *
 * For SSE/patch-streamed sources, `source` is expected to resolve to
 * the initial snapshot; the underlying stream wiring will reuse
 * `data()`'s `applyPatch` route in Phase 3.
 *
 * TODO(phase-3): wire SSE/patch routing through `data()` for sources
 * that opt into a live subscription. Phase 1 focuses on the Promise
 * round-trip.
 *
 * @param source Thunk that returns a Promise (or sync value). Tracked
 *   signal reads inside register as deps.
 * @param opts Optional `onError` hook fired on each rejection.
 * @returns A `LiveSignal<T>` — read it like any other signal.
 *
 * @example
 * ```ts
 * const id = state('agent-1')
 * const agent = live(() => ship.agents.get(id()))
 * effect(() => {
 *   const s = agent()
 *   if (s.kind === 'ok') console.log(s.value.name)
 * })
 * ```
 *
 * @see data for the lower-level options-shape primitive; bindWhileLoading
 *   and bindWhileError for boundary primitives that mount/unmount on
 *   the lifecycle states.
 */
export function live<T>(
  source: () => Promise<T> | T,
  opts?: LiveOpts,
): LiveSignal<T> {
  const sig = signal<LiveState<T>>({ kind: 'loading' })
  let token = 0

  effect(() => {
    const myToken = ++token
    let result: Promise<T> | T
    try {
      result = source()
    } catch (err) {
      if (myToken !== token) return
      const error = toError(err)
      sig.set({ kind: 'error', error })
      opts?.onError?.(err)
      return
    }
    sig.set({ kind: 'loading' })
    Promise.resolve(result).then(
      (value) => {
        if (myToken !== token) return
        sig.set({ kind: 'ok', value })
      },
      (err) => {
        if (myToken !== token) return
        const error = toError(err)
        sig.set({ kind: 'error', error })
        opts?.onError?.(err)
      },
    )
  })

  return sig
}
