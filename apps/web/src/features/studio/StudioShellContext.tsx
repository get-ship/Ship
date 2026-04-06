import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

/** Rail panel IDs -- each opens an overlay panel from the left */
export type PanelId = 'files' | 'git' | 'views' | 'jobs'

interface StudioShellState {
  /** Which overlay panel is open, or null */
  activePanel: PanelId | null
  setActivePanel: (panel: PanelId | null) => void
  togglePanel: (panel: PanelId) => void

  /** Right drawer open state */
  rightDrawerOpen: boolean
  toggleRightDrawer: () => void

  /** Command palette */
  commandPaletteOpen: boolean
  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void

  /** Legacy compat for session sidebar */
  sidebarOpen: boolean
  toggleSidebar: () => void
  closeSidebar: () => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const [activePanel, setActivePanelRaw] = useState<PanelId | null>(null)
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [rightDrawerOpen, setRightDrawerOpen] = useState(false)

  const setActivePanel = useCallback((panel: PanelId | null) => setActivePanelRaw(panel), [])
  const togglePanel = useCallback((panel: PanelId) => {
    setActivePanelRaw((p) => p === panel ? null : panel)
  }, [])

  const toggleCommandPalette = useCallback(() => setCommandPaletteOpen((p) => !p), [])
  const toggleRightDrawer = useCallback(() => setRightDrawerOpen((p) => !p), [])

  // Legacy: files panel = sidebar
  const sidebarOpen = activePanel === 'files'
  const toggleSidebar = useCallback(() => togglePanel('files'), [togglePanel])
  const closeSidebar = useCallback(() => {
    setActivePanelRaw((p) => p === 'files' ? null : p)
  }, [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey

      // Cmd+B = toggle files
      if (e.key === 'b' && mod) {
        e.preventDefault()
        togglePanel('files')
        return
      }

      // Cmd+K = command palette
      if (e.key === 'k' && mod) {
        e.preventDefault()
        toggleCommandPalette()
        return
      }

      // Cmd+/ = toggle right drawer
      if (e.key === '/' && mod) {
        e.preventDefault()
        toggleRightDrawer()
        return
      }

      // Escape = close panel or palette
      if (e.key === 'Escape') {
        if (commandPaletteOpen) {
          setCommandPaletteOpen(false)
          return
        }
        if (activePanel) {
          setActivePanelRaw(null)
          return
        }
      }

      // F11 = fullscreen
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) {
          void document.exitFullscreen()
        } else {
          void document.documentElement.requestFullscreen()
        }
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [togglePanel, toggleCommandPalette, toggleRightDrawer, activePanel, commandPaletteOpen])

  return (
    <StudioShellContext value={{
      activePanel, setActivePanel, togglePanel,
      rightDrawerOpen, toggleRightDrawer,
      commandPaletteOpen, toggleCommandPalette, setCommandPaletteOpen,
      sidebarOpen, toggleSidebar, closeSidebar,
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
