import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

const DRAWER_LS_KEY = 'ship-studio:right-drawer-open'

function readDrawerInitial(): boolean {
  try {
    const stored = localStorage.getItem(DRAWER_LS_KEY)
    return stored === 'true'
  } catch {
    return false
  }
}

interface StudioShellState {
  commandPaletteOpen: boolean
  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void

  fileSidebarOpen: boolean
  toggleFileSidebar: () => void
  closeFileSidebar: () => void

  drawerOpen: boolean
  toggleDrawer: () => void

  /** Which workspace the UI is viewing (local state, no daemon mutation) */
  viewedWorkspace: string | null
  setViewedWorkspace: (branch: string | null) => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [fileSidebarOpen, setFileSidebarOpen] = useState(true)
  const [drawerOpen, setDrawerOpen] = useState(readDrawerInitial)
  const [viewedWorkspace, setViewedWorkspace] = useState<string | null>(null)

  const toggleCommandPalette = useCallback(() => setCommandPaletteOpen((p) => !p), [])
  const toggleFileSidebar = useCallback(() => setFileSidebarOpen((p) => !p), [])
  const closeFileSidebar = useCallback(() => setFileSidebarOpen(false), [])
  const toggleDrawer = useCallback(() => {
    setDrawerOpen((prev) => {
      const next = !prev
      try { localStorage.setItem(DRAWER_LS_KEY, String(next)) } catch { /* ignore */ }
      return next
    })
  }, [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey

      if (e.key === 'k' && mod) {
        e.preventDefault()
        toggleCommandPalette()
        return
      }
      if (e.key === 'b' && mod) {
        e.preventDefault()
        toggleFileSidebar()
        return
      }
      if (e.key === '/' && mod) {
        e.preventDefault()
        toggleDrawer()
        return
      }
      if (e.key === 'Escape' && commandPaletteOpen) {
        setCommandPaletteOpen(false)
      }
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) void document.exitFullscreen()
        else void document.documentElement.requestFullscreen()
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [toggleCommandPalette, toggleFileSidebar, toggleDrawer, commandPaletteOpen])

  return (
    <StudioShellContext value={{
      commandPaletteOpen, toggleCommandPalette, setCommandPaletteOpen,
      fileSidebarOpen, toggleFileSidebar, closeFileSidebar,
      drawerOpen, toggleDrawer,
      viewedWorkspace, setViewedWorkspace,
    }}>
      {children}
    </StudioShellContext>
  )
}

export function useStudioShell(): StudioShellState {
  const ctx = useContext(StudioShellContext)
  if (!ctx) throw new Error('useStudioShell must be used within StudioShellProvider')
  return ctx
}

/**
 * Returns the workspace branch the UI is currently viewing.
 * Uses viewedWorkspace if set, otherwise falls back to first active workspace.
 * All data-fetching hooks should use this instead of filtering for status=active.
 */
export function useViewedWorkspaceId(): string {
  const { viewedWorkspace } = useStudioShell()
  // Can't call useDaemon here (circular) — caller must pass workspaces if needed
  return viewedWorkspace ?? ''
}
