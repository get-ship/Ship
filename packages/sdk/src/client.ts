/**
 * ShipClient — typed, capability-scoped client for the Ship daemon.
 *
 * Framework-agnostic. Uses @tanstack/query-core for caching internally.
 * Transport is pluggable (see ./transport.ts). Default = DirectTransport.
 *
 * This client is the single entry point for every caller — the Studio
 * shell, trusted web component views (via ViewMount injection), and
 * (future) sandboxed iframe extensions via BridgeTransport.
 *
 * Usage:
 *   const ship = new ShipClient({ token: '...' })
 *   const q = ship.query('/runtime/workspaces')
 *   q.subscribe(state => console.log(state.data))
 *
 *   // createShipApi-style surface (for view authors):
 *   await ship.getJobs()
 *   const unsub = ship.onEvent(e => console.log(e))
 */

import { QueryClient, QueryObserver } from '@tanstack/query-core'
import {
  DirectTransport,
  type Transport,
  type ShipEventEnvelope,
} from './transport'

// ── Capabilities ────────────────────────────────────────────

export type Capability =
  | 'read:workspaces' | 'read:agents' | 'read:skills' | 'read:jobs'
  | 'read:events' | 'read:artifacts' | 'read:git' | 'read:settings'
  | 'write:agents' | 'write:skills' | 'write:jobs'
  | 'write:artifacts' | 'write:settings'
  | 'emit:events' | 'subscribe:events'
  | `namespace:${string}`
  | '*'

// ── Options ─────────────────────────────────────────────────

export interface ShipClientOptions {
  /** Daemon base URL. Default: http://localhost:9315. Ignored if transport is provided. */
  baseUrl?: string
  /** Auth token. Ignored if transport is provided. */
  token?: string
  /** Requested capabilities. */
  capabilities?: Capability[]
  /** Request timeout in ms. Ignored if transport is provided. */
  timeout?: number
  /** Default stale time in ms. Default: 5000 */
  staleTime?: number
  /** Active workspace branch id — surfaced as `client.activeWorkspace`. */
  activeWorkspace?: string
  /** Custom transport. If omitted, a DirectTransport is constructed. */
  transport?: Transport
}

// ── Query types ─────────────────────────────────────────────

export interface ShipQueryState<T> {
  data: T | undefined
  error: Error | null
  isLoading: boolean
  isSuccess: boolean
  isError: boolean
  refetch: () => void
}

type ShipQueryListener<T> = (state: ShipQueryState<T>) => void
type Unsubscribe = () => void

export interface ShipQuery<T> extends ShipQueryState<T> {
  subscribe: (fn: ShipQueryListener<T>) => Unsubscribe
}

// ── SSE Event types ─────────────────────────────────────────

export type ShipEvent = ShipEventEnvelope

type EventListener = (event: ShipEvent) => void

// ── Client ──────────────────────────────────────────────────

export class ShipClient {
  private transport: Transport
  private capabilities: Capability[]
  private staleTime: number
  private queryClient: QueryClient
  private eventListeners = new Set<EventListener>()
  private namespaceListeners = new Map<string, Set<EventListener>>()
  private streamUnsub: (() => void) | null = null
  private token: string | undefined
  private baseUrl: string

  /** Active workspace branch id (surfaced to view authors). */
  public activeWorkspace: string | undefined

  constructor(options?: ShipClientOptions) {
    this.baseUrl = options?.baseUrl ?? 'http://localhost:9315'
    this.token = options?.token
    this.transport = options?.transport ?? new DirectTransport({
      baseUrl: this.baseUrl,
      token: this.token,
      timeout: options?.timeout,
    })
    this.capabilities = options?.capabilities ?? []
    this.staleTime = options?.staleTime ?? 5000
    this.activeWorkspace = options?.activeWorkspace

    this.queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: this.staleTime,
          gcTime: 5 * 60 * 1000,
          retry: 1,
          refetchOnWindowFocus: false,
        },
      },
    })
  }

  // ── Auth ────────────────────────────────────────────────

  setToken(token: string): void {
    this.token = token
    const t = this.transport as unknown as { setToken?: (t: string) => void }
    if (typeof t.setToken === 'function') t.setToken(token)
  }

  getToken(): string | null {
    return this.token ?? null
  }

  async authenticate(): Promise<boolean> {
    try {
      const res = await fetch(`${this.baseUrl}/auth/handshake`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ capabilities: this.capabilities }),
      })
      if (!res.ok) return false
      const body = (await res.json()) as { token: string; granted: Capability[] }
      this.setToken(body.token)
      return true
    } catch {
      return false
    }
  }

  async isConnected(): Promise<boolean> {
    try {
      await this.transport.request('/runtime/views', {
        signal: AbortSignal.timeout(2000),
      })
      return true
    } catch {
      return false
    }
  }

  // ── HTTP helpers ────────────────────────────────────────

  async fetch<T = unknown>(path: string, init?: RequestInit): Promise<T> {
    const method = (init?.method ?? 'GET').toUpperCase() as 'GET' | 'POST' | 'PUT' | 'DELETE'
    const body = init?.body !== undefined && typeof init.body === 'string'
      ? JSON.parse(init.body as string) as unknown
      : init?.body as unknown
    return this.transport.request<T>(path, { method, body, signal: init?.signal ?? undefined })
  }

  get<T = unknown>(path: string): Promise<T> {
    return this.transport.request<T>(path, { method: 'GET' })
  }

  post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.transport.request<T>(path, { method: 'POST', body })
  }

  put<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.transport.request<T>(path, { method: 'PUT', body })
  }

  del<T = unknown>(path: string): Promise<T> {
    return this.transport.request<T>(path, { method: 'DELETE' })
  }

  // ── Queries ─────────────────────────────────────────────

  query<T>(path: string, options?: { staleTime?: number }): ShipQuery<T> {
    const queryKey = ['ship', path]

    const observer = new QueryObserver<T, Error>(this.queryClient, {
      queryKey,
      queryFn: () => this.get<T>(path),
      staleTime: options?.staleTime ?? this.staleTime,
    })

    const currentResult = observer.getCurrentResult()

    const query: ShipQuery<T> = {
      data: currentResult.data,
      error: currentResult.error,
      isLoading: currentResult.isLoading,
      isSuccess: currentResult.isSuccess,
      isError: currentResult.isError,
      refetch: () => { void this.queryClient.invalidateQueries({ queryKey }) },
      subscribe: (fn: ShipQueryListener<T>): Unsubscribe => {
        return observer.subscribe((result) => {
          query.data = result.data
          query.error = result.error
          query.isLoading = result.isLoading
          query.isSuccess = result.isSuccess
          query.isError = result.isError
          fn(query)
        })
      },
    }

    return query
  }

  async mutate<T = unknown>(
    method: 'POST' | 'PUT' | 'DELETE',
    path: string,
    body?: unknown,
    invalidateKeys?: string[],
  ): Promise<T> {
    const result = method === 'DELETE'
      ? await this.del<T>(path)
      : method === 'PUT'
        ? await this.put<T>(path, body)
        : await this.post<T>(path, body)

    if (invalidateKeys) {
      for (const key of invalidateKeys) {
        void this.queryClient.invalidateQueries({ queryKey: ['ship', key] })
      }
    }

    return result
  }

  invalidate(path: string): void {
    void this.queryClient.invalidateQueries({ queryKey: ['ship', path] })
  }

  invalidatePrefix(prefix: string): void {
    void this.queryClient.invalidateQueries({
      predicate: (q) => {
        const key = q.queryKey as string[]
        return key[0] === 'ship' && key[1]?.startsWith(prefix)
      },
    })
  }

  // ── SSE Events ──────────────────────────────────────────

  /** Open the event stream. Idempotent. */
  connectEvents(): void {
    if (this.streamUnsub) return
    this.streamUnsub = this.transport.subscribeEvents((event) => {
      this.eventListeners.forEach((fn) => fn(event))
      for (const [ns, fns] of this.namespaceListeners) {
        if (event.event_type.startsWith(ns)) fns.forEach((fn) => fn(event))
      }
      this.autoInvalidate(event)
    })
  }

  /** Subscribe to all events. Opens the stream lazily. Returns unsubscribe. */
  onEvent(fn: EventListener): Unsubscribe {
    this.eventListeners.add(fn)
    this.connectEvents()
    return () => {
      this.eventListeners.delete(fn)
      this.maybeCloseStream()
    }
  }

  onNamespace(namespace: string, fn: EventListener): Unsubscribe {
    if (!this.namespaceListeners.has(namespace)) {
      this.namespaceListeners.set(namespace, new Set())
    }
    const fns = this.namespaceListeners.get(namespace)!
    fns.add(fn)
    this.connectEvents()
    return () => {
      fns.delete(fn)
      this.maybeCloseStream()
    }
  }

  disconnectEvents(): void {
    this.streamUnsub?.()
    this.streamUnsub = null
  }

  private maybeCloseStream(): void {
    if (this.eventListeners.size > 0) return
    for (const fns of this.namespaceListeners.values()) {
      if (fns.size > 0) return
    }
    this.disconnectEvents()
  }

  private autoInvalidate(event: ShipEvent): void {
    const t = event.event_type
    if (t.startsWith('workspace.')) this.invalidatePrefix('/runtime/workspaces')
    if (t.startsWith('session.')) {
      this.invalidatePrefix('/runtime/workspaces')
      this.invalidatePrefix('/runtime/sessions')
    }
    if (t.startsWith('job.')) this.invalidatePrefix('/runtime/jobs')
    if (t.startsWith('mesh.') || t.startsWith('actor.')) {
      this.invalidatePrefix('/runtime/agents')
    }
  }

  // ── View-author surface (ex-createShipApi) ──────────────

  /** List all jobs. */
  async getJobs(): Promise<unknown[]> {
    const data = await this.get<{ jobs: unknown[] }>('/runtime/jobs')
    return data.jobs
  }

  /** Fetch a single job by id. Returns the raw envelope for back-compat. */
  getJob(jobId: string): Promise<unknown> {
    return this.transport.request(
      `/runtime/jobs/${encodeURIComponent(jobId)}`,
      { method: 'GET', raw: true },
    )
  }

  /** Create a new job. Returns the raw response envelope for back-compat. */
  createJob(params: Record<string, unknown>): Promise<unknown> {
    return this.transport.request('/runtime/jobs', {
      method: 'POST',
      body: params,
      raw: true,
    })
  }

  /** Emit a ship event. Returns the raw response envelope. */
  emitEvent(eventType: string, entityId: string, payload: unknown): Promise<unknown> {
    return this.transport.request('/events/emit', {
      method: 'POST',
      body: { event_type: eventType, entity_id: entityId, payload },
      raw: true,
    })
  }

  /** List session files for a workspace. */
  getSessionFiles(workspaceId: string): Promise<unknown> {
    return this.transport.request(
      `/workspaces/${encodeURIComponent(workspaceId)}/session-files`,
      { method: 'GET', raw: true },
    )
  }

  /** Read a single session file. */
  readFile(workspaceId: string, path: string): Promise<unknown> {
    return this.transport.request(
      `/workspaces/${encodeURIComponent(workspaceId)}/session-files/${path}`,
      { method: 'GET', raw: true },
    )
  }

  // ── Cleanup ─────────────────────────────────────────────

  destroy(): void {
    this.disconnectEvents()
    this.transport.destroy()
    this.queryClient.clear()
    this.eventListeners.clear()
    this.namespaceListeners.clear()
  }
}
