import { useState, useEffect, useCallback } from 'react'
import { useRouterState } from '@tanstack/react-router'
import { GitBranch, Circle, Maximize, Minimize, PanelLeft, ChevronDown } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@ship/primitives'
import { Separator } from '@ship/primitives'
import { useDaemon } from './hooks/useDaemonData'
import { usePlatform } from './PlatformProvider'
import { resolveWorkspaceSelection } from './workspace-selection'

const WORKSPACE_ROUTES = ['/studio/session', '/studio/git', '/studio/view/']

export function WorkspaceIndicator() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const { workspaces, connected } = useDaemon()
  const platform = usePlatform()
  const fileSidebarOpen = platform.sidebarOpen
  const toggleFileSidebar = platform.toggleSidebar
  const viewedWorkspace = platform.workspace?.branch ?? null
  const setViewedWorkspace = platform.setWorkspace
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

  // Use viewedWorkspace if set, otherwise default to first active or first workspace
  const currentWs = viewedWorkspace
    ? workspaces.find((w) => w.branch === viewedWorkspace)
    : workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0]

  // Hydrate viewedWorkspace on first load or clear stale selection.
  useEffect(() => {
    if (!isWorkspaceView) return
    const next = resolveWorkspaceSelection(viewedWorkspace, workspaces)
    if (next !== undefined) setViewedWorkspace(next)
  }, [isWorkspaceView, viewedWorkspace, workspaces, setViewedWorkspace])

  if (!isWorkspaceView) return null
  if (!currentWs && connected) {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 border-b border-border/40 shrink-0 bg-card/30">
        <GitBranch className="size-3 text-muted-foreground/60" />
        <span className="text-[10px] text-muted-foreground/60">No workspaces</span>
        <div className="flex-1" />
      </div>
    )
  }

  const otherWorkspaces = workspaces.filter((w) => w.branch !== currentWs?.branch)
  const isArtifacts = pathname.startsWith('/studio/session')

  return (
    <div className="flex items-center gap-1.5 px-2 py-1 border-b border-border/40 shrink-0 bg-card/30">
      {isArtifacts && (
        <button
          onClick={toggleFileSidebar}
          className={`size-6 flex items-center justify-center rounded transition-colors ${
            fileSidebarOpen ? 'text-foreground/80' : 'text-muted-foreground/50 hover:text-foreground/80'
          }`}
          title="Toggle sidebar (Cmd+B)"
        >
          <PanelLeft className="size-3.5" />
        </button>
      )}

      {/* Workspace picker — view-only, no daemon mutation */}
      <Popover>
        <PopoverTrigger
          render={
            <button className="flex items-center gap-1 rounded px-1 py-0.5 hover:bg-muted/40 transition-colors">
              <GitBranch className="size-3 text-muted-foreground/60" />
              <span className="text-[10px] font-medium text-muted-foreground">
                {currentWs?.branch ?? 'No workspace'}
              </span>
              {currentWs?.active_agent && (
                <>
                  <Circle className="size-1 fill-border text-border" />
                  <span className="text-[10px] text-muted-foreground/60">
                    {currentWs.active_agent}
                  </span>
                </>
              )}
              <ChevronDown className="size-2.5 text-muted-foreground/60" />
            </button>
          }
        />
        <PopoverContent align="start" className="w-64 p-0">
          {currentWs && (
            <div className="px-3 py-2">
              <div className="text-[10px] text-muted-foreground/50 uppercase tracking-wider mb-1">Viewing</div>
              <div className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-emerald-500 shrink-0" />
                <span className="text-xs font-medium truncate">{currentWs.branch}</span>
              </div>
              {currentWs.active_agent && (
                <div className="text-[10px] text-muted-foreground/60 mt-0.5 pl-3.5">{currentWs.active_agent}</div>
              )}
            </div>
          )}

          {otherWorkspaces.length > 0 && (
            <>
              <Separator />
              <div className="py-1">
                <div className="px-3 py-1 text-[10px] text-muted-foreground/50 uppercase tracking-wider">Switch view to</div>
                {otherWorkspaces.map((ws) => (
                  <button
                    key={ws.branch}
                    onClick={() => setViewedWorkspace(ws.branch)}
                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
                  >
                    <span className={`size-1.5 rounded-full shrink-0 ${String(ws.status).toLowerCase() === 'active' ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
                    <span className="truncate">{ws.branch}</span>
                  </button>
                ))}
              </div>
            </>
          )}
        </PopoverContent>
      </Popover>

      <div className="flex-1" />

      <button
        onClick={handleFullscreen}
        className="size-6 flex items-center justify-center rounded text-muted-foreground/50 hover:text-foreground/80 transition-colors"
        title={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
      >
        {isFullscreen ? <Minimize className="size-3.5" /> : <Maximize className="size-3.5" />}
      </button>
    </div>
  )
}
