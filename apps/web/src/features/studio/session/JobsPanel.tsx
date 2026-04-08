import { Briefcase } from 'lucide-react'
import type { JobRecord } from '@ship/ui'

const STATUS_COLORS: Record<string, string> = {
  pending: 'bg-muted-foreground/30',
  dispatched: 'bg-sky-500',
  completed: 'bg-emerald-500',
  gate_pending: 'bg-amber-500',
  blocked: 'bg-red-500',
  merged: 'bg-emerald-600',
  failed: 'bg-red-600',
}

export function JobsPanel({ jobs, onClose }: { jobs: JobRecord[]; onClose: () => void }) {
  return (
    <aside className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 h-9 border-b border-border shrink-0">
        <div className="flex items-center gap-1.5 text-[11px] font-medium text-foreground/80">
          <Briefcase className="size-3.5" />
          Jobs
        </div>
        <button onClick={onClose} className="text-xs text-muted-foreground hover:text-foreground transition-colors">
          Close
        </button>
      </div>
      <div className="flex-1 overflow-y-auto">
        {jobs.length === 0 ? (
          <div className="px-3 py-8 text-center text-xs text-muted-foreground">
            No jobs
          </div>
        ) : (
          <div className="py-1.5 space-y-0.5">
            {jobs.map((job) => (
              <div key={job.job_id} className="px-3 py-2 hover:bg-muted/30 transition-colors">
                <div className="flex items-center gap-2">
                  <span className={`size-2 rounded-full shrink-0 ${STATUS_COLORS[job.status] ?? 'bg-muted-foreground/30'}`} />
                  <span className="text-xs font-medium text-foreground/80 truncate flex-1">{job.slug}</span>
                  <span className="text-[10px] text-muted-foreground/60">{job.status}</span>
                </div>
                <div className="ml-4 mt-0.5 text-[10px] text-muted-foreground/50 truncate">
                  {job.agent} / {job.branch}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  )
}
