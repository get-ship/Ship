// @ship/sdk-client/mux-client — per-client multiplex connection manager.
//
// Maintains one Window + one SSE stream per client instance. All live reads
// flow through the multiplex:
//   POST /_deck/mux/open      — once, lazily on first subscription
//   POST /_deck/mux/subscribe — once per (domain, op, args) tuple
//   GET  /_deck/mux/stream/{window_id} — one EventSource, shared
//
// mux-data frames carry the full re-resolved value for each subscription_id;
// this module routes each frame to the registered LiveSignal as `{kind:'ok'}`.
// mux-close surfaces an error to every live signal and resets the connection.
//
// Lifecycle:
//   - subscribe()  increments refCount; teardown decrements it.
//   - refCount → 0 closes the EventSource + drops the window so the next
//     subscribe re-opens cleanly.

import { type LiveSignal } from '@ship/sui-runtime'

// Fixed companion to deck's DECK_PREFIX ('/_deck'). Lives here so this
// module stays free of an index.ts import cycle.
const MUX = '/_deck/mux'

export class MuxClient {
  private windowId: string | null = null
  private streamEs: InstanceType<typeof globalThis.EventSource> | null = null
  private openPromise: Promise<string> | null = null
  private readonly subs = new Map<string, LiveSignal<unknown>>()
  private refCount = 0

  constructor(
    private readonly baseUrl: string,
    private readonly ESCtor: typeof globalThis.EventSource | undefined,
  ) {}

  /**
   * Ensure window + stream are open, subscribe on the server, and route
   * incoming mux-data frames to `sig`. The returned teardown unsubscribes
   * and decrements refCount (closing the window when the last sub leaves).
   */
  subscribe<T>(domain: string, op: string, args: unknown, sig: LiveSignal<T>): () => void {
    if (!this.ESCtor) {
      sig.set({
        kind: 'error',
        error: new Error('[@ship/sdk-client] no EventSource available'),
      })
      return () => {}
    }

    this.refCount++
    let subId: string | null = null
    let teardownCalled = false

    // Eagerly start the open so parallel subscribes reuse the same window.
    const openP = this._ensureWindow()

    void openP
      .then(async (windowId) => {
        try {
          const resp = await fetch(`${this.baseUrl}${MUX}/subscribe`, {
            method: 'POST',
            credentials: 'same-origin',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ window_id: windowId, domain, op, args: args ?? {} }),
          })
          if (!resp.ok) {
            if (!teardownCalled) {
              sig.set({
                kind: 'error',
                error: new Error(`[@ship/sdk-client] mux/subscribe failed: ${resp.status}`),
              })
            }
            return
          }
          const data = (await resp.json()) as {
            subscription_id: string
            initial_value: unknown
          }
          const id = data.subscription_id
          if (teardownCalled) {
            // Torn down before the server-round-trip completed — clean up server-side
            // using the windowId captured in this closure (this.windowId may be null
            // if _reset() already ran from a concurrent last-teardown).
            this._postUnsubscribe(windowId, id)
          } else {
            subId = id
            this.subs.set(id, sig as LiveSignal<unknown>)
            sig.set({ kind: 'ok', value: data.initial_value as T })
          }
        } catch (err) {
          if (!teardownCalled) {
            sig.set({ kind: 'error', error: toError(err) })
          }
        }
      })
      .catch((err) => {
        if (!teardownCalled) {
          sig.set({ kind: 'error', error: toError(err) })
        }
      })

    return () => {
      if (teardownCalled) return
      teardownCalled = true
      if (subId) {
        this.subs.delete(subId)
        if (this.windowId) this._postUnsubscribe(this.windowId, subId)
      }
      this.refCount--
      if (this.refCount === 0) this._reset()
    }
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  private _ensureWindow(): Promise<string> {
    if (this.windowId !== null) return Promise.resolve(this.windowId)
    if (this.openPromise !== null) return this.openPromise
    this.openPromise = fetch(`${this.baseUrl}${MUX}/open`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({}),
    })
      .then(async (resp) => {
        if (!resp.ok) {
          this.openPromise = null
          throw new Error(`[@ship/sdk-client] mux/open failed: ${resp.status}`)
        }
        const data = (await resp.json()) as { window_id: string }
        this.windowId = data.window_id
        this.openPromise = null
        this._openStream(data.window_id)
        return data.window_id
      })
      .catch((err: unknown) => {
        this.openPromise = null
        throw err
      })
    return this.openPromise
  }

  private _openStream(windowId: string): void {
    // ESCtor is guaranteed non-null — _openStream is only reachable through
    // _ensureWindow, which is only called from subscribe after the ESCtor guard.
    const es = new this.ESCtor!(
      `${this.baseUrl}${MUX}/stream/${encodeURIComponent(windowId)}`,
    )
    this.streamEs = es

    es.addEventListener(
      'mux-data',
      ((ev: MessageEvent) => {
        let frame: { sub: string; value: unknown }
        try {
          frame = JSON.parse(ev.data as string) as { sub: string; value: unknown }
        } catch {
          return
        }
        const sig = this.subs.get(frame.sub)
        if (sig) sig.set({ kind: 'ok', value: frame.value as never })
      }) as EventListener,
    )

    es.addEventListener(
      'mux-close',
      ((ev: MessageEvent) => {
        let reason = 'unknown'
        try {
          reason =
            ((JSON.parse(ev.data as string) as { reason?: string }).reason) ?? 'unknown'
        } catch {
          /* ignore parse errors */
        }
        const error = new Error(`[@ship/sdk-client] mux stream closed: ${reason}`)
        for (const sig of this.subs.values()) sig.set({ kind: 'error', error })
        this.subs.clear()
        this._reset()
      }) as EventListener,
    )
  }

  /** Fire-and-forget POST to mux/unsubscribe. */
  private _postUnsubscribe(windowId: string, subscriptionId: string): void {
    void fetch(`${this.baseUrl}${MUX}/unsubscribe`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        window_id: windowId,
        subscription_id: subscriptionId,
      }),
    }).catch(() => {
      /* fire-and-forget; errors are safe to swallow */
    })
  }

  private _reset(): void {
    if (this.streamEs) {
      this.streamEs.close()
      this.streamEs = null
    }
    this.windowId = null
    this.openPromise = null
  }
}

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))
