import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react'

interface StudioShellState {
  /** Command palette */
  commandPaletteOpen: boolean
  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void

  /** Session file sidebar (overlay within session page) */
  fileSidebarOpen: boolean
  toggleFileSidebar: () => void
  closeFileSidebar: () => void

  /** Agent chat/events panel (global floating) */
  agentPanelOpen: boolean
  toggleAgentPanel: () => void
}

const StudioShellContext = createContext<StudioShellState | null>(null)

export function StudioShellProvider({ children }: { children: ReactNode }) {
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false)
  const [fileSidebarOpen, setFileSidebarOpen] = useState(true)
  const [agentPanelOpen, setAgentPanelOpen] = useState(false)

  const toggleCommandPalette = useCallback(() => setCommandPaletteOpen((p) => !p), [])
  const toggleFileSidebar = useCallback(() => setFileSidebarOpen((p) => !p), [])
  const closeFileSidebar = useCallback(() => setFileSidebarOpen(false), [])
  const toggleAgentPanel = useCallback(() => setAgentPanelOpen((p) => !p), [])

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
      // Cmd+/ = agent panel
      if (e.key === '/' && mod) {
        e.preventDefault()
        toggleAgentPanel()
        return
      }
      if (e.key === 'Escape') {
        if (commandPaletteOpen) { setCommandPaletteOpen(false); return }
        if (fileSidebarOpen) { setFileSidebarOpen(false); return }
        if (agentPanelOpen) { setAgentPanelOpen(false); return }
      }
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) void document.exitFullscreen()
        else void document.documentElement.requestFullscreen()
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [toggleCommandPalette, toggleFileSidebar, toggleAgentPanel, commandPaletteOpen, fileSidebarOpen, agentPanelOpen])

  return (
    <StudioShellContext value={{
      commandPaletteOpen, toggleCommandPalette, setCommandPaletteOpen,
      fileSidebarOpen, toggleFileSidebar, closeFileSidebar,
      agentPanelOpen, toggleAgentPanel,
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
