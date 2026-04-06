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
    <div className="flex items-center gap-1.5 px-3 py-1 border-b border-border/40 shrink-0 bg-card/30">
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
    </div>
  )
}
