const DAEMON_PORT = import.meta.env.VITE_SHIP_DAEMON_PORT ?? '9315'
export const DAEMON_BASE_URL = `http://localhost:${DAEMON_PORT}`

let _cachedToken: string | null = null

/** Set the daemon auth token (called once by ShipClientProvider after reading from server). */
export function setDaemonToken(token: string | null) {
  _cachedToken = token
}

/** Get headers for direct daemon fetch calls. Prefer ShipClient over raw fetch. */
export function daemonHeaders(): Record<string, string> {
  const h: Record<string, string> = { 'Content-Type': 'application/json' }
  if (_cachedToken) h['Authorization'] = `Bearer ${_cachedToken}`
  return h
}

/** Authenticated fetch to the daemon. Drop-in replacement for fetch(DAEMON_BASE_URL + path). */
export function daemonFetch(path: string, init?: RequestInit): Promise<Response> {
  const headers = { ...daemonHeaders(), ...init?.headers }
  return fetch(`${DAEMON_BASE_URL}${path}`, { ...init, headers })
}
