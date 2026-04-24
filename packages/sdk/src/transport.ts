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

export interface ShipEventEnvelope {
  id: string
  event_type: string
  entity_id: string
  actor_id?: string
  workspace_id?: string
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
  private eventSource: EventSource | null = null
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
      if (this.listeners.size === 0) {
        this.eventSource?.close()
        this.eventSource = null
      }
    }
  }

  private ensureStream(): void {
    if (this.eventSource) return
    let url = `${this.baseUrl}/api/events/stream`
    if (this.token) url += `?token=${encodeURIComponent(this.token)}`
    this.eventSource = new EventSource(url)
    this.eventSource.addEventListener('ship.event', (e: MessageEvent) => {
      try {
        const envelope = JSON.parse(e.data as string) as ShipEventEnvelope
        this.listeners.forEach((l) => l(envelope))
      } catch {
        /* malformed */
      }
    })
  }

  destroy(): void {
    this.eventSource?.close()
    this.eventSource = null
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
 *
 * Today, `ViewHost.tsx` implements an ad-hoc version of this protocol
 * inline. The plan is to move that protocol into this class and have
 * both sides speak it through a `ShipClient`.
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
