// Sail runtime — fine-grained reactive signals.
//
// Solid-shaped: signals are sources, derived are memoized computeds,
// effects subscribe and re-run on dep change. Reading inside a tracked
// scope (effect/derived) auto-subscribes; writing notifies subscribers.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Signal engine.

type Subscriber = () => void

let CURRENT_OBSERVER: Subscriber | null = null
let BATCH_DEPTH = 0
const PENDING: Set<Subscriber> = new Set()

/**
 * Reactive signal. Read with `sig()`; write with `sig.set(v)` or
 * `sig.set(prev => next)`.
 */
export interface Signal<T> {
  (): T
  set(value: T | ((prev: T) => T)): void
  /** package-private — used by bind* helpers and patch applier. */
  __subs: Set<Subscriber>
  /** package-private — current cached value, useful for non-tracking reads. */
  __peek(): T
}

/** Create a fine-grained reactive signal. */
export function signal<T>(initial: T): Signal<T> {
  let value = initial
  const subs = new Set<Subscriber>()

  const sig = (() => {
    if (CURRENT_OBSERVER !== null) subs.add(CURRENT_OBSERVER)
    return value
  }) as Signal<T>

  sig.set = (next) => {
    const v =
      typeof next === 'function' ? (next as (prev: T) => T)(value) : next
    if (Object.is(v, value)) return
    value = v
    if (BATCH_DEPTH > 0) {
      for (const s of subs) PENDING.add(s)
    } else {
      // Snapshot subscribers so re-entrant unsubscribes during notify
      // don't disturb iteration.
      for (const s of [...subs]) s()
    }
  }
  sig.__subs = subs
  sig.__peek = () => value

  return sig
}

/**
 * Memoized computed. Reads its dependencies; recomputes when any change.
 * The returned reader auto-tracks and is itself a signal-like value.
 */
export interface Derived<T> extends Signal<T> {}

export function derived<T>(fn: () => T): Derived<T> {
  let value: T
  let dirty = true
  const subs = new Set<Subscriber>()
  let trackedSigs: Signal<unknown>[] = []

  const recompute = () => {
    const prev = CURRENT_OBSERVER
    CURRENT_OBSERVER = invalidate
    const oldTracked = trackedSigs
    const newTracked: Signal<unknown>[] = []
    // Trick: temporarily wrap tracking so we collect dependencies.
    const collector: Subscriber = () => invalidate()
    CURRENT_OBSERVER = collector
    try {
      value = fn()
    } finally {
      CURRENT_OBSERVER = prev
    }
    // Replace untracked subscriptions
    for (const t of oldTracked) t.__subs.delete(invalidate)
    trackedSigs = newTracked
    dirty = false
  }

  const invalidate = () => {
    if (dirty) return
    dirty = true
    // Notify subscribers — they read us next time and recompute lazily.
    for (const s of [...subs]) s()
  }

  const reader = (() => {
    if (CURRENT_OBSERVER !== null) subs.add(CURRENT_OBSERVER)
    if (dirty) recompute()
    return value
  }) as Derived<T>
  reader.set = () => {
    throw new Error('cannot set on a derived; mutate its dependencies instead')
  }
  reader.__subs = subs
  reader.__peek = () => {
    if (dirty) recompute()
    return value
  }

  return reader
}

/**
 * Run `fn` once eagerly + on any dep change. Returns a disposer that
 * unsubscribes. `fn` may return a cleanup callback that runs before
 * each re-run and on disposal.
 */
export function effect(fn: () => void | (() => void)): () => void {
  let cleanup: (() => void) | void = undefined
  let active = true

  const run = () => {
    if (!active) return
    if (cleanup) {
      try {
        cleanup()
      } catch (e) {
        console.error('[sail] effect cleanup threw:', e)
      }
      cleanup = undefined
    }
    const prev = CURRENT_OBSERVER
    CURRENT_OBSERVER = run
    try {
      cleanup = fn()
    } catch (e) {
      console.error('[sail] effect threw:', e)
    } finally {
      CURRENT_OBSERVER = prev
    }
  }

  run()

  return () => {
    active = false
    if (cleanup) {
      try {
        cleanup()
      } catch (e) {
        console.error('[sail] effect cleanup threw:', e)
      }
    }
  }
}

/**
 * Group multiple signal writes; subscribers fire once at the end.
 * Used by the patch applier to apply multi-op patches atomically.
 */
export function batch(fn: () => void): void {
  BATCH_DEPTH++
  try {
    fn()
  } finally {
    BATCH_DEPTH--
    if (BATCH_DEPTH === 0) {
      const subs = [...PENDING]
      PENDING.clear()
      for (const s of subs) s()
    }
  }
}

/**
 * Read signals without subscribing the current observer to them.
 * Useful inside effects when a value is consulted but shouldn't
 * trigger re-runs (e.g., timestamps, RNG seeds).
 */
export function untrack<T>(fn: () => T): T {
  const prev = CURRENT_OBSERVER
  CURRENT_OBSERVER = null
  try {
    return fn()
  } finally {
    CURRENT_OBSERVER = prev
  }
}
