import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

/** Sidebar panel IDs — each corresponds to a different left panel surface */
export type PanelId = 'workspaces' | 'artifacts' | 'search' | 'jobs' | 'settings'

interface StudioShellState {
  /** Which sidebar panel is open, or null if collapsed */
  activePanel: PanelId | null
  setActivePanel: (panel: PanelId | null) => void

  /** Legacy compat — maps to activePanel === 'artifacts' */
  sidebarOpen: boolean
  toggleSidebar: () => void
  closeSidebar: () => void

  /** Floating panels */
  eventsOpen: boolean
  toggleEvents: () => void
  chatOpen: boolean
  toggleChat: () => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const [activePanel, setActivePanelRaw] = useState<PanelId | null>(null)
  const [eventsOpen, setEventsOpen] = useState(false)
  const [chatOpen, setChatOpen] = useState(false)

  const setActivePanel = useCallback((panel: PanelId | null) => {
    setActivePanelRaw(panel)
  }, [])

  // Legacy compat for session sidebar
  const sidebarOpen = activePanel === 'artifacts'
  const toggleSidebar = useCallback(() => {
    setActivePanelRaw((p) => p === 'artifacts' ? null : 'artifacts')
  }, [])
  const closeSidebar = useCallback(() => {
    setActivePanelRaw((p) => p === 'artifacts' ? null : p)
  }, [])

  const toggleEvents = useCallback(() => setEventsOpen((p) => !p), [])
  const toggleChat = useCallback(() => setChatOpen((p) => !p), [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'b' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        toggleSidebar()
      }
      if (e.key === 'Escape' && activePanel) {
        setActivePanelRaw(null)
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [toggleSidebar, activePanel])

  return (
    <StudioShellContext value={{
      activePanel, setActivePanel,
      sidebarOpen, toggleSidebar, closeSidebar,
      eventsOpen, toggleEvents,
      chatOpen, toggleChat,
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
