import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

/** Rail panel IDs — each opens an overlay panel from the left */
export type PanelId = 'files' | 'git' | 'views' | 'jobs'

interface StudioShellState {
  /** Which overlay panel is open, or null */
  activePanel: PanelId | null
  setActivePanel: (panel: PanelId | null) => void
  togglePanel: (panel: PanelId) => void

  /** Floating overlays */
  eventsOpen: boolean
  toggleEvents: () => void
  chatOpen: boolean
  toggleChat: () => void

  /** Legacy compat for session sidebar */
  sidebarOpen: boolean
  toggleSidebar: () => void
  closeSidebar: () => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const [activePanel, setActivePanelRaw] = useState<PanelId | null>(null)
  const [eventsOpen, setEventsOpen] = useState(false)
  const [chatOpen, setChatOpen] = useState(false)

  const setActivePanel = useCallback((panel: PanelId | null) => setActivePanelRaw(panel), [])
  const togglePanel = useCallback((panel: PanelId) => {
    setActivePanelRaw((p) => p === panel ? null : panel)
  }, [])

  const toggleEvents = useCallback(() => setEventsOpen((p) => !p), [])
  const toggleChat = useCallback(() => setChatOpen((p) => !p), [])

  // Legacy: files panel = sidebar
  const sidebarOpen = activePanel === 'files'
  const toggleSidebar = useCallback(() => togglePanel('files'), [togglePanel])
  const closeSidebar = useCallback(() => {
    setActivePanelRaw((p) => p === 'files' ? null : p)
  }, [])

  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'b' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        togglePanel('files')
      }
      if (e.key === 'j' && (e.metaKey || e.ctrlKey) && e.shiftKey) {
        e.preventDefault()
        togglePanel('jobs')
      }
      if (e.key === 'Escape' && activePanel) {
        setActivePanelRaw(null)
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [togglePanel, activePanel])

  return (
    <StudioShellContext value={{
      activePanel, setActivePanel, togglePanel,
      eventsOpen, toggleEvents,
      chatOpen, toggleChat,
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
