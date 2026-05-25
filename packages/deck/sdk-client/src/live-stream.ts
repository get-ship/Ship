// @ship/sdk-client/live-stream — Connect server-stream → LiveSource adapter.
//
// The typed replacement for `makeSseSource`/`liveExpr` (#138). A per-op live
// read is now a Connect server-streaming rpc (`ops.<domain>.<op>Live(req)`)
// that yields the full value as a `JsonResponse` on the initial call and again
// on every relevant kernel event — the same freshness the `/_sui/data?expr=`
// SSE ran, now typed and keyed by the rpc method instead of a parsed string.
//
// This adapter is host-agnostic: it consumes any `AsyncIterable<{ json }>` and
// owns the AbortController, so the runtime `live()` stays free of any Connect
// concept (the host supplies the stream opener). `<data live>` compiles to
// `live(() => liveStream((signal) => ops.<domain>.<op>Live(req, { signal })))`.

import { type LiveSignal, type LiveSource } from '@ship/sui-runtime'

/**
 * Wrap a Connect server-stream into a `LiveSource`. `open` receives an
 * `AbortSignal` (wired to the subscription disposer) and returns the rpc's
 * `AsyncIterable` of `JsonResponse`-shaped frames. Each frame's `json` is the
 * full value (no deltas), so every frame flips the signal to `ok`.
 */
export function liveStream<T>(
  open: (signal: AbortSignal) => AsyncIterable<{ json: string }>,
): LiveSource<T> {
  return {
    subscribe(sig: LiveSignal<T>): () => void {
      const ctrl = new AbortController()
      void (async () => {
        try {
          for await (const frame of open(ctrl.signal)) {
            try {
              sig.set({ kind: 'ok', value: JSON.parse(frame.json) as T })
            } catch (err) {
              sig.set({ kind: 'error', error: toError(err) })
            }
          }
        } catch (err) {
          // An abort is a normal teardown, not a surfaced error.
          if (!ctrl.signal.aborted) {
            sig.set({ kind: 'error', error: toError(err) })
          }
        }
      })()
      return () => ctrl.abort()
    },
  }
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))
