const DAEMON_PORT = (typeof import.meta !== 'undefined' && (import.meta as { env?: Record<string, string> }).env?.VITE_SHIP_DAEMON_PORT) ?? '9315'
export const DAEMON_BASE_URL = `http://localhost:${DAEMON_PORT}`

let _cachedToken: string | null = null

export function setDaemonToken(token: string | null) {
  _cachedToken = token
}

export function daemonHeaders(): Record<string, string> {
  const h: Record<string, string> = { 'Content-Type': 'application/json' }
  if (_cachedToken) h['Authorization'] = `Bearer ${_cachedToken}`
  return h
}

export function daemonFetch(path: string, init?: RequestInit): Promise<Response> {
  const headers = { ...daemonHeaders(), ...init?.headers }
  return fetch(`${DAEMON_BASE_URL}${path}`, { ...init, headers })
}
