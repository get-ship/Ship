import { Square } from 'lucide-react'
import { useUpdateJob } from '#/features/studio/hooks/useDaemonData'
import type { JobRecord } from '@ship/ui'

const STATUS_DOT: Record<string, string> = {
  pending: 'bg-muted-foreground/30',
  dispatched: 'bg-sky-500',
  completed: 'bg-emerald-500',
  gate_pending: 'bg-amber-500',
  blocked: 'bg-red-500',
  merged: 'bg-emerald-600',
  failed: 'bg-red-600',
}

const STATUS_TEXT: Record<string, string> = {
  pending: 'text-muted-foreground',
  dispatched: 'text-sky-500',
  completed: 'text-emerald-500',
  gate_pending: 'text-amber-500',
  blocked: 'text-red-500',
  merged: 'text-emerald-600',
  failed: 'text-red-600',
}

export function formatRelativeTime(iso: string): string {
  const diffMs = Date.now() - new Date(iso).getTime()
  if (diffMs < 0) return 'just now'
  const seconds = Math.floor(diffMs / 1000)
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  return `${Math.floor(hours / 24)}d ago`
}

export function WorkflowRunCard({ job }: { job: JobRecord }) {
  const dotColor = STATUS_DOT[job.status] ?? 'bg-muted-foreground/30'
  const textColor = STATUS_TEXT[job.status] ?? 'text-muted-foreground'
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
          {job.provider && <span> / {job.provider}</span>}
          {job.model && <span> / {job.model}</span>}
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
