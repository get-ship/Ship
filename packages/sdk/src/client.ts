/**
 * ShipClient — typed, capability-scoped client for the Ship daemon.
 *
 * Framework-agnostic. Works over direct fetch (same-origin) or
 * the bridge (cross-origin). Used by Studio, extensions, and
 * third-party apps.
 *
 * Security model:
 * - Every client authenticates with a scoped token
 * - Tokens declare capabilities (read:workspaces, write:jobs, etc.)
 * - The daemon enforces capabilities per-request
 * - Apps register via `ship register-app` to get a token
 * - Studio gets a full-access token from the daemon on startup
 */

import type { Workspace, JobRecord, Skill } from './types'

// ── Capabilities ────────────────────────────────────────────

/** Granular capability scopes. Apps declare what they need. */
export type Capability =
  // Read scopes
  | 'read:workspaces'
  | 'read:agents'
  | 'read:skills'
  | 'read:jobs'
  | 'read:events'
  | 'read:artifacts'
  | 'read:git'
  | 'read:settings'
  // Write scopes
  | 'write:agents'
  | 'write:skills'
  | 'write:jobs'
  | 'write:artifacts'
  | 'write:settings'
  // Runtime scopes
  | 'emit:events'
  | 'subscribe:events'
  // Custom namespace (extension-defined)
  | `namespace:${string}`

// ── Client options ──────────────────────────────────────────

export interface ShipClientOptions {
  /** Daemon base URL. Default: http://localhost:9315 */
  baseUrl?: string
  /** Auth token. Required for all requests. */
  token?: string
  /** Requested capabilities. Validated by daemon on first request. */
  capabilities?: Capability[]
  /** Request timeout in ms. Default: 10000 */
  timeout?: number
}

// ── Query / Subscription types ──────────────────────────────

export type QueryStatus = 'idle' | 'loading' | 'success' | 'error'

export interface QueryState<T> {
  data: T | undefined
  error: Error | undefined
  status: QueryStatus
  isLoading: boolean
  refetch: () => Promise<void>
}

type Listener<T> = (state: QueryState<T>) => void
type Unsubscribe = () => void

// ── SSE Event types ─────────────────────────────────────────

export interface ShipEvent {
  id: string
  event_type: string
  entity_id: string
  actor_id?: string
  workspace_id?: string
  payload_json?: string
  created_at: string
}

type EventListener = (event: ShipEvent) => void

// ── Client ──────────────────────────────────────────────────

export class ShipClient {
  private baseUrl: string
  private token: string | undefined
  private capabilities: Capability[]
  private timeout: number
  private cache = new Map<string, QueryState<unknown>>()
  private listeners = new Map<string, Set<Listener<unknown>>>()
  private eventSource: EventSource | null = null
  private eventListeners = new Set<EventListener>()
  private namespaceListeners = new Map<string, Set<EventListener>>()

  constructor(options?: ShipClientOptions) {
    this.baseUrl = options?.baseUrl ?? 'http://localhost:9315'
    this.token = options?.token
    this.capabilities = options?.capabilities ?? []
    this.timeout = options?.timeout ?? 10000
  }

  // ── Auth ────────────────────────────────────────────────

  /** Set the auth token. Called after registration or daemon handshake. */
  setToken(token: string): void {
    this.token = token
  }

  /** Try to read the token from the daemon's bootstrap endpoint. */
  async authenticate(): Promise<boolean> {
    try {
      const res = await fetch(`${this.baseUrl}/auth/handshake`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ capabilities: this.capabilities }),
      })
      if (!res.ok) return false
      const body = await res.json() as { token: string; granted: Capability[] }
      this.token = body.token
      return true
    } catch {
      return false
    }
  }

  /** Check if the daemon is reachable. */
  async isConnected(): Promise<boolean> {
    try {
      const res = await fetch(`${this.baseUrl}/bridge/health`, {
        signal: AbortSignal.timeout(2000),
      })
      return res.ok
    } catch {
      return false
    }
  }

  // ── Fetch ───────────────────────────────────────────────

  private headers(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' }
    if (this.token) h['Authorization'] = `Bearer ${this.token}`
    return h
  }

  async fetch<T = unknown>(path: string, init?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}/api${path}`
    const res = await fetch(url, {
      ...init,
      headers: { ...this.headers(), ...init?.headers },
      signal: init?.signal ?? AbortSignal.timeout(this.timeout),
    })
    if (!res.ok) {
      const text = await res.text().catch(() => res.statusText)
      throw new Error(`Ship API ${res.status}: ${text}`)
    }
    const body = await res.json() as { ok: boolean; data: T }
    return body.data
  }

  async get<T = unknown>(path: string): Promise<T> {
    return this.fetch<T>(path)
  }

  async post<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.fetch<T>(path, {
      method: 'POST',
      body: body !== undefined ? JSON.stringify(body) : undefined,
    })
  }

  async put<T = unknown>(path: string, body?: unknown): Promise<T> {
    return this.fetch<T>(path, {
      method: 'PUT',
      body: body !== undefined ? JSON.stringify(body) : undefined,
    })
  }

  async del<T = unknown>(path: string): Promise<T> {
    return this.fetch<T>(path, { method: 'DELETE' })
  }

  // ── Queries (framework-agnostic reactive cache) ─────────

  /**
   * Create a reactive query. Returns current state + subscribe for updates.
   * Inspired by TanStack Query core — same pattern, no framework dependency.
   */
  query<T>(key: string, fetcher: () => Promise<T>): QueryState<T> & { subscribe: (fn: Listener<T>) => Unsubscribe } {
    const existing = this.cache.get(key) as QueryState<T> | undefined

    const state: QueryState<T> = existing ?? {
      data: undefined,
      error: undefined,
      status: 'idle',
      isLoading: false,
      refetch: async () => { await run() },
    }

    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set())
    }

    const notify = () => {
      const fns = this.listeners.get(key)
      if (fns) fns.forEach((fn) => fn(state as QueryState<unknown>))
    }

    const run = async () => {
      state.status = 'loading'
      state.isLoading = true
      notify()

      try {
        state.data = await fetcher()
        state.error = undefined
        state.status = 'success'
      } catch (e) {
        state.error = e instanceof Error ? e : new Error(String(e))
        state.status = 'error'
      } finally {
        state.isLoading = false
        this.cache.set(key, state as QueryState<unknown>)
        notify()
      }
    }

    // Auto-fetch on first access
    if (state.status === 'idle') {
      void run()
    }

    return {
      ...state,
      subscribe: (fn: Listener<T>) => {
        const fns = this.listeners.get(key)!
        fns.add(fn as Listener<unknown>)
        return () => { fns.delete(fn as Listener<unknown>) }
      },
    }
  }

  /** Invalidate a cached query, triggering refetch for subscribers. */
  invalidate(key: string): void {
    const cached = this.cache.get(key)
    if (cached) {
      void cached.refetch()
    }
  }

  // ── SSE Events ──────────────────────────────────────────

  /** Subscribe to the daemon's SSE event stream. */
  connectEvents(): void {
    if (this.eventSource) return

    const url = `${this.baseUrl}/api/runtime/events`
    this.eventSource = new EventSource(url)

    this.eventSource.addEventListener('ship.event', (e: MessageEvent) => {
      try {
        const event = JSON.parse(e.data as string) as ShipEvent
        // Broadcast to all listeners
        this.eventListeners.forEach((fn) => fn(event))
        // Broadcast to namespace listeners
        for (const [ns, fns] of this.namespaceListeners) {
          if (event.event_type.startsWith(ns)) {
            fns.forEach((fn) => fn(event))
          }
        }
        // Auto-invalidate relevant queries
        this.autoInvalidate(event)
      } catch { /* malformed */ }
    })

    this.eventSource.onerror = () => {
      // Reconnection is automatic with EventSource
    }
  }

  /** Subscribe to all events. */
  onEvent(fn: EventListener): Unsubscribe {
    this.eventListeners.add(fn)
    return () => { this.eventListeners.delete(fn) }
  }

  /** Subscribe to events in a specific namespace. */
  onNamespace(namespace: string, fn: EventListener): Unsubscribe {
    if (!this.namespaceListeners.has(namespace)) {
      this.namespaceListeners.set(namespace, new Set())
    }
    const fns = this.namespaceListeners.get(namespace)!
    fns.add(fn)
    return () => { fns.delete(fn) }
  }

  /** Disconnect from SSE. */
  disconnectEvents(): void {
    this.eventSource?.close()
    this.eventSource = null
  }

  private autoInvalidate(event: ShipEvent): void {
    const t = event.event_type
    if (t.startsWith('workspace.')) this.invalidate('workspaces')
    if (t.startsWith('session.')) this.invalidate('sessions')
    if (t.startsWith('job.')) this.invalidate('jobs')
    if (t.startsWith('mesh.') || t.startsWith('actor.')) this.invalidate('agents')
  }

  // ── Convenience methods (typed) ─────────────────────────

  workspaces() {
    return this.query<{ workspaces: Workspace[] }>('workspaces', () =>
      this.get('/runtime/workspaces'),
    )
  }

  jobs() {
    return this.query<{ jobs: JobRecord[] }>('jobs', () =>
      this.get('/runtime/jobs'),
    )
  }

  // ── Cleanup ─────────────────────────────────────────────

  destroy(): void {
    this.disconnectEvents()
    this.cache.clear()
    this.listeners.clear()
    this.eventListeners.clear()
    this.namespaceListeners.clear()
  }
}
