import { useState } from 'react'
import { createFileRoute, Link } from '@tanstack/react-router'
import { Layers, Users, Briefcase, GitBranch, Circle, Plus, Trash2, Loader2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { useDaemon, useDeleteWorkspace, useActivateWorkspace } from '#/features/studio/hooks/useDaemonData'
import { useShipQuery } from '#/features/studio/hooks/useShipQuery'
import { useGitStatus } from '#/features/studio/session/useGitInfo'
import { useSessionFiles } from '#/features/studio/session/useSessionFiles'
import { CreateWorkspaceDialog } from '#/features/studio/CreateWorkspaceDialog'
import { toast } from 'sonner'
import type { Workspace } from '@ship/ui'

import { DaemonGuard } from '#/features/studio/DaemonGuard'

export const Route = createFileRoute('/studio/')({
  component: () => <DaemonGuard><DashboardPage /></DaemonGuard>,
  ssr: false,
})

function DashboardPage() {
  const { workspaces, jobs, connected } = useDaemon()
  const { data: gitStatus } = useGitStatus()
  const { files } = useSessionFiles()
  const deleteMutation = useDeleteWorkspace()
  const activateMutation = useActivateWorkspace()
  const [createOpen, setCreateOpen] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null)

  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0]
  const activeWsId = activeWs?.name ?? null

  // Config counts from active workspace
  const agentsPath = activeWsId ? `/workspaces/${encodeURIComponent(activeWsId)}/agents` : null
  const skillsPath = activeWsId ? `/workspaces/${encodeURIComponent(activeWsId)}/skills` : null
  const { data: agentsConfig } = useShipQuery<{ agents: Array<{ id: string }> }>(agentsPath)
  const { data: skillsConfig } = useShipQuery<{ skills: Array<{ id: string }> }>(skillsPath)

  const agentCount = agentsConfig?.agents.length ?? 0
  const skillCount = skillsConfig?.skills.length ?? 0

  const runningJobs = jobs.filter((j) => j.status === 'dispatched')
  const pendingJobs = jobs.filter((j) => j.status === 'pending')

  // Hide job/* branches — those are agent work branches, not user workspaces
  const displayWorkspaces = workspaces.filter((ws) => !ws.name.startsWith('job/'))

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
                  {activeWs && <> &middot; {activeWs.name}</>}
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
            detail={
              agentCount > 0 || skillCount > 0
                ? `${agentCount} agent${agentCount !== 1 ? 's' : ''} · ${skillCount} skill${skillCount !== 1 ? 's' : ''}`
                : activeWsId ? 'No agents configured' : 'Select a workspace to get started'
            }
          />
          <NavCard
            to="/studio/workflows"
            icon={Briefcase}
            label="Workflows"
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
          {displayWorkspaces.length > 0 ? (
            <div className="space-y-1">
              {displayWorkspaces.map((ws) => {
                const status = String(ws.status).toLowerCase()
                const isActive = status === 'active'
                const isDeleting = deleteMutation.isPending && confirmDelete === ws.name
                return (
                  <WorkspaceRow
                    key={ws.name}
                    ws={ws}
                    isActive={isActive}
                    isDeleting={isDeleting}
                    isConfirmDelete={confirmDelete === ws.name}
                    activatePending={activateMutation.isPending}
                    onActivate={() => activateMutation.mutate(ws.name)}
                    onDelete={() => handleDelete(ws.name)}
                    onConfirmDelete={() => setConfirmDelete(ws.name)}
                    onCancelDelete={() => setConfirmDelete(null)}
                  />
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

interface WorkspaceRowProps {
  ws: Workspace
  isActive: boolean
  isDeleting: boolean
  isConfirmDelete: boolean
  activatePending: boolean
  onActivate: () => void
  onDelete: () => void
  onConfirmDelete: () => void
  onCancelDelete: () => void
}

function WorkspaceRow({
  ws, isActive, isDeleting, isConfirmDelete, activatePending,
  onActivate, onDelete, onConfirmDelete, onCancelDelete,
}: WorkspaceRowProps) {
  const gitPath = ws.is_worktree
    ? `/workspaces/${encodeURIComponent(ws.name)}/git/status`
    : null
  const { data: gitRaw } = useShipQuery<{ output: string }>(gitPath)
  const gitClean = gitRaw ? isGitOutputClean(gitRaw.output) : null

  return (
    <div className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-muted/30 transition-colors group">
      <span className={`size-2 rounded-full shrink-0 ${isActive ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
      <span className="text-sm font-medium flex-1 truncate">{ws.name}</span>

      {/* Git status badge */}
      {ws.is_worktree && gitRaw && (
        <span className={`text-[10px] ${gitClean ? 'text-emerald-500' : 'text-amber-500'}`}>
          {gitClean ? 'clean' : 'dirty'}
        </span>
      )}

      {ws.active_agent && (
        <span className="text-[10px] text-muted-foreground/60">{ws.active_agent}</span>
      )}
      {!isActive && (
        <button
          onClick={onActivate}
          disabled={activatePending}
          className="text-[10px] text-muted-foreground/40 hover:text-foreground opacity-0 group-hover:opacity-100 transition-opacity"
        >
          Activate
        </button>
      )}
      {isConfirmDelete ? (
        <span className="flex items-center gap-1.5">
          <button
            onClick={onDelete}
            disabled={isDeleting}
            className="text-[10px] text-destructive hover:text-destructive/80"
          >
            {isDeleting ? <Loader2 className="size-3 animate-spin" /> : 'Confirm'}
          </button>
          <button
            onClick={onCancelDelete}
            className="text-[10px] text-muted-foreground/40 hover:text-foreground"
          >
            Cancel
          </button>
        </span>
      ) : (
        <button
          onClick={onConfirmDelete}
          className="text-muted-foreground/30 hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
          title="Delete workspace"
        >
          <Trash2 className="size-3.5" />
        </button>
      )}
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

function isGitOutputClean(output: string): boolean {
  return (
    !output.includes('Changes to be committed') &&
    !output.includes('Changes not staged') &&
    !output.includes('Untracked files')
  )
}
