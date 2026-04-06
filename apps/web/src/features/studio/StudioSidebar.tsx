import { X } from 'lucide-react'
import { Button } from '@ship/primitives'
import { useStudioShell, type PanelId } from './StudioShellContext'
import { useDaemon } from './hooks/useDaemon'

/** Sidebar panel that slides in from the left, VS Code style. */
export function StudioSidebar() {
  const { activePanel, setActivePanel } = useStudioShell()

  if (!activePanel) return null

  return (
    <aside className="w-64 shrink-0 bg-card border-r border-border/60 flex flex-col min-h-0 overflow-hidden animate-in slide-in-from-left-1 duration-100">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border/40">
        <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
          {PANEL_TITLES[activePanel]}
        </span>
        <Button variant="ghost" size="icon" className="size-6" onClick={() => setActivePanel(null)}>
          <X className="size-3" />
        </Button>
      </div>
      <div className="flex-1 overflow-auto">
        <PanelContent panel={activePanel} />
      </div>
    </aside>
  )
}

const PANEL_TITLES: Record<PanelId, string> = {
  workspaces: 'Workspaces',
  artifacts: 'Artifacts',
  search: 'Search',
  jobs: 'Jobs',
  settings: 'Settings',
}

function PanelContent({ panel }: { panel: PanelId }) {
  switch (panel) {
    case 'workspaces': return <WorkspacesPanel />
    case 'jobs': return <JobsPanel />
    case 'settings': return <SettingsPanel />
    case 'artifacts': return null // session sidebar renders its own
    case 'search': return <SearchPanel />
  }
}

function WorkspacesPanel() {
  const { workspaces, connected } = useDaemon()

  return (
    <div className="py-1">
      <div className="px-3 py-1.5 text-[10px] text-muted-foreground/60">
        {connected ? 'Connected' : 'Disconnected'}
      </div>
      {workspaces.length === 0 && (
        <div className="px-3 py-6 text-xs text-muted-foreground text-center">No workspaces</div>
      )}
      {workspaces.map((ws) => {
        const status = String(ws.status).toLowerCase()
        return (
          <div
            key={ws.branch}
            className="px-3 py-2 text-xs hover:bg-muted transition-colors"
          >
            <div className="font-medium truncate text-foreground">{ws.branch}</div>
            <div className="flex items-center gap-2 mt-0.5">
              <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${status === 'active' ? 'bg-status-green' : 'bg-status-gray'}`} />
              <span className="text-muted-foreground/60 truncate">
                {ws.active_agent ?? 'no agent'}
              </span>
            </div>
          </div>
        )
      })}
    </div>
  )
}

function JobsPanel() {
  const { jobs } = useDaemon()

  const STATUS_COLORS: Record<string, string> = {
    pending: 'bg-status-gray',
    dispatched: 'bg-status-blue',
    completed: 'bg-status-green',
    failed: 'bg-status-red',
    blocked: 'bg-status-yellow',
    gate_pending: 'bg-status-purple',
    merged: 'bg-status-green',
  }

  return (
    <div className="py-1">
      {jobs.length === 0 && (
        <div className="px-3 py-6 text-xs text-muted-foreground text-center">No jobs</div>
      )}
      {jobs.map((job) => (
        <div key={job.job_id} className="px-3 py-2 text-xs hover:bg-muted transition-colors">
          <div className="flex items-center gap-2">
            <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${STATUS_COLORS[job.status] ?? 'bg-status-gray'}`} />
            <span className="font-medium truncate text-foreground">{job.slug}</span>
          </div>
          <div className="text-muted-foreground/60 mt-0.5 pl-3.5 truncate">
            {job.agent} &middot; {job.branch}
          </div>
        </div>
      ))}
    </div>
  )
}

function SearchPanel() {
  return (
    <div className="p-3">
      <input
        type="text"
        placeholder="Search files..."
        className="w-full rounded-md border border-border/60 bg-background px-2.5 py-1.5 text-xs placeholder:text-muted-foreground/50 focus:outline-none focus:ring-1 focus:ring-primary/40"
      />
      <div className="mt-4 text-xs text-muted-foreground text-center">Type to search</div>
    </div>
  )
}

function SettingsPanel() {
  const { connected } = useDaemon()
  return (
    <div className="p-3 space-y-3">
      <div>
        <div className="text-[10px] text-muted-foreground/60 uppercase tracking-wider mb-1">Daemon</div>
        <div className="text-xs flex items-center gap-2">
          <span className={`w-1.5 h-1.5 rounded-full ${connected ? 'bg-status-green' : 'bg-status-red'}`} />
          {connected ? 'Connected' : 'Disconnected'}
        </div>
      </div>
    </div>
  )
}
