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
import { DirectTransport, type Transport } from './transport'

// ── Capabilities ────────────────────────────────────────────

export type Capability =
  | 'read:workspaces' | 'read:agents' | 'read:skills' | 'read:jobs'
  | 'read:events' | 'read:artifacts' | 'read:git' | 'read:settings'
  | 'write:agents' | 'write:skills' | 'write:jobs'
  | 'write:artifacts' | 'write:settings'
  | 'emit:events'
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

// ── Client ──────────────────────────────────────────────────

export class ShipClient {
  private transport: Transport
  private capabilities: Capability[]
  private staleTime: number
  private queryClient: QueryClient
  private token: string | undefined
  private baseUrl: string
  private timeout: number

  /** Active workspace branch id (surfaced to view authors). */
  public activeWorkspace: string | undefined

  constructor(options?: ShipClientOptions) {
    this.baseUrl = options?.baseUrl ?? 'http://localhost:9315'
    this.token = options?.token
    this.timeout = options?.timeout ?? 10000
    this.transport = options?.transport ?? new DirectTransport({
      baseUrl: this.baseUrl,
      token: this.token,
      timeout: this.timeout,
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

  /**
   * Try server-side attestation: exchange a getship.dev-signed JWT for a
   * daemon session token. Returns the session token or null if attestation
   * isn't available (daemon doesn't trust issuer, user not signed in, etc.).
   *
   * `jwt` should come from getship.dev's /api/auth/daemon-attest endpoint.
   */
  async serverAttest(jwt: string): Promise<string | null> {
    try {
      const res = await fetch(`${this.baseUrl}/auth/server-attest`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ jwt }),
      })
      if (!res.ok) return null
      const body = await res.json() as { ok: boolean; session_token?: string }
      return body.session_token ?? null
    } catch {
      return null
    }
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
      await this.transport.request('/runtime/views', {
        signal: AbortSignal.timeout(2000),
      })
      return true
    } catch {
      return false
    }
  }

  // ── HTTP helpers ────────────────────────────────────────

  /** Callback fired when the daemon returns 401/403. Used to trigger re-pairing. */
  private onAuthFailure: (() => void) | null = null

  /** Register a callback for auth failures (401/403). Called at most once per failure. */
  setAuthFailureHandler(cb: (() => void) | null): void {
    this.onAuthFailure = cb
  }

  private headers(): Record<string, string> {
    return this.token ? { Authorization: `Bearer ${this.token}` } : {}
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

  // ── Collections ────────────────────────────────────────

  /** List registered collection schemas. */
  async listCollections(): Promise<unknown[]> {
    const data = await this.get<{ data: unknown[] }>('/collections')
    return data.data ?? []
  }

  /** Get a collection schema definition. */
  getCollectionSchema(collection: string): Promise<unknown> {
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/schema`,
      { method: 'GET', raw: true },
    )
  }

  /** Create an item in a collection. */
  createCollectionItem(collection: string, data: Record<string, unknown>, id?: string): Promise<unknown> {
    const body: Record<string, unknown> = { data }
    if (id) body.id = id
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/items`,
      { method: 'POST', body, raw: true },
    )
  }

  /** Get a single collection item. */
  getCollectionItem(collection: string, itemId: string): Promise<unknown> {
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/items/${encodeURIComponent(itemId)}`,
      { method: 'GET', raw: true },
    )
  }

  /** Update a collection item (partial merge). */
  updateCollectionItem(collection: string, itemId: string, data: Record<string, unknown>): Promise<unknown> {
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/items/${encodeURIComponent(itemId)}`,
      { method: 'PUT', body: { data }, raw: true },
    )
  }

  /** Delete a collection item. */
  deleteCollectionItem(collection: string, itemId: string): Promise<unknown> {
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/items/${encodeURIComponent(itemId)}`,
      { method: 'DELETE', raw: true },
    )
  }

  /**
   * Query items in a collection with filters, sorting, and pagination.
   * Filters use Django-style `field__op` keys:
   *   { status: 'active', priority__gte: 3 }
   * Operators: __ne, __gt, __gte, __lt, __lte, __in, __nin, __contains, __exists
   */
  queryCollection(collection: string, params: {
    filter?: Record<string, unknown>,
    sort?: Record<string, number>,
    limit?: number,
    offset?: number,
  } = {}): Promise<unknown> {
    return this.transport.request(
      `/collections/${encodeURIComponent(collection)}/query`,
      { method: 'POST', body: { collection, ...params }, raw: true },
    )
  }

  // ── Cleanup ─────────────────────────────────────────────

  destroy(): void {
    this.transport.destroy()
    this.queryClient.clear()
  }
}
