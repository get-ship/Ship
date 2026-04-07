import { createFileRoute } from '@tanstack/react-router'
import { useState, useMemo } from 'react'
import { LayoutList, Plus, Square } from 'lucide-react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { useDaemon, useUpdateJob } from '#/features/studio/hooks/useDaemonData'
import { CreateJobDialog } from '#/features/studio/CreateJobDialog'
import type { JobRecord, JobStatus } from '@ship/ui'

export const Route = createFileRoute('/studio/jobs')({
  component: JobsPage,
  ssr: false,
})

const STATUS_COLORS: Record<string, string> = {
  pending: 'bg-muted-foreground/30',
  dispatched: 'bg-sky-500',
  completed: 'bg-emerald-500',
  gate_pending: 'bg-amber-500',
  blocked: 'bg-red-500',
  merged: 'bg-emerald-600',
  failed: 'bg-red-600',
}

const STATUS_TEXT_COLORS: Record<string, string> = {
  pending: 'text-muted-foreground',
  dispatched: 'text-sky-500',
  completed: 'text-emerald-500',
  gate_pending: 'text-amber-500',
  blocked: 'text-red-500',
  merged: 'text-emerald-600',
  failed: 'text-red-600',
}

type FilterTab = 'all' | JobStatus

const FILTER_TABS: { value: FilterTab; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'pending', label: 'Pending' },
  { value: 'dispatched', label: 'Running' },
  { value: 'completed', label: 'Completed' },
  { value: 'failed', label: 'Failed' },
]

function formatRelativeTime(iso: string): string {
  const now = Date.now()
  const then = new Date(iso).getTime()
  const diffMs = now - then
  if (diffMs < 0) return 'just now'

  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'

  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`

  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

function JobsPage() {
  const { jobs } = useDaemon()
  const [filter, setFilter] = useState<FilterTab>('all')
  const [createOpen, setCreateOpen] = useState(false)

  const counts = useMemo(() => {
    const c = { pending: 0, dispatched: 0, completed: 0, failed: 0, blocked: 0, gate_pending: 0, merged: 0 }
    for (const job of jobs) {
      if (job.status in c) c[job.status as keyof typeof c]++
    }
    return c
  }, [jobs])

  const filtered = useMemo(() => {
    if (filter === 'all') return jobs
    return jobs.filter((j) => j.status === filter)
  }, [jobs, filter])

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="font-display text-2xl font-bold text-foreground">Jobs</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {jobs.length} job{jobs.length !== 1 ? 's' : ''} in queue
            </p>
          </div>
          <Button onClick={() => setCreateOpen(true)}>
            <Plus className="size-3.5" />
            New Job
          </Button>
        </div>

        {/* Stats bar */}
        {jobs.length > 0 && (
          <div className="flex items-center gap-4 mb-6 rounded-lg border border-border/60 bg-card/50 px-4 py-3">
            <StatDot color="bg-muted-foreground/30" label="Pending" count={counts.pending} />
            <StatDot color="bg-sky-500" label="Running" count={counts.dispatched} />
            <StatDot color="bg-amber-500" label="Gate" count={counts.gate_pending} />
            <StatDot color="bg-emerald-500" label="Completed" count={counts.completed} />
            <StatDot color="bg-emerald-600" label="Merged" count={counts.merged} />
            <StatDot color="bg-red-500" label="Blocked" count={counts.blocked} />
            <StatDot color="bg-red-600" label="Failed" count={counts.failed} />
          </div>
        )}

        {/* Filter tabs + content */}
        <Tabs value={filter} onValueChange={(v) => setFilter(v as FilterTab)} className="gap-4">
          <TabsList>
            {FILTER_TABS.map((tab) => (
              <TabsTrigger key={tab.value} value={tab.value}>
                {tab.label}
                {tab.value !== 'all' && counts[tab.value as keyof typeof counts] > 0 && (
                  <span className="ml-1 text-[10px] opacity-60">
                    {counts[tab.value as keyof typeof counts]}
                  </span>
                )}
              </TabsTrigger>
            ))}
          </TabsList>

          {FILTER_TABS.map((tab) => (
            <TabsContent key={tab.value} value={tab.value}>
              <JobList
                jobs={tab.value === 'all' ? filtered : filtered}
                onCreateNew={() => setCreateOpen(true)}
              />
            </TabsContent>
          ))}
        </Tabs>

        <CreateJobDialog open={createOpen} onOpenChange={setCreateOpen} />
      </div>
    </div>
  )
}

function JobList({ jobs, onCreateNew }: { jobs: JobRecord[]; onCreateNew: () => void }) {
  if (jobs.length === 0) {
    return <EmptyState onCreateNew={onCreateNew} />
  }

  return (
    <div className="space-y-2">
      {jobs.map((job) => (
        <JobCard key={job.job_id} job={job} />
      ))}
    </div>
  )
}

function EmptyState({ onCreateNew }: { onCreateNew: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40 mb-4">
        <LayoutList className="size-5 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">No jobs in queue</p>
      <p className="mt-1 text-xs text-muted-foreground max-w-xs">
        Create a job to dispatch work to an agent. Jobs track status, dependencies, and errors.
      </p>
      <Button onClick={onCreateNew} className="mt-4">
        <Plus className="size-3.5" />
        Create Job
      </Button>
    </div>
  )
}

function JobCard({ job }: { job: JobRecord }) {
  const dotColor = STATUS_COLORS[job.status] ?? 'bg-muted-foreground/30'
  const textColor = STATUS_TEXT_COLORS[job.status] ?? 'text-muted-foreground'
  const updateJob = useUpdateJob()
  const canStop = job.status === 'dispatched' || job.status === 'pending'

  return (
    <div className="flex items-start gap-3 rounded-lg border border-border/60 bg-card p-4 hover:border-border transition-colors">
      <span className={`size-2.5 rounded-full shrink-0 mt-1.5 ${dotColor}`} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-semibold text-foreground truncate">{job.slug}</span>
          <span className={`text-[10px] font-medium ${textColor}`}>
            {job.status.replace('_', ' ')}
          </span>
        </div>
        <div className="text-xs text-muted-foreground/60 mt-0.5 truncate">
          {job.agent} / {job.branch}
        </div>
        {job.blocker && (
          <div className="mt-2 rounded-md border border-red-500/20 bg-red-500/5 px-3 py-1.5">
            <p className="text-xs text-red-500">Blocked: {job.blocker}</p>
          </div>
        )}
        {job.error && (
          <div className="mt-2 rounded-md border border-red-600/20 bg-red-600/5 px-3 py-1.5">
            <p className="text-xs text-red-600">Error: {job.error}</p>
          </div>
        )}
        {job.depends_on && job.depends_on.length > 0 && (
          <div className="mt-1.5 flex items-center gap-1 flex-wrap">
            <span className="text-[10px] text-muted-foreground/50">depends on:</span>
            {job.depends_on.map((dep) => (
              <span
                key={dep}
                className="text-[10px] px-1.5 py-0.5 rounded border border-border text-muted-foreground"
              >
                {dep}
              </span>
            ))}
          </div>
        )}
        {job.steps && job.steps.steps.length > 0 && (
          <div className="mt-2 flex items-center gap-1">
            {job.steps.steps.map((step) => {
              const isComplete = job.completed_steps?.includes(step.id)
              const isCurrent = job.current_step === step.id
              return (
                <div
                  key={step.id}
                  className={`flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium ${
                    isComplete ? 'bg-emerald-500/10 text-emerald-600'
                    : isCurrent ? 'bg-sky-500/10 text-sky-600 ring-1 ring-sky-500/30'
                    : 'bg-muted/30 text-muted-foreground/60'
                  }`}
                >
                  {step.id}
                </div>
              )
            })}
          </div>
        )}
      </div>
      <div className="flex flex-col items-end gap-1 shrink-0 mt-0.5">
        <span className="text-[11px] text-muted-foreground/50">
          {formatRelativeTime(job.created_at)}
        </span>
        {canStop && (
          <button
            onClick={() => updateJob.mutate({ job_id: job.job_id, status: 'failed', error: 'stopped by user' })}
            disabled={updateJob.isPending}
            className="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-50"
          >
            <Square className="size-2.5" />
            Stop
          </button>
        )}
      </div>
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
