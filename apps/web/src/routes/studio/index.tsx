import { createFileRoute, Link } from '@tanstack/react-router'
import { Layers, Users, Briefcase, GitBranch, Circle } from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { useGitStatus } from '#/features/studio/session/useGitInfo'
import { useSessionFiles } from '#/features/studio/session/useSessionFiles'

export const Route = createFileRoute('/studio/')({
  component: DashboardPage,
  ssr: false,
})

function DashboardPage() {
  const { workspaces, agents, jobs, connected } = useDaemon()
  const { data: gitStatus } = useGitStatus()
  const { files } = useSessionFiles()

  const activeWs = workspaces.find((w) => String(w.status).toLowerCase() === 'active')
  const runningJobs = jobs.filter((j) => j.status === 'dispatched')
  const pendingJobs = jobs.filter((j) => j.status === 'pending')

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
        {workspaces.length > 0 && (
          <div>
            <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wider mb-3">Workspaces</h2>
            <div className="space-y-1">
              {workspaces.map((ws) => {
                const status = String(ws.status).toLowerCase()
                return (
                  <div key={ws.branch} className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-muted/30 transition-colors">
                    <span className={`size-2 rounded-full shrink-0 ${status === 'active' ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`} />
                    <span className="text-sm font-medium flex-1 truncate">{ws.branch}</span>
                    {ws.active_agent && (
                      <span className="text-[10px] text-muted-foreground/60">{ws.active_agent}</span>
                    )}
                  </div>
                )
              })}
            </div>
          </div>
        )}
      </div>
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
