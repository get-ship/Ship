import { useState } from 'react'
import { createFileRoute, Link } from '@tanstack/react-router'
import { Layers, Users, Briefcase, GitBranch, Circle, Plus, Trash2, Loader2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { useDaemon, useDeleteWorkspace, useActivateWorkspace } from '#/features/studio/hooks/useDaemon'
import { useGitStatus } from '#/features/studio/session/useGitInfo'
import { useSessionFiles } from '#/features/studio/session/useSessionFiles'
import { CreateWorkspaceDialog } from '#/features/studio/CreateWorkspaceDialog'
import { toast } from 'sonner'

export const Route = createFileRoute('/studio/')({
  component: DashboardPage,
  ssr: false,
})

function DashboardPage() {
  const { workspaces, agents, jobs, connected } = useDaemon()
  const { data: gitStatus } = useGitStatus()
  const { files } = useSessionFiles()
  const deleteMutation = useDeleteWorkspace()
  const activateMutation = useActivateWorkspace()
  const [createOpen, setCreateOpen] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)

  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')
  const runningJobs = jobs.filter((j) => j.status === 'dispatched')
  const pendingJobs = jobs.filter((j) => j.status === 'pending')

  const handleDelete = (branch: string) => {
    deleteMutation.mutate(branch, {
      onSuccess: () => {
        toast.success(`Workspace "${branch}" deleted`)
        setConfirmDelete(null)
      },
      onError: (err) => {
        toast.error(err instanceof Error ? err.message : 'Failed to delete workspace')
      },
    })
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto py-8 px-6">
        {/* Header */}
        <div className="flex items-center gap-3 mb-8">
          <img src="/ship-logos/ship_logo.svg" alt="" className="size-6" />
          <div>
            <h1 className="text-lg font-semibold">Ship Studio</h1>
            <p className="text-xs text-muted-foreground">
              {connected ? (
                <span className="flex items-center gap-1.5">
                  <Circle className="size-1.5 fill-emerald-500 text-emerald-500" />
                  Connected
                  {activeWs && <> &middot; {activeWs.branch}</>}
                  {activeWs?.active_agent && <> &middot; {activeWs.active_agent}</>}
                </span>
              ) : (
                <span className="flex items-center gap-1.5">
                  <Circle className="size-1.5 fill-red-500 text-red-500" />
                  Daemon not connected
                </span>
              )}
            </p>
          </div>
        </div>

        {/* Quick nav cards */}
        <div className="grid grid-cols-2 gap-3 mb-8">
          <NavCard
            to="/studio/session"
            icon={Layers}
            label="Session"
            detail={`${files.length} artifact${files.length !== 1 ? 's' : ''}`}
          />
          <NavCard
            to="/studio/git"
            icon={GitBranch}
            label="Git"
            detail={gitStatus ? (gitStatus.clean ? 'Clean' : `${countChanges(gitStatus)} changed`) : 'Loading...'}
          />
          <NavCard
            to="/studio/agents"
            icon={Users}
            label="Agents"
            detail={`${agents.length} registered`}
          />
          <NavCard
            to="/studio/jobs"
            icon={Briefcase}
            label="Jobs"
            detail={runningJobs.length > 0
              ? `${runningJobs.length} running`
              : pendingJobs.length > 0
                ? `${pendingJobs.length} pending`
                : `${jobs.length} total`}
          />
        </div>

        {/* Workspaces */}
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Workspaces</h2>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 gap-1 text-xs"
              onClick={() => setCreateOpen(true)}
            >
              <Plus className="size-3" />
              New
            </Button>
          </div>
          {workspaces.length > 0 ? (
            <div className="space-y-1">
              {workspaces.map((ws) => {
                const status = String(ws.status).toLowerCase()
                const isActive = status === 'active'
                const isDeleting = deleteMutation.isPending && confirmDelete === ws.branch
                return (
                  <div key={ws.branch} className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-muted/30 transition-colors group">
                    <span className={`size-2 rounded-full shrink-0 ${isActive ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
                    <span className="text-sm font-medium flex-1 truncate">{ws.branch}</span>
                    {ws.active_agent && (
                      <span className="text-[10px] text-muted-foreground/60">{ws.active_agent}</span>
                    )}
                    {!isActive && (
                      <button
                        onClick={() => activateMutation.mutate(ws.branch)}
                        disabled={activateMutation.isPending}
                        className="text-[10px] text-muted-foreground/40 hover:text-foreground opacity-0 group-hover:opacity-100 transition-opacity"
                      >
                        Activate
                      </button>
                    )}
                    {confirmDelete === ws.branch ? (
                      <span className="flex items-center gap-1.5">
                        <button
                          onClick={() => handleDelete(ws.branch)}
                          disabled={isDeleting}
                          className="text-[10px] text-destructive hover:text-destructive/80"
                        >
                          {isDeleting ? <Loader2 className="size-3 animate-spin" /> : 'Confirm'}
                        </button>
                        <button
                          onClick={() => setConfirmDelete(null)}
                          className="text-[10px] text-muted-foreground/40 hover:text-foreground"
                        >
                          Cancel
                        </button>
                      </span>
                    ) : (
                      <button
                        onClick={() => setConfirmDelete(ws.branch)}
                        className="text-muted-foreground/30 hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
                        title="Delete workspace"
                      >
                        <Trash2 className="size-3.5" />
                      </button>
                    )}
                  </div>
                )
              })}
            </div>
          ) : (
            <p className="text-xs text-muted-foreground/50 px-3 py-4 text-center">
              No workspaces yet
            </p>
          )}
        </div>
      </div>

      <CreateWorkspaceDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}

function NavCard({ to, icon: Icon, label, detail }: {
  to: string
  icon: typeof Layers
  label: string
  detail: string
}) {
  return (
    <Link
      to={to}
      className="flex items-center gap-3 px-4 py-3 rounded-xl border border-border/60 bg-card hover:bg-muted/30 hover:border-border transition-colors no-underline group"
    >
      <Icon className="size-5 text-muted-foreground group-hover:text-foreground transition-colors" />
      <div>
        <div className="text-sm font-medium">{label}</div>
        <div className="text-[10px] text-muted-foreground/60">{detail}</div>
      </div>
    </Link>
  )
}

function countChanges(gs: { staged?: unknown[]; modified?: unknown[]; untracked?: unknown[] }): number {
  return (gs.staged?.length ?? 0) + (gs.modified?.length ?? 0) + (gs.untracked?.length ?? 0)
}
