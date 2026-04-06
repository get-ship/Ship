import { useRouterState } from '@tanstack/react-router'
import { GitBranch, Circle } from 'lucide-react'
import { useDaemon } from './hooks/useDaemon'

/** Routes that are workspace-scoped (show the indicator). */
const WORKSPACE_ROUTES = ['/studio/session', '/studio/git', '/studio/view/']

export function WorkspaceIndicator() {
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const { workspaces, connected } = useDaemon()

  const isWorkspaceView = WORKSPACE_ROUTES.some((r) => pathname.startsWith(r))
  if (!isWorkspaceView) return null

  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')
  if (!activeWs && connected) return null

  return (
    <div className="absolute top-2 left-2 z-10 flex items-center gap-1.5 rounded-full border border-border/50 bg-card/80 backdrop-blur-sm px-2.5 py-1 shadow-sm">
      <GitBranch className="size-3 text-muted-foreground" />
      <span className="text-[11px] font-medium text-foreground/80">
        {activeWs?.branch ?? 'No workspace'}
      </span>
      {activeWs?.active_agent && (
        <>
          <Circle className="size-1 fill-border text-border" />
          <span className="text-[10px] text-muted-foreground">
            {activeWs.active_agent}
          </span>
        </>
      )}
    </div>
  )
}
