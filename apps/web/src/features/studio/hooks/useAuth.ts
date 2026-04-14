import { useCallback, useEffect, useRef, useState } from 'react'
import type { ShipClient } from '@ship/sdk'
import { setDaemonToken } from '#/lib/daemon-config'

// ── Types ───────────────────────────────────────────────────────────────────

export type AuthState = 'anonymous' | 'authenticated'

export interface PairingState {
  code: string
  claimToken: string
  devMode: boolean
}

export interface AuthResult {
  auth: AuthState
  token: string | null
  pairing: PairingState | null
  beginPairing: () => void
  onAuthenticated: (() => void) | null
}

// ── Constants ───────────────────────────────────────────────────────────────

const SESSION_TOKEN_KEY = 'ship-studio:session-token'
const PAIR_POLL_MS = 2000

// ── Helpers ─────────────────────────────────────────────────────────────────

function extractFragmentToken(): string | null {
  const hash = window.location.hash
  if (!hash) return null
  const match = hash.match(/token=([^&]+)/)
  if (!match) return null
  window.location.hash = ''
  return match[1]
}

// ── Hook ────────────────────────────────────────────────────────────────────

export function useAuth(
  client: ShipClient,
  onConnected: () => void,
) {
  const [auth, setAuth] = useState<AuthState>('anonymous')
  const [token, setToken] = useState<string | null>(null)
  const [pairing, setPairing] = useState<PairingState | null>(null)
  const pairPollRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const restartPairingRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    let cancelled = false

    if ('setAuthFailureHandler' in client) {
      ;(client as any).setAuthFailureHandler(() => {
        if (cancelled) return
        restartPairingRef.current?.()
      })
    }

    function applyToken(sessionToken: string) {
      try { localStorage.setItem(SESSION_TOKEN_KEY, sessionToken) } catch { /* ignore */ }
      client.setToken(sessionToken)
      setDaemonToken(sessionToken)
      setToken(sessionToken)
      setAuth('authenticated')
      setPairing(null)
      onConnected()
    }

    const fragToken = extractFragmentToken()
    const cachedSession = (() => {
      try { return localStorage.getItem(SESSION_TOKEN_KEY) } catch { return null }
    })()
    const initialToken = fragToken ?? cachedSession

    if (initialToken) {
      client.setToken(initialToken)
      setDaemonToken(initialToken)
      setToken(initialToken)
      setAuth(fragToken ? 'anonymous' : 'authenticated')
      client.isConnected().then((ok) => {
        if (cancelled) return
        if (ok) {
          onConnected()
        } else {
          try { localStorage.removeItem(SESSION_TOKEN_KEY) } catch { /* ignore */ }
          client.setToken('')
          setDaemonToken(null)
          setToken(null)
          setAuth('anonymous')
          startPairing()
        }
      })
    } else {
      startAuth()
    }

    restartPairingRef.current = () => {
      try { localStorage.removeItem(SESSION_TOKEN_KEY) } catch { /* ignore */ }
      client.setToken('')
      setDaemonToken(null)
      setToken(null)
      setAuth('anonymous')
      setPairing(null)
      startAuth()
    }

    async function startAuth() {
      if (cancelled) return
      try {
        const res = await fetch('/api/auth/daemon-attest', {
          method: 'POST',
          credentials: 'include',
        })
        if (res.ok) {
          const body = await res.json() as { jwt?: string }
          if (body.jwt) {
            const sessionToken = await client.serverAttest(body.jwt)
            if (sessionToken) {
              applyToken(sessionToken)
              return
            }
          }
        }
      } catch {
        /* no getship.dev session — fall through to pairing */
      }
      startPairing()
    }

    function startPairing() {
      if (cancelled) return
      client.pairInit('studio').then((init) => {
        if (cancelled || !init) return
        setPairing({ code: init.code, claimToken: init.claim_token, devMode: init.dev_mode })
        if (pairPollRef.current) clearInterval(pairPollRef.current)
        pairPollRef.current = setInterval(async () => {
          if (cancelled) return
          const status = await client.pairStatus(init.claim_token)
          if (status.status === 'approved') {
            if (pairPollRef.current) clearInterval(pairPollRef.current)
            pairPollRef.current = null
            applyToken(status.session_token)
          } else if (status.status === 'expired' || status.status === 'unknown') {
            if (pairPollRef.current) clearInterval(pairPollRef.current)
            pairPollRef.current = null
            startPairing()
          }
        }, PAIR_POLL_MS)
      })
    }

    return () => {
      cancelled = true
      if (pairPollRef.current) clearInterval(pairPollRef.current)
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const beginPairing = useCallback(() => {
    restartPairingRef.current?.()
  }, [])

  return { auth, token, pairing, beginPairing }
}
