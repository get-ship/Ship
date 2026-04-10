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
  branch: string
}

interface ConnectionMeta {
  reconnects: number
}

export interface PlatformState {
  daemon: DaemonState
  auth: AuthState
  token: string | null
  workspace: Workspace | null
  setWorkspace: (branch: string | null) => void
  commandPaletteOpen: boolean
  toggleCommandPalette: () => void
  sidebarOpen: boolean
  toggleSidebar: () => void
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
const MAX_RETRIES = 3
const BASE_BACKOFF_MS = 1000

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
  const branch = localStorage.getItem(WORKSPACE_KEY)
  return branch ? { branch } : null
}

// ── Provider ────────────────────────────────────────────────────────────────

export function PlatformProvider({ children }: { children: ReactNode }) {
  const clientRef = useRef<ShipClient>(
    new ShipClient({ baseUrl: DAEMON_BASE_URL }),
  )
  const retryCountRef = useRef(0)
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const [daemon, setDaemon] = useState<DaemonState>('connecting')
  const [auth, setAuth] = useState<AuthState>('anonymous')
  const [token, setToken] = useState<string | null>(null)
  const [workspace, setWorkspaceState] = useState<Workspace | null>(
    loadSavedWorkspace,
  )
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [reconnects, setReconnects] = useState(0)

  const client = clientRef.current

  // ── Workspace management ──────────────────────────────────────────────

  const setWorkspace = useCallback(
    (branch: string | null) => {
      if (branch) {
        setWorkspaceState({ branch })
        localStorage.setItem(WORKSPACE_KEY, branch)
        client.invalidatePrefix('/runtime/sessions')
      } else {
        setWorkspaceState(null)
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

    function onConnected() {
      setDaemon('connected')
      client.connectEvents()
      if ('onSSEError' in client) {
        ;(client as any).onSSEError(() => scheduleReconnect())
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

    const fragToken = extractFragmentToken()
    if (fragToken) {
      client.setToken(fragToken)
      setDaemonToken(fragToken)
      setToken(fragToken)
      setAuth('anonymous')
      checkDaemon()
    } else {
      client.authenticate().then((authOk) => {
        if (cancelled) return
        setAuth(authOk ? 'authenticated' : 'anonymous')
        checkDaemon()
      })
    }

    return () => {
      cancelled = true
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current)
      client.destroy()
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ── Context value ─────────────────────────────────────────────────────

  const value: PlatformState = {
    daemon,
    auth,
    token,
    workspace,
    setWorkspace,
    commandPaletteOpen,
    toggleCommandPalette,
    sidebarOpen,
    toggleSidebar,
    client,
    connection: { reconnects },
  }

  return (
    <PlatformContext.Provider value={value}>{children}</PlatformContext.Provider>
  )
}
