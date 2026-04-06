import { useState, useEffect, useCallback } from 'react'
import { useRouterState } from '@tanstack/react-router'
import { GitBranch, Circle, Maximize, Minimize, PanelLeft } from 'lucide-react'
import { useDaemon } from './hooks/useDaemon'
import { useStudioShell } from './StudioShellContext'

/** Routes that are workspace-scoped (show the indicator). */
const WORKSPACE_ROUTES = ['/studio/session', '/studio/git', '/studio/view/']

export function WorkspaceIndicator() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const { workspaces, connected } = useDaemon()
  const { fileSidebarOpen, toggleFileSidebar } = useStudioShell()
  const [isFullscreen, setIsFullscreen] = useState(false)

  useEffect(() => {
    const onFs = () => setIsFullscreen(!!document.fullscreenElement)
    document.addEventListener('fullscreenchange', onFs)
    return () => document.removeEventListener('fullscreenchange', onFs)
  }, [])

  const handleFullscreen = useCallback(() => {
    if (document.fullscreenElement) void document.exitFullscreen()
    else void document.documentElement.requestFullscreen()
  }, [])

  const isWorkspaceView = WORKSPACE_ROUTES.some((r) => pathname.startsWith(r))
  if (!isWorkspaceView) return null

  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')
  if (!activeWs && connected) return null

  const isArtifacts = pathname.startsWith('/studio/session')

  return (
    <div className="flex items-center gap-1.5 px-2 py-1 border-b border-border/40 shrink-0 bg-card/30">
      {/* Sidebar toggle — only on artifacts view */}
      {isArtifacts && (
        <button
          onClick={toggleFileSidebar}
          className={`size-6 flex items-center justify-center rounded transition-colors ${
            fileSidebarOpen ? 'text-foreground/70' : 'text-muted-foreground/50 hover:text-foreground/70'
          }`}
          title="Toggle sidebar (Cmd+B)"
        >
          <PanelLeft className="size-3.5" />
        </button>
      )}

      {/* Workspace info */}
      <GitBranch className="size-3 text-muted-foreground/60" />
      <span className="text-[10px] font-medium text-muted-foreground">
        {activeWs?.branch ?? 'No workspace'}
      </span>
      {activeWs?.active_agent && (
        <>
          <Circle className="size-1 fill-border text-border" />
          <span className="text-[10px] text-muted-foreground/60">
            {activeWs.active_agent}
          </span>
        </>
      )}

      <div className="flex-1" />

      {/* Fullscreen */}
      <button
        onClick={handleFullscreen}
        className="size-6 flex items-center justify-center rounded text-muted-foreground/50 hover:text-foreground/70 transition-colors"
        title={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
      >
        {isFullscreen
          ? <Minimize className="size-3.5" />
          : <Maximize className="size-3.5" />}
      </button>
    </div>
  )
}
