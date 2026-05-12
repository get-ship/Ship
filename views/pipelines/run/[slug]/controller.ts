import { controller, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type StepRun = { id: string; state: string; step_type: string; agent?: string }
type ObsRow = { step_name: string; trigger: string; duration_secs: number; exit_code?: number; commits: number; done_declared: boolean; succeeded: boolean; verdict: string; log_tail?: string }
type TimelineEvent = { at: string; event_type: string; summary: string }
type RunDetail = { slug: string; agent: string; branch?: string; provider?: string; model?: string; updated_at: string; tone: string; status_text: string; state: string; job_id: string; error?: string; steps?: StepRun[] }
type RunCtx = { run: RunDetail; observations: ObsRow[]; timeline: TimelineEvent[] }

const EMPTY_RUN: RunDetail = { slug: '', agent: '', updated_at: '', tone: 'muted', status_text: '', state: '', job_id: '' }
const EMPTY: RunCtx = { run: EMPTY_RUN, observations: [], timeline: [] }
const read = (ctx: unknown): RunCtx => (unwrap(ctx as any) as RunCtx | undefined) ?? EMPTY

export default controller<Scope>({
  scope: ({ ctx }) => {
    const run = derived((s) => read(s.ctx).run)
    const observations = derived((s) => read(s.ctx).observations)
    const timeline = derived((s) => read(s.ctx).timeline)
    return { run, observations, timeline }
  },
  on: {
    stopRun: async (_s, jobId: string) => {
      await fetch('/api/v1/jobs/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
