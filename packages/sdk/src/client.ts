/**
 * ShipClient — typed, capability-scoped client for the Ship daemon.
 *
 * Framework-agnostic. Uses @tanstack/query-core for caching internally.
 * Works over direct fetch with CORS + token auth.
 *
 * Usage:
 *   const ship = new ShipClient({ token: '...' })
 *   const q = ship.query('/runtime/workspaces')
 *   q.subscribe(state => console.log(state.data))
 */

import { QueryClient, QueryObserver } from '@tanstack/query-core'

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
  /** Daemon base URL. Default: http://localhost:9315 */
  baseUrl?: string
  /** Auth token. */
  token?: string
  /** Requested capabilities. */
  capabilities?: Capability[]
  /** Request timeout in ms. Default: 10000 */
  timeout?: number
  /** Default stale time in ms. Default: 5000 */
  staleTime?: number
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

// ── Pairing types ───────────────────────────────────────────

export interface PairInit {
  code: string
  claim_token: string
  code_expires_in_secs: number
  dev_mode: boolean
}

export type PairStatus =
  | { status: 'pending' }
  | { status: 'approved'; session_token: string }
  | { status: 'expired' }
  | { status: 'unknown' }

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
  private staleTime: number
  private queryClient: QueryClient
  private eventSource: EventSource | null = null
  private eventListeners = new Set<EventListener>()
  private namespaceListeners = new Map<string, Set<EventListener>>()

  constructor(options?: ShipClientOptions) {
    this.baseUrl = options?.baseUrl ?? 'http://localhost:9315'
    this.token = options?.token
    this.capabilities = options?.capabilities ?? []
    this.timeout = options?.timeout ?? 10000
    this.staleTime = options?.staleTime ?? 5000

    this.queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: this.staleTime,
          gcTime: 5 * 60 * 1000, // 5 min garbage collection
          retry: 1,
          refetchOnWindowFocus: false,
        },
      },
    })
  }

  // ── Auth ────────────────────────────────────────────────

  setToken(token: string): void {
    this.token = token
  }

  getToken(): string | null {
    return this.token ?? null
  }

  /** Start a new pairing flow. Returns the code the user must paste into their CLI. */
  async pairInit(clientLabel?: string): Promise<PairInit | null> {
    try {
      const res = await fetch(`${this.baseUrl}/auth/pair/init`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ client_label: clientLabel ?? null }),
      })
      if (!res.ok) return null
      return await res.json() as PairInit
    } catch {
      return null
    }
  }

  /** Poll for pairing approval. Returns the session token when approved. */
  async pairStatus(claimToken: string): Promise<PairStatus> {
    try {
      const res = await fetch(`${this.baseUrl}/auth/pair/status?claim_token=${encodeURIComponent(claimToken)}`)
      if (!res.ok) return { status: 'unknown' }
      return await res.json() as PairStatus
    } catch {
      return { status: 'unknown' }
    }
  }

  async isConnected(): Promise<boolean> {
    try {
      const res = await fetch(`${this.baseUrl}/api/runtime/views`, {
        headers: this.headers(),
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

  /** Callback fired when the daemon returns 401/403. Used to trigger re-pairing. */
  private onAuthFailure: (() => void) | null = null

  /** Register a callback for auth failures (401/403). Called at most once per failure. */
  setAuthFailureHandler(cb: (() => void) | null): void {
    this.onAuthFailure = cb
  }

  async fetch<T = unknown>(path: string, init?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}/api${path}`
    const res = await fetch(url, {
      ...init,
      headers: { ...this.headers(), ...init?.headers },
      signal: init?.signal ?? AbortSignal.timeout(this.timeout),
    })
    if (res.status === 401 || res.status === 403) {
      // Notify the app so it can clear the cached token and re-pair.
      try { this.onAuthFailure?.() } catch { /* ignore */ }
      const text = await res.text().catch(() => res.statusText)
      throw new Error(`Ship API ${res.status}: ${text}`)
    }
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

  // ── Queries ─────────────────────────────────────────────

  /**
   * Create a reactive query with built-in caching.
   *
   * Deduplicates concurrent requests. Stale-while-revalidate.
   * Auto-invalidated by SSE events. Subscribe from any framework.
   */
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

  /**
   * Execute a mutation — write operation that invalidates related queries.
   */
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

  /** Invalidate a query by path, forcing refetch on next access. */
  invalidate(path: string): void {
    void this.queryClient.invalidateQueries({ queryKey: ['ship', path] })
  }

  /** Invalidate all queries matching a prefix. */
  invalidatePrefix(prefix: string): void {
    void this.queryClient.invalidateQueries({
      predicate: (q) => {
        const key = q.queryKey as string[]
        return key[0] === 'ship' && key[1]?.startsWith(prefix)
      },
    })
  }

  // ── SSE Events ──────────────────────────────────────────

  connectEvents(): void {
    if (this.eventSource) return

    let url = `${this.baseUrl}/api/runtime/events`
    // EventSource doesn't support custom headers — pass token as query param
    if (this.token) url += `?token=${encodeURIComponent(this.token)}`
    this.eventSource = new EventSource(url)

    this.eventSource.addEventListener('ship.event', (e: MessageEvent) => {
      try {
        const event = JSON.parse(e.data as string) as ShipEvent
        this.eventListeners.forEach((fn) => fn(event))
        for (const [ns, fns] of this.namespaceListeners) {
          if (event.event_type.startsWith(ns)) {
            fns.forEach((fn) => fn(event))
          }
        }
        this.autoInvalidate(event)
      } catch { /* malformed */ }
    })
  }

  onEvent(fn: EventListener): Unsubscribe {
    this.eventListeners.add(fn)
    return () => { this.eventListeners.delete(fn) }
  }

  onNamespace(namespace: string, fn: EventListener): Unsubscribe {
    if (!this.namespaceListeners.has(namespace)) {
      this.namespaceListeners.set(namespace, new Set())
    }
    const fns = this.namespaceListeners.get(namespace)!
    fns.add(fn)
    return () => { fns.delete(fn) }
  }

  disconnectEvents(): void {
    this.eventSource?.close()
    this.eventSource = null
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

  // ── Cleanup ─────────────────────────────────────────────

  destroy(): void {
    this.disconnectEvents()
    this.queryClient.clear()
    this.eventListeners.clear()
    this.namespaceListeners.clear()
  }
}
