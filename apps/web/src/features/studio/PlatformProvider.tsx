import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  useCallback,
  type ReactNode,
} from 'react'
import { ShipClient } from '@ship/sdk'
import { DAEMON_BASE_URL, setDaemonToken } from '#/lib/daemon-config'

// ── Types ───────────────────────────────────────────────────────────────────

type DaemonState = 'connected' | 'disconnected' | 'connecting'
type AuthState = 'anonymous' | 'authenticated'

interface Workspace {
  name: string
}

interface ConnectionMeta {
  reconnects: number
}

export interface PairingState {
  code: string
  claimToken: string
  devMode: boolean
}

export interface PlatformState {
  daemon: DaemonState
  auth: AuthState
  token: string | null
  /** Active pairing request (when auth is pending). Consumers can render the code. */
  pairing: PairingState | null
  /** Start a new pairing flow (cancels any in-progress one). */
  beginPairing: () => void
  workspace: Workspace | null
  setWorkspace: (name: string | null) => void
  commandPaletteOpen: boolean
  toggleCommandPalette: () => void
  sidebarOpen: boolean
  toggleSidebar: () => void
  drawerOpen: boolean
  toggleDrawer: () => void
  client: ShipClient
  connection: ConnectionMeta
}

// ── Context ─────────────────────────────────────────────────────────────────

const PlatformContext = createContext<PlatformState | null>(null)

export function usePlatform(): PlatformState {
  const ctx = useContext(PlatformContext)
  if (!ctx) {
    throw new Error('usePlatform must be used within PlatformProvider')
  }
  return ctx
}

// ── Constants ───────────────────────────────────────────────────────────────

const WORKSPACE_KEY = 'ship-studio:viewed-workspace'
const DRAWER_LS_KEY = 'ship-studio:right-drawer-open'
const SESSION_TOKEN_KEY = 'ship-studio:session-token'
const MAX_RETRIES = 3
const BASE_BACKOFF_MS = 1000
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

function loadSavedWorkspace(): Workspace | null {
  const name = localStorage.getItem(WORKSPACE_KEY)
  return name ? { name } : null
}

function readDrawerInitial(): boolean {
  try {
    const stored = localStorage.getItem(DRAWER_LS_KEY)
    return stored === 'true'
  } catch {
    return false
  }
}

// ── Provider ────────────────────────────────────────────────────────────────

export function PlatformProvider({ children }: { children: ReactNode }) {
  const clientRef = useRef<ShipClient>(
    new ShipClient({ baseUrl: DAEMON_BASE_URL }),
  )
  const retryCountRef = useRef(0)
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const workspaceRef = useRef<Workspace | null>(loadSavedWorkspace())

  const [daemon, setDaemon] = useState<DaemonState>('connecting')
  const [auth, setAuth] = useState<AuthState>('anonymous')
  const [token, setToken] = useState<string | null>(null)
  const [pairing, setPairing] = useState<PairingState | null>(null)
  const pairPollRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const restartPairingRef = useRef<(() => void) | null>(null)
  const [workspace, setWorkspaceState] = useState<Workspace | null>(
    loadSavedWorkspace,
  )
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [drawerOpen, setDrawerOpen] = useState(readDrawerInitial)
  const [reconnects, setReconnects] = useState(0)

  const client = clientRef.current

  // ── Workspace management ──────────────────────────────────────────────

  const setWorkspace = useCallback(
    (name: string | null) => {
      if (name) {
        const ws = { name }
        setWorkspaceState(ws)
        workspaceRef.current = ws
        localStorage.setItem(WORKSPACE_KEY, name)
        client.invalidatePrefix('/runtime/sessions')
      } else {
        setWorkspaceState(null)
        workspaceRef.current = null
        localStorage.removeItem(WORKSPACE_KEY)
      }
    },
    [client],
  )

  // ── Shell UI toggles ─────────────────────────────────────────────────

  const toggleCommandPalette = useCallback(
    () => setCommandPaletteOpen((v) => !v),
    [],
  )
  const toggleSidebar = useCallback(() => setSidebarOpen((v) => !v), [])
  const toggleDrawer = useCallback(() => {
    setDrawerOpen((prev) => {
      const next = !prev
      try { localStorage.setItem(DRAWER_LS_KEY, String(next)) } catch { /* ignore */ }
      return next
    })
  }, [])

  // ── Global keyboard shortcuts ────────────────────────────────────────

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey
      if (e.key === 'k' && mod) { e.preventDefault(); toggleCommandPalette(); return }
      if (e.key === 'b' && mod) { e.preventDefault(); toggleSidebar(); return }
      if (e.key === '/' && mod) { e.preventDefault(); toggleDrawer(); return }
      if (e.key === 'Escape' && commandPaletteOpen) {
        toggleCommandPalette()
      }
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) void document.exitFullscreen()
        else void document.documentElement.requestFullscreen()
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [toggleCommandPalette, toggleSidebar, toggleDrawer, commandPaletteOpen])

  // ── SSE error handler (for reconnect) ─────────────────────────────────

  const scheduleReconnect = useCallback(() => {
    setDaemon('connecting')
    const attempt = retryCountRef.current
    if (attempt >= MAX_RETRIES) {
      setDaemon('disconnected')
      return
    }
    const delay = BASE_BACKOFF_MS * Math.pow(2, attempt)
    retryTimerRef.current = setTimeout(async () => {
      const ok = await client.isConnected()
      if (ok) {
        retryCountRef.current = 0
        setDaemon('connected')
        setReconnects((r) => r + 1)
        client.connectEvents()
      } else {
        retryCountRef.current += 1
        scheduleReconnect()
      }
    }, delay)
  }, [client])

  // ── Init lifecycle ────────────────────────────────────────────────────

  useEffect(() => {
    let cancelled = false

    // If the daemon starts returning 401/403, the session was revoked or
    // the daemon regenerated state. Drop the cached token and re-pair.
    if ('setAuthFailureHandler' in client) {
      ;(client as any).setAuthFailureHandler(() => {
        if (cancelled) return
        restartPairingRef.current?.()
      })
    }

    function onConnected() {
      setDaemon('connected')
      client.connectEvents()
      if ('onSSEError' in client) {
        ;(client as any).onSSEError(() => scheduleReconnect())
      }
      // Auto-select workspace if none set
      if (!workspaceRef.current) {
        client.get<{ workspaces: Array<{ name: string; status: string; last_activated_at?: string }> }>('/runtime/workspaces').then((data) => {
          if (cancelled) return
          const sorted = data.workspaces
            ?.slice()
            .sort((a: any, b: any) => (b.last_activated_at ?? '').localeCompare(a.last_activated_at ?? ''))
          const best = sorted?.find((w: any) => String(w.status).toLowerCase() === 'active') ?? sorted?.[0]
          if (best) {
            setWorkspace(best.name)
          }
        }).catch(() => {})
      }
    }

    function checkDaemon() {
      client.isConnected().then((ok) => {
        if (cancelled) return
        if (ok) {
          onConnected()
        } else {
          setDaemon('disconnected')
        }
      })
    }

    // Try tokens in priority order:
    //   1. URL fragment (`#token=...` from `ship studio` command)
    //   2. Cached session token from a previous pairing
    //   3. Start a fresh pairing flow
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
      // Verify the token actually works; on failure, fall through to pairing.
      client.isConnected().then((ok) => {
        if (cancelled) return
        if (ok) {
          onConnected()
        } else {
          // Cached token is stale — drop it and pair again.
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

    // Expose startAuth to beginPairing/401-handlers via a ref.
    restartPairingRef.current = () => {
      try { localStorage.removeItem(SESSION_TOKEN_KEY) } catch { /* ignore */ }
      client.setToken('')
      setDaemonToken(null)
      setToken(null)
      setAuth('anonymous')
      setPairing(null)
      startAuth()
    }

    /** Try zero-click server attestation first; fall back to pair flow. */
    async function startAuth() {
      if (cancelled) return
      setDaemon('disconnected')

      // Attempt server attestation: fetch a JWT from getship.dev (requires
      // Better Auth session cookie), then exchange with the daemon.
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
              try { localStorage.setItem(SESSION_TOKEN_KEY, sessionToken) } catch { /* ignore */ }
              client.setToken(sessionToken)
              setDaemonToken(sessionToken)
              setToken(sessionToken)
              setAuth('authenticated')
              setPairing(null)
              onConnected()
              return
            }
          }
        }
      } catch {
        /* no getship.dev session, or offline — fall through to pairing */
      }

      startPairing()
    }

    function startPairing() {
      if (cancelled) return
      setDaemon('disconnected')
      client.pairInit('studio').then((init) => {
        if (cancelled || !init) return
        setPairing({ code: init.code, claimToken: init.claim_token, devMode: init.dev_mode })
        // Poll for approval.
        if (pairPollRef.current) clearInterval(pairPollRef.current)
        pairPollRef.current = setInterval(async () => {
          if (cancelled) return
          const status = await client.pairStatus(init.claim_token)
          if (status.status === 'approved') {
            if (pairPollRef.current) clearInterval(pairPollRef.current)
            pairPollRef.current = null
            try { localStorage.setItem(SESSION_TOKEN_KEY, status.session_token) } catch { /* ignore */ }
            client.setToken(status.session_token)
            setDaemonToken(status.session_token)
            setToken(status.session_token)
            setAuth('authenticated')
            setPairing(null)
            onConnected()
          } else if (status.status === 'expired' || status.status === 'unknown') {
            if (pairPollRef.current) clearInterval(pairPollRef.current)
            pairPollRef.current = null
            startPairing()  // restart with a fresh code
          }
        }, PAIR_POLL_MS)
      })
    }

    // Poll for daemon recovery when disconnected (every 10s, starts after init)
    let initDone = false
    const healthInterval = setInterval(() => {
      if (cancelled || !initDone) return
      // Only poll when not already connected
      if (client.getToken()) return
      client.authenticate().then((ok) => {
        if (cancelled) return
        if (ok) {
          setToken(client.getToken())
          if (client.getToken()) setDaemonToken(client.getToken()!)
          onConnected()
        }
      })
    }, 10_000)
    // Delay health polling until init completes
    setTimeout(() => { initDone = true }, 3_000)

    return () => {
      cancelled = true
      clearInterval(healthInterval)
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current)
      if (pairPollRef.current) clearInterval(pairPollRef.current)
      client.destroy()
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ── Manual pairing restart ────────────────────────────────────────────

  const beginPairing = useCallback(() => {
    // Restart the pair flow in-place (no page reload).
    restartPairingRef.current?.()
  }, [])

  // ── Context value ─────────────────────────────────────────────────────

  const value: PlatformState = {
    daemon,
    auth,
    token,
    pairing,
    beginPairing,
    workspace,
    setWorkspace,
    commandPaletteOpen,
    toggleCommandPalette,
    sidebarOpen,
    toggleSidebar,
    drawerOpen,
    toggleDrawer,
    client,
    connection: { reconnects },
  }

  return (
    <PlatformContext.Provider value={value}>{children}</PlatformContext.Provider>
  )
}
