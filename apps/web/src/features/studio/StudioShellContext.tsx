// Compatibility layer — delegates to PlatformProvider.
// TODO: migrate all consumers to usePlatform() directly, then delete this file.

import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'
import { usePlatform } from './PlatformProvider'

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

  viewedWorkspace: string | null
  setViewedWorkspace: (branch: string | null) => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const platform = usePlatform()
  const [drawerOpen, setDrawerOpen] = useState(readDrawerInitial)

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
      if (e.key === 'k' && mod) { e.preventDefault(); platform.toggleCommandPalette(); return }
      if (e.key === 'b' && mod) { e.preventDefault(); platform.toggleSidebar(); return }
      if (e.key === '/' && mod) { e.preventDefault(); toggleDrawer(); return }
      if (e.key === 'Escape' && platform.commandPaletteOpen) {
        platform.toggleCommandPalette()
      }
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) void document.exitFullscreen()
        else void document.documentElement.requestFullscreen()
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [platform, toggleDrawer])

  return (
    <StudioShellContext value={{
      commandPaletteOpen: platform.commandPaletteOpen,
      toggleCommandPalette: platform.toggleCommandPalette,
      setCommandPaletteOpen: () => platform.toggleCommandPalette(),
      fileSidebarOpen: platform.sidebarOpen,
      toggleFileSidebar: platform.toggleSidebar,
      closeFileSidebar: () => {}, // TODO: add to platform if needed
      drawerOpen,
      toggleDrawer,
      viewedWorkspace: platform.workspace?.branch ?? null,
      setViewedWorkspace: (branch) => platform.setWorkspace(branch),
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

export function useViewedWorkspaceId(): string {
  const { viewedWorkspace } = useStudioShell()
  return viewedWorkspace ?? ''
}
