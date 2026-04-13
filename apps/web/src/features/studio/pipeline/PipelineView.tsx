import type { JobRecord, StepObservedPayload, PipelineStep } from '@ship/ui'
import { StepCard, type StepStatus } from './StepCard'

export interface PipelineViewProps {
  job: JobRecord
  observations?: StepObservedPayload[]
}

/**
 * Derive step status from job state.
 */
function deriveStatus(step: PipelineStep, job: JobRecord): StepStatus {
  if (job.completed_steps?.includes(step.id)) return 'completed'
  if (job.current_step === step.id) {
    return job.status === 'failed' ? 'failed' : 'running'
  }
  return 'pending'
}

/**
 * Build a layered DAG from pipeline steps.
 * Returns an array of "columns" where each column contains steps
 * that can run in parallel (same dependency depth).
 */
function buildColumns(steps: PipelineStep[]): PipelineStep[][] {
  if (steps.length === 0) return []

  const stepMap = new Map(steps.map((s) => [s.id, s]))
  const depths = new Map<string, number>()

  function getDepth(id: string): number {
    if (depths.has(id)) return depths.get(id)!
    const step = stepMap.get(id)
    if (!step?.after || step.after.length === 0) {
      depths.set(id, 0)
      return 0
    }
    const maxParent = Math.max(...step.after.map((pid) => getDepth(pid)))
    const d = maxParent + 1
    depths.set(id, d)
    return d
  }

  for (const step of steps) getDepth(step.id)

  const maxDepth = Math.max(...depths.values())
  const columns: PipelineStep[][] = Array.from({ length: maxDepth + 1 }, () => [])
  for (const step of steps) {
    columns[depths.get(step.id)!].push(step)
  }

  return columns
}

export function PipelineView({ job, observations }: PipelineViewProps) {
  const steps = job.steps?.steps
  if (!steps || steps.length === 0) {
    return (
      <div className="text-sm text-muted-foreground/60 py-4">
        No pipeline steps defined.
      </div>
    )
  }

  const columns = buildColumns(steps)
  const obsMap = new Map(
    (observations ?? []).map((o) => [o.step_name, o])
  )

  return (
    <div className="flex items-start gap-0 overflow-x-auto py-2">
      {columns.map((column, colIdx) => (
        <div key={colIdx} className="flex items-center gap-0 shrink-0">
          {/* Arrow connector (skip before first column) */}
          {colIdx > 0 && <Arrow />}

          {/* Vertical stack for parallel steps */}
          <div className="flex flex-col gap-2">
            {column.map((step) => (
              <StepCard
                key={step.id}
                step={step}
                status={deriveStatus(step, job)}
                observation={obsMap.get(step.id)}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  )
}

function Arrow() {
  return (
    <div className="flex items-center px-1 shrink-0">
      <div className="h-px w-4 bg-border" />
      <div className="border-t-[4px] border-b-[4px] border-l-[6px] border-t-transparent border-b-transparent border-l-border" />
    </div>
  )
}
