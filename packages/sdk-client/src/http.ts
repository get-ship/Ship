// @ship/sdk-client/http — REST helpers for the SDK's mutation paths.
//
// All helpers are cookie-bound (`credentials: 'same-origin'`) so the
// daemon auth middleware sees the browser session. Non-2xx responses
// surface the daemon's JSON error envelope (or status line as fallback);
// 204s resolve to `undefined`; JSON bodies parse via `Response.json()`.

/**
 * Make a JSON-bodied request. `body === undefined` skips the body —
 * which is what DELETE expects.
 */
export async function mutateJson<T>(
  baseUrl: string,
  method: 'POST' | 'PUT' | 'PATCH' | 'DELETE',
  path: string,
  body?: unknown,
): Promise<T> {
  const init: RequestInit = { method, credentials: 'same-origin' }
  if (body !== undefined) {
    init.headers = { 'content-type': 'application/json' }
    init.body = JSON.stringify(body)
  }
  return throwOnError<T>(await fetch(`${baseUrl}${path}`, init), method, path)
}

/**
 * Make a multipart/form-data request. The browser sets the boundary —
 * do NOT set `content-type` manually.
 */
export async function mutateForm<T>(
  baseUrl: string,
  path: string,
  body: FormData,
): Promise<T> {
  const resp = await fetch(`${baseUrl}${path}`, {
    method: 'POST',
    credentials: 'same-origin',
    body,
  })
  return throwOnError<T>(resp, 'POST', path)
}

/**
 * GET-style body fetch — returns raw text / blob / parsed JSON depending
 * on `as`. Used by `artifacts.get` for preview panes and binary
 * downloads.
 */
export async function fetchBody<T>(
  baseUrl: string,
  path: string,
  as: 'text' | 'blob' | 'json',
): Promise<T> {
  const resp = await fetch(`${baseUrl}${path}`, { credentials: 'same-origin' })
  if (!resp.ok) {
    let detail = `${resp.status} ${resp.statusText}`
    try {
      const t = await resp.text()
      if (t) detail = t
    } catch { /* keep status-line fallback */ }
    throw new Error(`[ship-sdk-client] GET ${path} failed: ${detail}`)
  }
  if (as === 'text') return (await resp.text()) as unknown as T
  if (as === 'blob') return (await resp.blob()) as unknown as T
  return (await resp.json()) as T
}

/**
 * Shared response handler for `mutateJson` + `mutateForm`. Surfaces the
 * daemon's JSON error envelope on non-2xx; on 204 returns `undefined`;
 * on 2xx returns the parsed JSON body (or `undefined` for non-JSON).
 */
async function throwOnError<T>(
  resp: Response,
  method: string,
  path: string,
): Promise<T> {
  if (!resp.ok) {
    let detail = `${resp.status} ${resp.statusText}`
    try {
      const text = await resp.text()
      if (text) detail = text
    } catch { /* keep status-line fallback */ }
    throw new Error(`[ship-sdk-client] ${method} ${path} failed: ${detail}`)
  }
  if (resp.status === 204) return undefined as unknown as T
  const ct = resp.headers.get('content-type') ?? ''
  if (ct.includes('application/json')) return (await resp.json()) as T
  return undefined as unknown as T
}

/**
 * Thrown by {@link shipFetch} on non-2xx responses. Carries the HTTP status
 * and the raw response body (the daemon's JSON error envelope when present,
 * otherwise the status text) so callers can inspect the daemon's typed error.
 */
export class ShipApiError extends Error {
  readonly status: number
  readonly body: string
  constructor(status: number, body: string) {
    super(`Ship API ${status}: ${body}`)
    this.status = status
    this.body = body
  }
}

/**
 * Browser cookie-bound JSON fetch. Resolves with the parsed JSON body on
 * 2xx; throws {@link ShipApiError} on non-2xx. `credentials: 'same-origin'`
 * so the daemon auth middleware sees the browser session — same shape as
 * `mutateJson`, but caller-driven (no method/path helpers) for callers that
 * need to construct the URL themselves (e.g. legacy REST endpoints).
 */
export async function shipFetch<T>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(url, {
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json', ...(init?.headers || {}) },
    ...init,
  })
  if (!res.ok) {
    const text = await res.text().catch(() => '')
    throw new ShipApiError(res.status, text)
  }
  return res.json() as Promise<T>
}
