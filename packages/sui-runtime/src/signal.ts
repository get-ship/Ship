// SUI runtime — fine-grained reactive signals.
//
// Solid-shaped: signals are sources, derived are memoized computeds,
// effects subscribe and re-run on dep change. Reading inside a tracked
// scope (effect/derived) auto-subscribes; writing notifies subscribers.
//
// Dependency tracking is diff-based: each tracked run records an
// "untrack" thunk per fresh subscription; the next run (or disposal)
// fires the prior thunks first so deps the closure no longer reads
// stop firing it. Mirrors `crates/sui-signals` to keep server-side
// and browser-side reactivity in lockstep.

type Subscriber = () => void
type UntrackThunk = () => void

let CURRENT_OBSERVER: Subscriber | null = null
// `null` outside any tracked run. Inside an effect/derived run, points
// at the Vec that observer is using to collect untracks for this run.
let CURRENT_UNTRACKS: UntrackThunk[] | null = null
let BATCH_DEPTH = 0
const PENDING: Set<Subscriber> = new Set()

/**
 * A reactive signal — a callable getter paired with a setter and a
 * subscribe handle.
 *
 * Reading inside an `effect` or `derived` automatically registers a
 * dependency. Writing notifies subscribers synchronously (or coalesces
 * the notification when the write happens inside `batch`).
 *
 * @see signal to construct one; derived for memoized reads; effect for
 *   side effects.
 */
export interface Signal<T> {
  /** Read the current value (and track in any active reactive scope). */
  (): T
  /**
   * Write a new value. Accepts either the next value directly or an
   * updater function that receives the previous value.
   */
  set(value: T | ((prev: T) => T)): void
  /** package-private — used by bind* helpers and patch applier. */
  __subs: Set<Subscriber>
  /** package-private — current cached value, useful for non-tracking reads. */
  __peek(): T
}

/**
 * Subscribe the current observer (if any) to `subs` and record an untrack
 * thunk so it can be removed on the observer's next run or disposal.
 *
 * Called by both `signal` and `derived` getters.
 */
function track(subs: Set<Subscriber>): void {
  if (CURRENT_OBSERVER === null) return
  if (subs.has(CURRENT_OBSERVER)) return
  const sub = CURRENT_OBSERVER
  subs.add(sub)
  if (CURRENT_UNTRACKS !== null) {
    CURRENT_UNTRACKS.push(() => subs.delete(sub))
  }
}

/**
 * Create a fine-grained reactive signal.
 *
 * The returned function is the reader: call it inside a `derived` or
 * `effect` to subscribe. `.set(next)` writes a new value (skipped via
 * `Object.is` if unchanged) and synchronously runs every dependent
 * scope. Writes inside `batch` are deferred to a single end-of-batch
 * flush.
 *
 * @param initial Starting value the reader returns until the first set.
 * @returns A signal: callable getter + `.set` writer.
 *
 * @example
 * ```ts
 * const count = signal(0)
 * effect(() => console.log('count:', count()))
 * count.set(1)              // → 'count: 1'
 * count.set((n) => n + 1)   // → 'count: 2'
 * ```
 *
 * @see derived for computed values; effect for side effects; batch to
 *   coalesce multi-signal writes.
 */
export function signal<T>(initial: T): Signal<T> {
  let value = initial
  const subs = new Set<Subscriber>()

  const sig = (() => {
    track(subs)
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
 * A memoized computed — reads its dependencies on first access and
 * recomputes lazily when any of them change. Behaves as a read-only
 * signal: callable getter, callable from inside other reactive scopes.
 * Calling `.set` on a derived throws.
 */
export interface Derived<T> extends Signal<T> {}

/**
 * Create a memoized computed value. `fn` runs once on first read to
 * seed the cache, then re-runs on demand when a tracked dependency
 * invalidates.
 *
 * Subscribers are notified on invalidation, not on recompute — readers
 * pay the recomputation cost only when they actually pull. Dependencies
 * are diffed each run: deps read in run N but not in run N+1 are
 * unsubscribed, so conditional reads (e.g. `if cond() { a() } else
 * { b() }`) do not leak stale subscriptions.
 *
 * @param fn Pure read function; tracked reads inside register as deps.
 * @returns A signal-like reader. Calling `.set` throws.
 *
 * @example
 * ```ts
 * const first = signal('Ada')
 * const last = signal('Lovelace')
 * const full = derived(() => `${first()} ${last()}`)
 * effect(() => console.log(full())) // → 'Ada Lovelace'
 * first.set('Grace')                 // → 'Grace Lovelace'
 * ```
 *
 * @see signal for sources; effect for side effects.
 */
export function derived<T>(fn: () => T): Derived<T> {
  let value: T
  let dirty = true
  const subs = new Set<Subscriber>()
  let untracks: UntrackThunk[] = []

  const invalidate: Subscriber = () => {
    if (dirty) return
    dirty = true
    // Notify subscribers — they pull us lazily on their next run.
    for (const s of [...subs]) s()
  }

  const recompute = () => {
    // Drop prior dependency subscriptions.
    for (const u of untracks) u()
    untracks = []

    const prevObs = CURRENT_OBSERVER
    const prevUnt = CURRENT_UNTRACKS
    CURRENT_OBSERVER = invalidate
    CURRENT_UNTRACKS = untracks
    try {
      value = fn()
    } finally {
      CURRENT_OBSERVER = prevObs
      CURRENT_UNTRACKS = prevUnt
    }
    dirty = false
  }

  const reader = (() => {
    track(subs)
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
 * Run `fn` once eagerly, then re-run on every tracked dependency
 * change. Returns a disposer that unsubscribes and stops further runs.
 *
 * `fn` may return a cleanup callback. The cleanup runs once before each
 * re-run (so users can detach listeners, abort fetches, etc.) and once
 * on disposal. Errors thrown inside `fn` or its cleanup are logged and
 * swallowed — one bad effect never breaks the reactive graph.
 *
 * Dependencies are diffed each run: signals read in run N but not in
 * run N+1 are unsubscribed. Disposal proactively unsubscribes from
 * every signal the effect was tracking (no waiting for lazy pruning).
 *
 * @param fn Side-effecting function; tracked reads register as deps.
 *   May return an optional cleanup callback.
 * @returns A disposer that detaches the effect.
 *
 * @example
 * ```ts
 * const url = signal('/api/feed')
 * const dispose = effect(() => {
 *   const ctrl = new AbortController()
 *   fetch(url(), { signal: ctrl.signal }).catch(() => {})
 *   return () => ctrl.abort()  // cleanup runs before next re-run
 * })
 * // later...
 * dispose()
 * ```
 *
 * @see signal, derived; batch to coalesce writes that would otherwise
 *   re-fire the effect multiple times.
 */
export function effect(fn: () => void | (() => void)): () => void {
  let cleanup: (() => void) | void = undefined
  let active = true
  let untracks: UntrackThunk[] = []

  const run: Subscriber = () => {
    if (!active) return
    if (cleanup) {
      try {
        cleanup()
      } catch (e) {
        console.error('[sui] effect cleanup threw:', e)
      }
      cleanup = undefined
    }

    // Drop prior subscriptions so this run cleanly re-tracks.
    for (const u of untracks) u()
    untracks = []

    const prevObs = CURRENT_OBSERVER
    const prevUnt = CURRENT_UNTRACKS
    CURRENT_OBSERVER = run
    CURRENT_UNTRACKS = untracks
    try {
      cleanup = fn()
    } catch (e) {
      console.error('[sui] effect threw:', e)
    } finally {
      CURRENT_OBSERVER = prevObs
      CURRENT_UNTRACKS = prevUnt
    }
  }

  run()

  return () => {
    active = false
    // Proactively remove this effect from every signal's sub set.
    for (const u of untracks) u()
    untracks = []
    if (cleanup) {
      try {
        cleanup()
      } catch (e) {
        console.error('[sui] effect cleanup threw:', e)
      }
      cleanup = undefined
    }
  }
}

/**
 * Group multiple signal writes into one notification cycle. Subscribers
 * are queued during `fn` and flushed exactly once when the outermost
 * `batch` returns.
 *
 * Nests safely — only the outermost batch flushes. Used by the patch
 * applier to keep multi-op SSE patches atomic against derived/effects.
 *
 * @param fn Code that writes one or more signals.
 *
 * @example
 * ```ts
 * const x = signal(0), y = signal(0)
 * effect(() => console.log(x() + y()))   // logs 0
 * batch(() => {
 *   x.set(1)
 *   y.set(2)
 * })                                     // logs 3 once (not 1, then 3)
 * ```
 *
 * @see applyPatches which uses batch to apply many patches atomically.
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
 * Useful inside effects/derived when a value is consulted but
 * shouldn't trigger re-runs (timestamps, RNG seeds, peeking at peers).
 *
 * @param fn Code that reads signals; reads inside are not tracked.
 * @returns Whatever `fn` returns.
 *
 * @example
 * ```ts
 * const count = signal(0), label = signal('hits')
 * effect(() => {
 *   const stamp = untrack(() => Date.now())     // no dep on Date
 *   console.log(`${label()}: ${count()} @${stamp}`)
 * })
 * ```
 *
 * @see effect, derived.
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
