import { X } from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import type { ViewId } from './Rail'

interface LeftPanelProps {
  open: boolean
  onClose: () => void
  activeView: ViewId
  onViewChange: (view: ViewId) => void
  activeWorkspace: string | null
  onWorkspaceChange: (branch: string) => void
}

export function LeftPanel({ open, onClose, activeWorkspace, onWorkspaceChange }: LeftPanelProps) {
  const { workspaces } = useDaemon()

  if (!open) return null

  return (
    <>
      {/* Backdrop */}
      <div className="absolute inset-0 z-30" onClick={onClose} />

      {/* Panel */}
      <div className="absolute left-12 top-0 bottom-0 w-64 z-40 bg-card border-r border-border/60 shadow-lg flex flex-col animate-in slide-in-from-left-2 duration-150">
        <div className="flex items-center justify-between px-3 py-2 border-b border-border/40">
          <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Workspaces</span>
          <button onClick={onClose} className="text-muted-foreground hover:text-foreground">
            <X className="size-3.5" />
          </button>
        </div>

        <div className="flex-1 overflow-auto py-1">
          {workspaces.length === 0 && (
            <div className="px-3 py-6 text-xs text-muted-foreground text-center">
              No workspaces
            </div>
          )}
          {workspaces.map((ws) => (
            <button
              key={ws.branch}
              onClick={() => {
                onWorkspaceChange(ws.branch)
                onClose()
              }}
              className={`w-full text-left px-3 py-2 text-xs transition-colors ${
                activeWorkspace === ws.branch
                  ? 'bg-primary/10 text-foreground'
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground'
              }`}
            >
              <div className="font-medium truncate">{ws.branch}</div>
              <div className="flex items-center gap-2 mt-0.5">
                <StatusDot status={ws.status} />
                <span className="text-[10px] text-muted-foreground/60">
                  {ws.active_agent ?? 'no agent'}
                </span>
              </div>
            </button>
          ))}
        </div>
      </div>
    </>
  )
}

function StatusDot({ status }: { status: unknown }) {
  const s = String(status)
  const color = s === 'Active' || s === 'active'
    ? 'bg-status-green'
    : s === 'Archived' || s === 'archived'
      ? 'bg-status-gray'
      : 'bg-status-yellow'
  return <span className={`w-1.5 h-1.5 rounded-full ${color}`} />
}
