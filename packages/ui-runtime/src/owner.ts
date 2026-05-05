// Mount-time ownership / cleanup tracking.
//
// Pieces of the runtime that allocate long-lived resources during
// `setup()` — `live()` opens an EventSource, `data()` opens an SSE
// subscription, custom hooks may install global listeners — need a
// way to register a cleanup that fires when the enclosing view
// unmounts. Without this, every `mount()` call leaks the resources
// allocated by its setup body. The OOM crash users saw after a
// few minutes of navigation was exactly this: each visit to a view
// with `<data live>` opened a fresh EventSource that was never
// closed, plus the signal graph, effect closures, and event
// listeners hanging off it.
//
// Pattern: `mount.ts` wraps the call to the view module's `setup()`
// (and any other phase that's allowed to register cleanups) in
// `withOwner(owner, () => setup(ctx))`. Allocators inside that scope
// call `registerOwnerCleanup(fn)` to enroll a teardown function;
// the owner's `dispose()` runs them all in reverse insertion order.
// Outside any owner scope, `registerOwnerCleanup` is a no-op (so
// the API is safe to call from non-mount contexts like tests or
// long-lived globals).
//
// Stays free of any DOM / runtime concerns so it can be imported
// from `live.ts`, `data.ts`, hooks, etc. without circular deps.

export interface Owner {
  cleanups: Array<() => void>
  disposed: boolean
}

let CURRENT_OWNER: Owner | null = null

/** Run `body` with `owner` as the current cleanup-collection scope. */
export function withOwner<R>(owner: Owner, body: () => R): R {
  const prev = CURRENT_OWNER
  CURRENT_OWNER = owner
  try {
    return body()
  } finally {
    CURRENT_OWNER = prev
  }
}

/** Register a teardown function with the current owner (if any). */
export function registerOwnerCleanup(cleanup: () => void): void {
  const owner = CURRENT_OWNER
  if (!owner || owner.disposed) return
  owner.cleanups.push(cleanup)
}

/**
 * Build a fresh owner. Call `dispose()` to run every cleanup
 * registered during the owner's active scope, in reverse order.
 * Idempotent.
 */
export function createOwner(): Owner & { dispose: () => void } {
  const owner: Owner & { dispose: () => void } = {
    cleanups: [],
    disposed: false,
    dispose() {
      if (owner.disposed) return
      owner.disposed = true
      for (let i = owner.cleanups.length - 1; i >= 0; i--) {
        try {
          owner.cleanups[i]!()
        } catch (e) {
          console.error('[sui/owner] cleanup threw:', e)
        }
      }
      owner.cleanups.length = 0
    },
  }
  return owner
}
