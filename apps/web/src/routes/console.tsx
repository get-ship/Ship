import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { Rail, type ViewId } from '#/features/console/Rail'
import { LeftPanel } from '#/features/console/LeftPanel'
import { FloatingEvents } from '#/features/console/FloatingEvents'
import { useDaemon } from '#/features/studio/hooks/useDaemon'

export const Route = createFileRoute('/console')({
  component: ConsoleShell,
  ssr: false,
})

function ConsoleShell() {
  const [activeView, setActiveView] = useState<ViewId>('jobs')
  const [panelOpen, setPanelOpen] = useState(false)
  const [eventsOpen, setEventsOpen] = useState(false)
  const [_chatOpen, setChatOpen] = useState(false)

  const { workspaces } = useDaemon()
  const activeWs = workspaces.find((w) => String(w.status) === 'Active' || String(w.status) === 'active')
  const [activeWorkspace, setActiveWorkspace] = useState<string | null>(null)

  // Auto-select first active workspace
  useEffect(() => {
    if (!activeWorkspace && activeWs) {
      setActiveWorkspace(activeWs.branch)
    }
  }, [activeWs, activeWorkspace])

  const togglePanel = useCallback(() => setPanelOpen((p) => !p), [])
  const toggleEvents = useCallback(() => setEventsOpen((p) => !p), [])
  const toggleChat = useCallback(() => setChatOpen((p) => !p), [])

  // Keyboard shortcuts
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'b' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        togglePanel()
      }
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        // Command palette — future
      }
      if (e.key === 'Escape') {
        setPanelOpen(false)
      }
      if (e.key === 'F11') {
        e.preventDefault()
        if (document.fullscreenElement) {
          void document.exitFullscreen()
        } else {
          void document.documentElement.requestFullscreen()
        }
      }
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [togglePanel])

  return (
    <div className="flex h-full w-full overflow-hidden">
      <Rail
        activeView={activeView}
        onViewChange={setActiveView}
        onTogglePanel={togglePanel}
        onToggleEvents={toggleEvents}
        onToggleChat={toggleChat}
        workspaceBranch={activeWorkspace}
      />

      <div className="flex-1 relative min-w-0 overflow-hidden">
        <LeftPanel
          open={panelOpen}
          onClose={() => setPanelOpen(false)}
          activeView={activeView}
          onViewChange={setActiveView}
          activeWorkspace={activeWorkspace}
          onWorkspaceChange={setActiveWorkspace}
        />

        {/* Content area — full bleed */}
        <main className="h-full overflow-auto">
          <ConsoleContent
            view={activeView}
            workspace={activeWorkspace}
          />
        </main>

        <FloatingEvents open={eventsOpen} onClose={() => setEventsOpen(false)} />
      </div>
    </div>
  )
}

function ConsoleContent({ view, workspace }: { view: ViewId; workspace: string | null }) {
  switch (view) {
    case 'jobs':
      return <JobsPlaceholder />
    case 'artifacts':
      return <ViewPlaceholder name="Artifacts" workspace={workspace} />
    case 'agent-editor':
      return <ViewPlaceholder name="Agent Editor" workspace={workspace} />
    case 'skills-ide':
      return <ViewPlaceholder name="Skills IDE" workspace={workspace} />
    case 'settings':
      return <ViewPlaceholder name="Settings" workspace={null} />
    case 'sprint-dag':
      return <ViewPlaceholder name="Sprint DAG" workspace={null} />
    default:
      return <ViewPlaceholder name={view} workspace={workspace} />
  }
}

function JobsPlaceholder() {
  const { jobs } = useDaemon()
  return (
    <div className="p-6">
      <h1 className="text-lg font-semibold mb-4">Jobs</h1>
      {jobs.length === 0 ? (
        <p className="text-sm text-muted-foreground">No jobs in queue.</p>
      ) : (
        <div className="space-y-2">
          {jobs.map((job) => (
            <div key={job.job_id} className="flex items-center gap-3 p-3 rounded-lg border border-border/60 bg-card">
              <StatusDot status={job.status} />
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium truncate">{job.slug}</div>
                <div className="text-xs text-muted-foreground">{job.agent} &middot; {job.branch}</div>
              </div>
              <span className="text-[10px] text-muted-foreground/60 shrink-0">{job.status}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

function ViewPlaceholder({ name, workspace }: { name: string; workspace: string | null }) {
  return (
    <div className="flex items-center justify-center h-full text-muted-foreground">
      <div className="text-center">
        <div className="text-lg font-medium">{name}</div>
        {workspace && <div className="text-xs mt-1">{workspace}</div>}
      </div>
    </div>
  )
}

function StatusDot({ status }: { status: string }) {
  const colors: Record<string, string> = {
    pending: 'bg-status-gray',
    dispatched: 'bg-status-blue',
    completed: 'bg-status-green',
    failed: 'bg-status-red',
    blocked: 'bg-status-yellow',
    gate_pending: 'bg-status-purple',
    merged: 'bg-status-green',
  }
  return <span className={`w-2 h-2 rounded-full shrink-0 ${colors[status] ?? 'bg-status-gray'}`} />
}
