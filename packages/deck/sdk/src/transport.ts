/**
 * Transport abstraction for ShipClient.
 *
 * A transport is responsible for two things:
 *   1. Request/response — `request(method, path, body?) -> Promise<unknown>`
 *      (payload already unwrapped from the {ok,data} envelope)
 *   2. Event streaming — `subscribeEvents(fn) -> unsubscribe`
 *
 * Three implementations are contemplated:
 *
 *   - DirectTransport (implemented): fetch + EventSource against a daemon
 *     running on loopback. Used by the Studio shell and by daemon-hosted
 *     component views that live in the same origin as the SDK.
 *
 *   - InjectedTransport (stub): forwards to a parent-injected `window.shipApi`
 *     object. Future use: nested iframes / test harnesses where the host
 *     already owns a ShipClient and wants to share its transport.
 *
 *   - BridgeTransport (stub): postMessage bridge for sandboxed iframes.
 *     Request/response correlation over structured cloning. The parent
 *     holds the real ShipClient and forwards calls over postMessage.
 *
 * Only DirectTransport is wired up in this cut; the other two are exported
 * as class stubs with the right shape so view authors and future phases
 * can implement them without re-deriving the interface.
 */

import { createShipConnectClient } from './connect.js'

export interface ShipEventEnvelope {
  id: string
  event_type: string
  entity_id: string
  actor_id?: string
  space_id?: string
  payload_json?: string
  created_at: string
}

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE'

export interface TransportRequestInit {
  method?: HttpMethod
  body?: unknown
  signal?: AbortSignal
  /** If true, return raw Response JSON rather than unwrapping {ok,data}. */
  raw?: boolean
}

export interface Transport {
  /**
   * Perform a request and return the (unwrapped) response data.
   * Implementations should unwrap the `{ok: true, data: T}` envelope when
   * present unless `init.raw` is set.
   */
  request<T = unknown>(path: string, init?: TransportRequestInit): Promise<T>

  /**
   * Subscribe to the daemon SSE event stream. Returns an unsubscribe fn.
   * Implementations are free to share a single underlying stream across
   * subscribers.
   */
  subscribeEvents(fn: (event: ShipEventEnvelope) => void): () => void

  /** Release any held resources (SSE connections, etc). */
  destroy(): void
}

// ── DirectTransport ─────────────────────────────────────────

export interface DirectTransportOptions {
  baseUrl?: string
  token?: string
  timeout?: number
}

export class DirectTransport implements Transport {
  private baseUrl: string
  private token: string | undefined
  private timeout: number
  private streamAbort: AbortController | null = null
  private listeners = new Set<(e: ShipEventEnvelope) => void>()

  constructor(options?: DirectTransportOptions) {
    this.baseUrl = options?.baseUrl ?? 'http://localhost:9315'
    this.token = options?.token
    this.timeout = options?.timeout ?? 10000
  }

  setToken(token: string | undefined): void {
    this.token = token
  }

  private headers(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' }
    if (this.token) h['Authorization'] = `Bearer ${this.token}`
    return h
  }

  async request<T = unknown>(path: string, init?: TransportRequestInit): Promise<T> {
    const url = path.startsWith('http')
      ? path
      : `${this.baseUrl}${path.startsWith('/api') ? '' : '/api'}${path}`
    const res = await fetch(url, {
      method: init?.method ?? 'GET',
      headers: this.headers(),
      body: init?.body !== undefined ? JSON.stringify(init.body) : undefined,
      signal: init?.signal ?? AbortSignal.timeout(this.timeout),
    })
    if (!res.ok) {
      const text = await res.text().catch(() => res.statusText)
      throw new Error(`Ship API ${res.status}: ${text}`)
    }
    // 204 / empty bodies
    if (res.status === 204) return undefined as T
    const json = (await res.json()) as unknown
    if (init?.raw) return json as T
    if (
      json &&
      typeof json === 'object' &&
      'ok' in (json as Record<string, unknown>) &&
      'data' in (json as Record<string, unknown>)
    ) {
      return (json as { data: T }).data
    }
    return json as T
  }

  subscribeEvents(fn: (event: ShipEventEnvelope) => void): () => void {
    this.listeners.add(fn)
    this.ensureStream()
    return () => {
      this.listeners.delete(fn)
      if (this.listeners.size === 0) this.closeStream()
    }
  }

  private ensureStream(): void {
    if (this.streamAbort) return
    const abort = new AbortController()
    this.streamAbort = abort
    void this.runStream(abort.signal)
  }

  /**
   * Tail the Connect `StreamService.Events` server-stream (the successor to
   * the retired `/api/v1/events/stream` SSE route). Each frame carries the
   * serialized `EventEnvelope` in `JsonResponse.json`. Reconnects with linear
   * backoff on drop — mirroring the old `EventSource` auto-reconnect — until
   * the last subscriber leaves and the signal aborts.
   */
  private async runStream(signal: AbortSignal): Promise<void> {
    const client = createShipConnectClient({
      baseUrl: this.baseUrl,
      ...(this.token ? { token: this.token } : {}),
    })
    let backoff = 1000
    while (!signal.aborted) {
      try {
        for await (const msg of client.stream.events({}, { signal })) {
          backoff = 1000
          try {
            const envelope = JSON.parse(msg.json) as ShipEventEnvelope
            this.listeners.forEach((l) => l(envelope))
          } catch {
            /* malformed frame */
          }
        }
      } catch {
        /* stream dropped — fall through to backoff + retry */
      }
      if (signal.aborted) return
      await new Promise((r) => setTimeout(r, backoff))
      backoff = Math.min(backoff * 2, 15000)
    }
  }

  private closeStream(): void {
    this.streamAbort?.abort()
    this.streamAbort = null
  }

  destroy(): void {
    this.closeStream()
    this.listeners.clear()
  }
}

// ── InjectedTransport (stub) ────────────────────────────────

/**
 * STUB — not wired up in Phase 6.
 *
 * Forwards all calls to a parent-injected `window.shipApi`-shaped object.
 * Intended for nested view-in-view scenarios where a child component
 * wants to reuse the host's already-authenticated client without
 * re-opening a DirectTransport.
 *
 * To implement: mirror each ShipClient method onto the injected target
 * and multiplex `onEvent` subscriptions through the host's single stream.
 */
export class InjectedTransport implements Transport {
  constructor(_target: unknown) {
    throw new Error('InjectedTransport is a stub; not implemented in Phase 6')
  }

  request<T = unknown>(_path: string, _init?: TransportRequestInit): Promise<T> {
    throw new Error('InjectedTransport.request not implemented')
  }

  subscribeEvents(_fn: (event: ShipEventEnvelope) => void): () => void {
    throw new Error('InjectedTransport.subscribeEvents not implemented')
  }

  destroy(): void {
    /* noop */
  }
}

// ── BridgeTransport (stub) ──────────────────────────────────

/**
 * STUB — not wired up in Phase 6.
 *
 * postMessage bridge between a sandboxed iframe (child) and its host.
 * The host holds the real ShipClient (with a DirectTransport) and
 * relays calls. Correlates requests by monotonically increasing seq.
 */
export class BridgeTransport implements Transport {
  constructor(_peer: Window) {
    throw new Error('BridgeTransport is a stub; not implemented in Phase 6')
  }

  request<T = unknown>(_path: string, _init?: TransportRequestInit): Promise<T> {
    throw new Error('BridgeTransport.request not implemented')
  }

  subscribeEvents(_fn: (event: ShipEventEnvelope) => void): () => void {
    throw new Error('BridgeTransport.subscribeEvents not implemented')
  }

  destroy(): void {
    /* noop */
  }
}
