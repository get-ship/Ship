import { createFileRoute } from '@tanstack/react-router'
import { Briefcase } from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemon'

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

function JobsPage() {
  const { jobs } = useDaemon()

  return (
    <div className="flex-1 overflow-auto p-6">
      <div className="flex items-center gap-2 mb-6">
        <Briefcase className="size-5 text-muted-foreground" />
        <h1 className="text-lg font-semibold">Jobs</h1>
        <span className="text-xs text-muted-foreground">{jobs.length}</span>
      </div>

      {jobs.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
          <Briefcase className="size-8 opacity-30 mb-3" />
          <p className="text-sm">No jobs in queue</p>
        </div>
      ) : (
        <div className="space-y-1">
          {jobs.map((job) => (
            <div key={job.job_id} className="flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-muted/30 transition-colors">
              <span className={`size-2 rounded-full shrink-0 ${STATUS_COLORS[job.status] ?? 'bg-muted-foreground/30'}`} />
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium truncate">{job.slug}</div>
                <div className="text-xs text-muted-foreground/60 truncate">{job.agent} / {job.branch}</div>
              </div>
              <span className="text-[10px] text-muted-foreground/50 shrink-0">{job.status}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
