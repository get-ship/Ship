//! The one `MutableState` proxy. Both mount paths build their writable state
//! through `createMountState` — there is no second implementation.
//!
//! Two paths, one builder, one difference (a flag):
//! - **view-mount + `on:` handlers** read signal keys as their UNWRAPPED value
//!   (the ergonomic `state.count++`, `state.label = 'hi'`); the raw signal is
//!   reached via `state.$signals.x`. → `unwrap: true`.
//! - **component-mount** reads signal keys as the RAW signal so refs and prop
//!   signals stay callable (`state.termHost()`, `state.open()`). → `unwrap: false`.
//!
//! Everything else — `$signals`, set-routing (with the function-wrap for
//! `state.foo = () => …`), `has`/`ownKeys`/descriptors — is identical and lives
//! here once. The compiler emits a call to `componentMountState` instead of
//! hand-rolling a proxy into every compiled module (so behavior can't drift
//! between compiled output and the runtime).

import type { Signal } from './signal'
import { isSignal } from './signal'

/** The read-only accessor key exposing the raw signal map: `state.$signals.x`. */
export const SIGNALS_KEY = '$signals'

export interface MountStateOptions {
  /** Reading a signal key returns its unwrapped value (view) vs the raw,
   *  callable signal (component). */
  unwrap: boolean
}

/**
 * Build the canonical writable-state proxy over `scope`. `signals` is the
 * writable-signal subset of `scope` (drives `$signals`, set-routing, and key
 * enumeration). `unwrap` selects value-read vs raw-signal-read semantics.
 */
export function createMountState(
  scope: Record<string, unknown>,
  signals: Record<string, Signal<unknown>>,
  opts: MountStateOptions,
): Record<string, unknown> {
  const { unwrap } = opts
  return new Proxy(scope, {
    get(t, k: string | symbol) {
      if (k === SIGNALS_KEY) return signals
      // Unwrap only DECLARED signal keys (view ergonomic); everything else —
      // non-signal members, and all keys under unwrap:false — reads raw, so
      // refs/prop-signals stay callable (state.termHost()).
      if (unwrap && (k as string) in signals) return signals[k as string]!()
      return (t as Record<string | symbol, unknown>)[k]
    },
    set(_t, k: string | symbol, value: unknown) {
      if (k === SIGNALS_KEY) {
        console.warn(
          `[ship-set] ${SIGNALS_KEY} is the read-only signal-map accessor; the assignment was ignored.`,
        )
        return true
      }
      const sig = signals[k as string]
      if (sig) {
        // signal.set treats a function arg as an updater; a `state.foo = () =>
        // {…}` assignment means STORE the function, so wrap it as a constant
        // updater.
        if (typeof value === 'function') sig.set(() => value as never)
        else sig.set(value as never)
        return true
      }
      console.warn(
        `[ship-set] s.${String(k)} = … was ignored — ${String(k)} is not a ` +
          `writable signal. Declare it as <state name="${String(k)}"> in the ` +
          `template or return it from the scope factory.`,
      )
      // Return true so strict-mode module assignments don't throw — the write
      // is dropped, the warning is the signal.
      return true
    },
    has(t, k) {
      return k === SIGNALS_KEY || k in (t as object)
    },
    ownKeys() {
      return [SIGNALS_KEY, ...Object.keys(signals)]
    },
    getOwnPropertyDescriptor(_t, k: string | symbol) {
      if (k === SIGNALS_KEY) {
        return { configurable: true, enumerable: true, writable: false, value: signals }
      }
      const sig = signals[k as string]
      if (!sig) return undefined
      return {
        configurable: true,
        enumerable: true,
        writable: true,
        value: unwrap ? sig() : sig,
      }
    },
  }) as Record<string, unknown>
}

/** The writable-signal subset of a scope object. */
export function pickSignals(scope: Record<string, unknown>): Record<string, Signal<unknown>> {
  const out: Record<string, Signal<unknown>> = {}
  for (const [k, v] of Object.entries(scope)) {
    if (isSignal(v)) out[k] = v
  }
  return out
}

/**
 * Component-mount state. The compiler emits `componentMountState($scope)` in
 * place of a hand-rolled proxy. Raw-signal reads (`unwrap: false`) keep refs
 * and prop signals callable; `$signals` exposes the signal map.
 */
export function componentMountState(scope: Record<string, unknown>): Record<string, unknown> {
  return createMountState(scope, pickSignals(scope), { unwrap: false })
}
