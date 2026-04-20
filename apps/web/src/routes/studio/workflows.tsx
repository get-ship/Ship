import { createFileRoute } from '@tanstack/react-router'
import { DaemonGuard } from '#/features/studio/DaemonGuard'
import { useState, useMemo } from 'react'
import { Plus, Play, LayoutList } from 'lucide-react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { CreateJobDialog } from '#/features/studio/CreateJobDialog'
import { WorkflowRunCard } from '#/features/studio/WorkflowRunCard'
import { WorkflowTemplates } from '#/features/studio/WorkflowTemplates'
import type { JobRecord } from '@ship/set'

export const Route = createFileRoute('/studio/workflows')({
  component: () => <DaemonGuard><WorkflowsPage /></DaemonGuard>,
  ssr: false,
})

type Section = 'runs' | 'templates' | 'history'

const ACTIVE_STATUSES = new Set(['pending', 'dispatched', 'gate_pending', 'blocked'])
const DONE_STATUSES = new Set(['completed', 'merged', 'failed'])

function WorkflowsPage() {
  const { jobs } = useDaemon()
  const [section, setSection] = useState<Section>('runs')
  const [createOpen, setCreateOpen] = useState(false)

  const activeRuns = useMemo(
    () => jobs.filter((j) => ACTIVE_STATUSES.has(j.status)),
    [jobs],
  )
  const history = useMemo(
    () => jobs.filter((j) => DONE_STATUSES.has(j.status)).sort(
      (a, b) => b.updated_at.localeCompare(a.updated_at),
    ),
    [jobs],
  )

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="font-display text-2xl font-bold text-foreground">Workflows</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Pipeline templates, runs, and history
            </p>
          </div>
          <Button onClick={() => setCreateOpen(true)}>
            <Play className="size-3.5" />
            New Run
          </Button>
        </div>

        {jobs.length > 0 && <RunStats jobs={jobs} />}

        <Tabs value={section} onValueChange={(v) => setSection(v as Section)} className="gap-4">
          <TabsList>
            <TabsTrigger value="runs">
              Active Runs
              {activeRuns.length > 0 && (
                <span className="ml-1.5 text-[10px] opacity-60">{activeRuns.length}</span>
              )}
            </TabsTrigger>
            <TabsTrigger value="templates">Templates</TabsTrigger>
            <TabsTrigger value="history">
              History
              {history.length > 0 && (
                <span className="ml-1.5 text-[10px] opacity-60">{history.length}</span>
              )}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="runs">
            <RunsList runs={activeRuns} onNewRun={() => setCreateOpen(true)} />
          </TabsContent>
          <TabsContent value="templates">
            <WorkflowTemplates />
          </TabsContent>
          <TabsContent value="history">
            <HistoryList runs={history} />
          </TabsContent>
        </Tabs>

        <CreateJobDialog open={createOpen} onOpenChange={setCreateOpen} />
      </div>
    </div>
  )
}

// ── Run stats ─────────────────────────────────────────────────────────────────

function RunStats({ jobs }: { jobs: JobRecord[] }) {
  const counts = useMemo(() => {
    const c = { pending: 0, dispatched: 0, completed: 0, failed: 0, blocked: 0, gate_pending: 0, merged: 0 }
    for (const job of jobs) {
      if (job.status in c) c[job.status as keyof typeof c]++
    }
    return c
  }, [jobs])

  return (
    <div className="flex items-center gap-4 mb-6 rounded-lg border border-border/60 bg-card/50 px-4 py-3">
      <StatDot color="bg-muted-foreground/30" label="Pending" count={counts.pending} />
      <StatDot color="bg-sky-500" label="Running" count={counts.dispatched} />
      <StatDot color="bg-amber-500" label="Gate" count={counts.gate_pending} />
      <StatDot color="bg-emerald-500" label="Done" count={counts.completed} />
      <StatDot color="bg-emerald-600" label="Merged" count={counts.merged} />
      <StatDot color="bg-red-500" label="Blocked" count={counts.blocked} />
      <StatDot color="bg-red-600" label="Failed" count={counts.failed} />
    </div>
  )
}

function StatDot({ color, label, count }: { color: string; label: string; count: number }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className={`size-2 rounded-full ${color}`} />
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-xs font-medium text-foreground">{count}</span>
    </div>
  )
}

// ── Active runs ───────────────────────────────────────────────────────────────

function RunsList({ runs, onNewRun }: { runs: JobRecord[]; onNewRun: () => void }) {
  if (runs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40 mb-4">
          <Play className="size-5 text-muted-foreground" />
        </div>
        <p className="text-sm font-medium text-foreground">No active runs</p>
        <p className="mt-1 text-xs text-muted-foreground max-w-xs">
          Launch a run from a pipeline template or create one manually.
        </p>
        <Button onClick={onNewRun} className="mt-4">
          <Plus className="size-3.5" />
          New Run
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {runs.map((job) => <WorkflowRunCard key={job.job_id} job={job} />)}
    </div>
  )
}

// ── History ───────────────────────────────────────────────────────────────────

function HistoryList({ runs }: { runs: JobRecord[] }) {
  if (runs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40 mb-4">
          <LayoutList className="size-5 text-muted-foreground" />
        </div>
        <p className="text-sm font-medium text-foreground">No completed runs</p>
        <p className="mt-1 text-xs text-muted-foreground max-w-xs">
          Completed, merged, and failed runs will appear here.
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {runs.map((job) => <WorkflowRunCard key={job.job_id} job={job} />)}
    </div>
  )
}
