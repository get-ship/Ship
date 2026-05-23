import { controller, derived, live, unwrap } from '@ship/sui-runtime'
import type { MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// `on:` handlers receive the writable state proxy; annotate explicitly
// since the OnSection union blocks contextual inference.
type S = MutableState<Scope>

type StepRun = { id: string; state: string; step_type: string; agent?: string }
type StepRunWithFlag = StepRun & { isLast: boolean }
type ObsRow = { step_name: string; trigger: string; duration_secs: number; exit_code?: number; commits: number; done_declared: boolean; succeeded: boolean; verdict: string; log_tail?: string }
type TimelineEvent = { at: string; event_type: string; summary: string }
type RunDetail = { slug: string; agent: string; branch?: string; provider?: string; model?: string; updated_at: string; tone: string; status_text: string; state: string; job_id: string; error?: string; steps?: StepRun[] }
type RunCtx = { run: RunDetail; observations: ObsRow[]; timeline: TimelineEvent[] }

const EMPTY_RUN: RunDetail = { slug: '', agent: '', updated_at: '', tone: 'muted', status_text: '', state: '', job_id: '' }
const EMPTY: RunCtx = { run: EMPTY_RUN, observations: [], timeline: [] }
const read = (ctx: unknown): RunCtx => (unwrap(ctx as any) as RunCtx | undefined) ?? EMPTY

export default controller<Scope>({
  scope: ({ ship, route }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    // BUG (latent, pre-existing): `route` is a `RouteSignals` object, not a
    // function — `route()` throws at runtime, and `.params.slug` is a Signal
    // (should be `route.params.slug?.()`). Other studio views use the correct
    // `route.params.X()` form. Cast preserves current behavior; the runtime
    // bug needs a separate behavior fix (out of scope for this type pass).
    const ctx = live(() => s.pipelines.run({ slug: (route as unknown as () => typeof route)().params.slug }))
    const run = derived(() => read(ctx()).run)
    const observations = derived(() => read(ctx()).observations)
    const timeline = derived(() => read(ctx()).timeline)
    const steps = derived<StepRunWithFlag[]>(() => {
      const list = read(ctx()).run.steps ?? []
      const last = list.length - 1
      return list.map((step, i) => ({ ...step, isLast: i === last }))
    })
    const errorFirstLine = derived(() => {
      const err = read(ctx()).run.error ?? ''
      const nl = err.indexOf('\n')
      return nl >= 0 ? err.slice(0, nl) : err
    })
    const errorMultiline = derived(() => (read(ctx()).run.error ?? '').includes('\n'))
    return { ctx, run, observations, timeline, steps, errorFirstLine, errorMultiline }
  },
  on: {
    stopRun: async (_s: S, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
