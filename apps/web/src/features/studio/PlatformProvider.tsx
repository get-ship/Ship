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
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useAuth, type AuthState, type PairingState } from './hooks/useAuth'

// ── Types ───────────────────────────────────────────────────────────────────

type DaemonState = 'connected' | 'disconnected' | 'connecting'

interface Workspace {
  name: string
}

interface ConnectionMeta {
  reconnects: number
}

export type { PairingState }

export interface PlatformState {
  daemon: DaemonState
  auth: AuthState
  token: string | null
  pairing: PairingState | null
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
const MAX_RETRIES = 3
const BASE_BACKOFF_MS = 1000

// ── Helpers ─────────────────────────────────────────────────────────────────

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

  // ── Connection handler ────────────────────────────────────────────────

  const handleConnected = useCallback(() => {
    setDaemon('connected')
    client.connectEvents()
    if ('onSSEError' in client) {
      ;(client as any).onSSEError(() => scheduleReconnect())
    }
    if (!workspaceRef.current) {
      client.get<{ workspaces: Array<{ name: string; status: string; last_activated_at?: string }> }>('/runtime/workspaces').then((data) => {
        const sorted = data.workspaces
          ?.slice()
          .sort((a: any, b: any) => (b.last_activated_at ?? '').localeCompare(a.last_activated_at ?? ''))
        const best = sorted?.find((w: any) => String(w.status).toLowerCase() === 'active') ?? sorted?.[0]
        if (best) setWorkspace(best.name)
      }).catch(() => {})
    }
  }, [client, scheduleReconnect, setWorkspace])

  // ── Auth lifecycle (extracted) ────────────────────────────────────────

  const { auth, token, pairing, beginPairing } = useAuth(client, handleConnected)

  // Set daemon to disconnected when auth falls through to pairing
  useEffect(() => {
    if (pairing) setDaemon('disconnected')
  }, [pairing])

  // ── Health polling ────────────────────────────────────────────────────

  useEffect(() => {
    let cancelled = false
    let healthStarted = false
    const healthInterval = setInterval(() => {
      if (cancelled || !healthStarted) return
      client.isConnected().then((ok) => {
        if (cancelled) return
        if (ok) handleConnected()
      })
    }, 10_000)
    setTimeout(() => { healthStarted = true }, 5_000)
    return () => {
      cancelled = true
      clearInterval(healthInterval)
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current)
      client.destroy()
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

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
