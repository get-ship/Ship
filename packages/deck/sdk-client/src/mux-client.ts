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
// mux-close or EventSource onerror surfaces an error to every live signal and
// schedules a reconnect with exponential backoff (1s → 2s → 4s … capped at
// 30s). 4xx responses on mux/subscribe are treated as permanent failures —
// those subscriptions are not retried on reconnect.
//
// Lifecycle:
//   - subscribe()  increments refCount; teardown decrements it.
//   - refCount → 0 cancels any pending reconnect, closes the EventSource +
//     drops the window so the next subscribe re-opens cleanly.

import { type LiveSignal } from '@ship/sui-runtime'
import { DECK_PREFIX } from './constants'

// Derived companion to deck's DECK_PREFIX. Imported from `./constants` rather
// than `./index` to avoid a `mux-client ↔ index` import cycle.
const MUX = `${DECK_PREFIX}/mux`

/** Per-subscription record. Survives stream resets so the client can
 *  re-subscribe all non-permanent registrations after reconnecting. */
interface RegEntry {
  domain: string
  op: string
  args: unknown
  sig: LiveSignal<unknown>
  /** Server-assigned subscription ID; null when not yet subscribed or after stream loss. */
  subId: string | null
  teardownCalled: boolean
  /** True when a 4xx was returned — this registration is never reconnected. */
  permanent: boolean
}

export class MuxClient {
  private windowId: string | null = null
  private streamEs: InstanceType<typeof globalThis.EventSource> | null = null
  private openPromise: Promise<string> | null = null
  /** Maps server-assigned subId → local reg symbol (for mux-data frame routing). */
  private readonly subs = new Map<string, symbol>()
  /** All active registrations keyed by a local symbol. Survives stream resets. */
  private readonly regs = new Map<symbol, RegEntry>()
  private refCount = 0

  // Exponential backoff — disabled when refCount reaches 0.
  private reconnTimer: ReturnType<typeof setTimeout> | null = null
  private reconnAttempt = 0
  private static readonly BASE_MS = 1_000
  private static readonly MAX_MS  = 30_000

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
      sig.set({ kind: 'error', error: new Error('[@ship/sdk-client] no EventSource available') })
      return () => {}
    }

    this.refCount++
    const key = Symbol()
    const reg: RegEntry = {
      domain, op, args,
      sig: sig as LiveSignal<unknown>,
      subId: null, teardownCalled: false, permanent: false,
    }
    this.regs.set(key, reg)

    // Eagerly start the open so parallel subscribes reuse the same window.
    void this._ensureWindow()
      .then((windowId) => {
        if (reg.teardownCalled) return
        return this._doSubscribeReg(windowId, key, reg)
      })
      .catch((err) => {
        if (reg.teardownCalled) return
        reg.sig.set({ kind: 'error', error: toError(err) })
        // Daemon unreachable on first open — schedule a backoff retry.
        this._scheduleReconnect()
      })

    return () => {
      if (reg.teardownCalled) return
      reg.teardownCalled = true
      this.regs.delete(key)
      if (reg.subId) {
        this.subs.delete(reg.subId)
        if (this.windowId) this._postUnsubscribe(this.windowId, reg.subId)
      }
      this.refCount--
      if (this.refCount === 0) {
        this._cancelReconnect()
        this._reset()
      }
    }
  }

  // ── Private helpers ────────────────────────────────────────────────────────

  /** POST mux/subscribe for one registration. 4xx → permanent (no reconnect). */
  private async _doSubscribeReg(windowId: string, key: symbol, reg: RegEntry): Promise<void> {
    try {
      const resp = await fetch(`${this.baseUrl}${MUX}/subscribe`, {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ window_id: windowId, domain: reg.domain, op: reg.op, args: reg.args ?? {} }),
      })
      if (!resp.ok) {
        // 4xx = op not found or auth failure — permanent, do not reconnect.
        if (resp.status >= 400 && resp.status < 500) reg.permanent = true
        if (!reg.teardownCalled) {
          reg.sig.set({
            kind: 'error',
            error: new Error(`[@ship/sdk-client] mux/subscribe failed: ${resp.status}`),
          })
        }
        return
      }
      const data = (await resp.json()) as { subscription_id: string; initial_value: unknown }
      if (reg.teardownCalled) {
        // Torn down mid-round-trip — clean up server-side.
        this._postUnsubscribe(windowId, data.subscription_id)
        return
      }
      reg.subId = data.subscription_id
      this.subs.set(data.subscription_id, key)
      reg.sig.set({ kind: 'ok', value: data.initial_value as never })
    } catch (err) {
      if (!reg.teardownCalled) {
        reg.sig.set({ kind: 'error', error: toError(err) })
      }
    }
  }

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
        } catch { return }
        const regKey = this.subs.get(frame.sub)
        if (regKey !== undefined) {
          const reg = this.regs.get(regKey)
          if (reg) reg.sig.set({ kind: 'ok', value: frame.value as never })
        }
      }) as EventListener,
    )

    es.addEventListener(
      'mux-close',
      ((ev: MessageEvent) => {
        let reason = 'unknown'
        try {
          reason = ((JSON.parse(ev.data as string) as { reason?: string }).reason) ?? 'unknown'
        } catch { /* ignore parse errors */ }
        this._handleStreamLoss(new Error(`[@ship/sdk-client] mux stream closed: ${reason}`))
      }) as EventListener,
    )

    // Take ownership of reconnection: close the EventSource on error (preventing
    // the browser's built-in reconnect loop which has no backoff) and schedule
    // our own retry with exponential backoff. addEventListener is used over
    // es.onerror so the same path fires in both real browser EventSource and
    // the FakeEventSource used in tests (which routes through the listeners map).
    es.addEventListener('error', () => {
      if (this.streamEs !== es) return
      this._handleStreamLoss(new Error('[@ship/sdk-client] mux stream: connection lost'))
    })
  }

  /** Set all non-permanent active sigs to error, close the stream, schedule reconnect. */
  private _handleStreamLoss(error: Error): void {
    for (const reg of this.regs.values()) {
      if (!reg.teardownCalled && !reg.permanent) {
        reg.subId = null
        reg.sig.set({ kind: 'error', error })
      }
    }
    this.subs.clear()
    this._reset()
    this._scheduleReconnect()
  }

  private _scheduleReconnect(): void {
    if (this.reconnTimer !== null) return  // already scheduled
    // Skip if no reconnectable registrations remain.
    const hasPending = Array.from(this.regs.values()).some(
      (r) => !r.teardownCalled && !r.permanent,
    )
    if (!hasPending) return
    const delay = Math.min(
      MuxClient.BASE_MS * (1 << Math.min(this.reconnAttempt, 15)),
      MuxClient.MAX_MS,
    )
    this.reconnAttempt++
    this.reconnTimer = setTimeout(() => {
      this.reconnTimer = null
      void this._doReconnect()
    }, delay)
  }

  private async _doReconnect(): Promise<void> {
    const pending = Array.from(this.regs.entries()).filter(
      ([, r]) => !r.teardownCalled && !r.permanent && r.subId === null,
    )
    if (pending.length === 0) { this.reconnAttempt = 0; return }

    let windowId: string
    try {
      windowId = await this._ensureWindow()
    } catch {
      this._scheduleReconnect()
      return
    }

    let allOk = true
    for (const [key, reg] of pending) {
      if (reg.teardownCalled) continue
      try {
        await this._doSubscribeReg(windowId, key, reg)
        if (reg.subId === null && !reg.permanent) allOk = false
      } catch {
        allOk = false
      }
    }
    if (allOk) {
      this.reconnAttempt = 0
    } else {
      this._scheduleReconnect()
    }
  }

  private _cancelReconnect(): void {
    if (this.reconnTimer !== null) {
      clearTimeout(this.reconnTimer)
      this.reconnTimer = null
    }
    this.reconnAttempt = 0
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
