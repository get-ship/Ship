// @ship/sdk-client/shared-source — refcounted LiveSource dedup.
//
// Background (#198): every `live(() => ship.<domain>.<op>(args))` call inside
// `live()` calls `subscribe(sig)` on a fresh `LiveSource`, which opens a fresh
// upstream (SSE or Connect server-stream). Two views that both read
// `ship.agents.summaries()` open two connections for identical data, two
// reactive trees that re-flip on every patch.
//
// `sharedSource(key, build)` returns a stable LiveSource that:
// - opens the upstream lazily on the first subscriber
// - fans out every upstream value to all subscribers
// - replays the last value to late joiners (so a second view joining after
//   the first patch arrived doesn't sit in `loading` forever)
// - tears down the upstream when the last subscriber disposes
//
// The cache is module-scoped — keyed by `domain|op|args` from `sse()`. It
// only deduplicates calls within a single SDK client instance (one cache per
// process; createClient is a singleton in practice).

import { type LiveSignal, type LiveSource, type LiveState } from '@ship/sui-runtime'

interface CacheEntry<T> {
  refCount: number
  last: LiveState<T> | null
  subs: Set<LiveSignal<T>>
  dispose: () => void
}

const cache = new Map<string, CacheEntry<unknown>>()

export function sharedSource<T>(key: string, build: () => LiveSource<T>): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      let entry = cache.get(key) as CacheEntry<T> | undefined
      if (!entry) {
        const fresh: CacheEntry<T> = {
          refCount: 0,
          last: null,
          subs: new Set(),
          dispose: () => {},
        }
        // Upstream pushes to a fanout-shaped LiveSignal whose .set() broadcasts.
        // We don't want a real reactive signal here — that would tie cache
        // lifetime to whatever effect-scope first subscribes.
        const fanoutSig = ((): LiveSignal<T> => {
          let v: LiveState<T> = { kind: 'loading' }
          const fn = (() => v) as LiveSignal<T>
          fn.set = (next: LiveState<T> | ((prev: LiveState<T>) => LiveState<T>)) => {
            const nv = typeof next === 'function'
              ? (next as (prev: LiveState<T>) => LiveState<T>)(v)
              : next
            v = nv
            fresh.last = nv
            for (const s of fresh.subs) s.set(nv)
          }
          fn.__subs = new Set()
          fn.__peek = () => v
          return fn
        })()
        const upstreamUnsub = build().subscribe(fanoutSig)
        fresh.dispose = () => {
          try { upstreamUnsub() } catch { /* upstream teardown errors swallowed */ }
        }
        cache.set(key, fresh as CacheEntry<unknown>)
        entry = fresh
      }
      entry.refCount++
      entry.subs.add(sig)
      if (entry.last) sig.set(entry.last)
      return () => {
        if (!entry) return
        entry.subs.delete(sig)
        entry.refCount--
        if (entry.refCount === 0) {
          entry.dispose()
          cache.delete(key)
        }
      }
    },
  }
}

/** Test-only: drop every cached entry and close upstreams. */
export function _resetSharedSourceCache(): void {
  for (const e of cache.values()) e.dispose()
  cache.clear()
}
