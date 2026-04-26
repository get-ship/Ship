// data() — server-owned signal with optional SSE patch subscription.
//
// The compiler emits each `data live x = source.query(...)` as a
// data({ initial, subscribe }) call. The signal carries an SSR'd value
// at hydrate time; patches arrive as `patch` SSE events and are routed
// through applyPatch (or a custom handler).
//
// See docs/sail/03-runtime-architecture.md § Data primitives.
// See docs/sail/05-server-data-bridge.md for the SchemaSource side.

import { applyPatch, type Patch } from './patch'
import { signal, type Signal } from './signal'

export interface DataOpts<T> {
  /** Initial value (typically server-rendered). */
  initial: T
  /** SSE endpoint URL; omit for static data (no live subscription). */
  subscribe?: string
  /** Custom patch handler — overrides the default applyPatch route. */
  onPatch?: (patch: Patch, sig: Signal<T>) => void
  /** Error handler for SSE/parse failures. */
  onError?: (err: unknown) => void
  /**
   * EventSource constructor injection — defaults to `globalThis.EventSource`.
   * Tests pass a fake; SSR passes `undefined` to skip subscription.
   */
  EventSource?: typeof globalThis.EventSource
}

export interface DataHandle<T> {
  /** The reactive signal. */
  signal: Signal<T>
  /** Disposer — closes the SSE connection. */
  dispose: () => void
}

/**
 * Create a data signal. If `subscribe` is provided and an EventSource
 * implementation is available, opens the stream and routes `patch`
 * events into the signal.
 */
export function data<T>(opts: DataOpts<T>): DataHandle<T> {
  const sig = signal(opts.initial)
  const ESCtor = opts.EventSource ?? globalThis.EventSource
  if (!opts.subscribe || !ESCtor) {
    return { signal: sig, dispose: () => {} }
  }

  const sse = new ESCtor(opts.subscribe)
  const onMessage = (ev: MessageEvent) => {
    let patch: Patch
    try {
      patch = JSON.parse(ev.data) as Patch
    } catch (err) {
      opts.onError?.(err)
      return
    }
    try {
      if (opts.onPatch) opts.onPatch(patch, sig)
      else applyPatch(sig, patch)
    } catch (err) {
      opts.onError?.(err)
    }
  }
  const onError = (err: Event) => opts.onError?.(err)

  sse.addEventListener('patch', onMessage as EventListener)
  sse.addEventListener('error', onError)

  return {
    signal: sig,
    dispose: () => {
      sse.removeEventListener('patch', onMessage as EventListener)
      sse.removeEventListener('error', onError)
      sse.close()
    },
  }
}
