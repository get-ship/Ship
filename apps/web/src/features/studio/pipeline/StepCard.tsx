import type { PipelineStep, StepObservedPayload, StepType, StepVerdict, StepTrigger } from '@ship/set'
import { Badge } from '@ship/primitives'

export type StepStatus = 'completed' | 'running' | 'pending' | 'failed'

export interface StepCardProps {
  step: PipelineStep
  status: StepStatus
  observation?: StepObservedPayload
}

const STATUS_RING: Record<StepStatus, string> = {
  completed: 'border-emerald-500/40 bg-emerald-500/5',
  running: 'border-sky-500/40 bg-sky-500/5 ring-1 ring-sky-500/20',
  pending: 'border-border/60 bg-muted/20',
  failed: 'border-red-500/40 bg-red-500/5',
}

const STATUS_ICON: Record<StepStatus, string> = {
  completed: '\u2713',
  running: '\u27F3',
  pending: '\u25CB',
  failed: '\u2717',
}

const STATUS_COLOR: Record<StepStatus, string> = {
  completed: 'text-emerald-600',
  running: 'text-sky-600',
  pending: 'text-muted-foreground/50',
  failed: 'text-red-600',
}

const TYPE_LABEL: Record<StepType, string> = {
  agent: 'Agent',
  gate: 'Gate',
  merge: 'Merge',
  human: 'Human',
  run: 'Run',
}

const VERDICT_VARIANT: Record<StepVerdict, 'default' | 'destructive' | 'outline'> = {
  advance: 'default',
  failed: 'destructive',
  inconclusive: 'outline',
}

const VERDICT_LABEL: Record<StepVerdict, string> = {
  advance: 'advance',
  failed: 'failed',
  inconclusive: 'inconclusive',
}

const TRIGGER_LABEL: Record<StepTrigger, string> = {
  done_declared: 'done declared',
  process_exit: 'process exit',
  turn_settled: 'turn settled',
  timeout: 'timeout',
  channel_closed: 'channel closed',
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${Math.round(secs)}s`
  const m = Math.floor(secs / 60)
  const s = Math.round(secs % 60)
  return s > 0 ? `${m}m${s}s` : `${m}m`
}

export function StepCard({ step, status, observation }: StepCardProps) {
  return (
    <div
      className={`flex flex-col gap-1.5 rounded-lg border px-3 py-2.5 min-w-[140px] max-w-[200px] transition-colors ${STATUS_RING[status]}`}
    >
      {/* Header: name + type badge */}
      <div className="flex items-center gap-1.5">
        <span className={`text-sm font-mono ${STATUS_COLOR[status]}`}>
          {STATUS_ICON[status]}
        </span>
        <span className="text-sm font-semibold text-foreground truncate">
          {step.id}
        </span>
      </div>

      <div className="flex items-center gap-1.5">
        <span className="text-[10px] font-medium text-muted-foreground/70 uppercase tracking-wide">
          {TYPE_LABEL[step.type]}
        </span>
        <span className={`text-[10px] ${STATUS_COLOR[status]}`}>
          {status}
        </span>
      </div>

      {/* Evidence section */}
      {observation && (
        <div className="flex flex-col gap-1 mt-0.5 border-t border-border/40 pt-1.5">
          <div className="flex items-center gap-1 flex-wrap">
            <Badge variant={VERDICT_VARIANT[observation.verdict]} className="text-[9px]">
              {VERDICT_LABEL[observation.verdict]}
            </Badge>
            <span className="text-[9px] text-muted-foreground/60">
              {TRIGGER_LABEL[observation.trigger]}
            </span>
          </div>
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground/70">
            {observation.commits_since_start != null && observation.commits_since_start > 0 && (
              <span>{observation.commits_since_start} commit{observation.commits_since_start !== 1 ? 's' : ''}</span>
            )}
            <span>{formatDuration(observation.duration_secs)}</span>
          </div>
        </div>
      )}
    </div>
  )
}
