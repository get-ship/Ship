// @ship/sdk-client/sse — SSE plumbing for the SDK.
//
// The first `patch` event with `op === 'set'` flips the signal from
// `loading` to `ok`. Subsequent events flow through `applyPatch` against
// a façade that maps T <→ ok-state, so future server-side narrowing
// (per-row patches via `merge` / `append`) lights up without an SDK
// change.

import {
  applyPatch,
  type LiveSignal,
  type LiveSource,
  type Patch,
} from '@ship/sui-runtime'
// Note: `applyPatch` / `Patch` are re-exported from the runtime barrel
// (see packages/sui/runtime/src/index.ts). Importing from the bare
// specifier — not the `@ship/sui-runtime/patch` subpath — keeps the
// bundled `_deck/sdk-client.js` resolvable under the view-host importmap
// which only maps the bare specifier, not the subpath.

/**
 * Wire one `_deck/data` SSE channel into a `LiveSource`. The first
 * `patch` event with `op === 'set'` flips the signal to ok; subsequent
 * events go through `applyPatch` against a façade.
 *
 * Visible for tests via the package main barrel — production code uses
 * `createClient` in `./index.ts` or `liveExpr` above.
 */
export function makeSseSource<T>(
  url: string,
  ESCtor: typeof globalThis.EventSource | undefined,
): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      if (!ESCtor) {
        sig.set({
          kind: 'error',
          error: new Error('[@ship/sdk-client] no EventSource available'),
        })
        return () => {}
      }
      const facade = makeFacade<T>(sig)
      const es = new ESCtor(url)
      let firstSeen = false

      const onPatch = (ev: MessageEvent): void => {
        let patch: Patch
        try {
          patch = JSON.parse(ev.data) as Patch
        } catch (err) {
          sig.set({ kind: 'error', error: toError(err) })
          return
        }
        try {
          if (!firstSeen) {
            firstSeen = true
            // First frame is always a full `set` from the resolver.
            // Use the value directly so the signal transitions cleanly
            // from loading → ok with the same patch flow as updates.
            if (patch.op === 'set' && (!patch.path || patch.path === '')) {
              sig.set({ kind: 'ok', value: patch.value as T })
            } else {
              applyPatch(facade, patch)
            }
          } else {
            applyPatch(facade, patch)
          }
        } catch (err) {
          sig.set({ kind: 'error', error: toError(err) })
        }
      }
      const onError = (): void => {
        if (!firstSeen) {
          sig.set({
            kind: 'error',
            error: new Error('[@ship/sdk-client] SSE connection error'),
          })
        }
      }

      es.addEventListener('patch', onPatch as EventListener)
      es.addEventListener('error', onError)

      return () => {
        es.removeEventListener('patch', onPatch as EventListener)
        es.removeEventListener('error', onError)
        es.close()
      }
    },
  }
}

/**
 * Project a `LiveSignal<T>` into a plain `Signal<T>` view for `applyPatch`.
 * Reads return the inner ok value (undefined while loading/error); writes
 * transition to `{ kind: 'ok', value }`.
 */
function makeFacade<T>(sig: LiveSignal<T>): import('@ship/sui-runtime').Signal<T> {
  type Inner = import('@ship/sui-runtime').Signal<T>
  const reader = (() => {
    const v = sig()
    return v.kind === 'ok' ? v.value : (undefined as unknown as T)
  }) as Inner
  reader.set = (next) => {
    const cur = sig.__peek()
    const prev = cur.kind === 'ok' ? cur.value : (undefined as unknown as T)
    const v = typeof next === 'function' ? (next as (p: T) => T)(prev) : next
    sig.set({ kind: 'ok', value: v })
  }
  reader.__subs = sig.__subs
  reader.__peek = () => {
    const v = sig.__peek()
    return v.kind === 'ok' ? v.value : (undefined as unknown as T)
  }
  return reader
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))
