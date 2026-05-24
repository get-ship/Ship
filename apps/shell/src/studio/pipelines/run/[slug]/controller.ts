import { controller, derived, live } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

type StepRun = { id: string; state: string; step_type: string; agent?: string }
type StepRunWithFlag = StepRun & { isLast: boolean }
type ObsRow = { step_name: string; trigger: string; duration_secs: number; exit_code?: number; commits: number; done_declared: boolean; succeeded: boolean; verdict: string; log_tail?: string }
type TimelineEvent = { at: string; event_type: string; summary: string }
type RunDetail = { slug: string; agent: string; branch?: string; provider?: string; model?: string; updated_at: string; tone: string; status_text: string; state: string; job_id: string; error?: string; steps?: StepRun[] }
type RunCtx = { run: RunDetail; observations: ObsRow[]; timeline: TimelineEvent[] }

const EMPTY_RUN: RunDetail = { slug: '', agent: '', updated_at: '', tone: 'muted', status_text: '', state: '', job_id: '' }
const EMPTY: RunCtx = { run: EMPTY_RUN, observations: [], timeline: [] }
// `envelope` is the live signal's current value (`runCtx()`): pull the `ok`
// value, falling back to EMPTY while loading / on error. Guards `.value` for
// null on the run op (returns 404→error, never a null ok value).
const read = (envelope: unknown): RunCtx => {
  const env = envelope as { kind?: string; value?: Partial<RunCtx> } | undefined
  if (env?.kind !== 'ok' || !env.value) return EMPTY
  // Merge over EMPTY: the payload omits empty collections (observations,
  // timeline) — fall back so `{#for}` / `.length` never hit undefined.
  return { ...EMPTY, ...env.value }
}

export default controller({
  scope: ({ route }: ViewScope<Scope>) => {
    // `route` is a RouteSignals object; `params.slug` is a Signal — read it via
    // `route.params.slug?.()` (matches every other studio dynamic-route view).
    // Requires the `/studio/pipelines/run/:slug` pattern registered via
    // defineRoutes at shell startup; absent that, slug is undefined.
    // The shell SDK object exposes no `pipelines` surface, so subscribe to the
    // run op by name via `liveExpr` (the views/events pattern).
    const slug = derived(() => String(route.params.slug?.() ?? ''))
    const runCtx = live(() => liveExpr<RunCtx>(`ship.pipelines.run(${JSON.stringify({ slug: slug() })})`))
    const run = derived(() => read(runCtx()).run)
    const observations = derived(() => read(runCtx()).observations)
    const timeline = derived(() => read(runCtx()).timeline)
    const steps = derived<StepRunWithFlag[]>(() => {
      const list = read(runCtx()).run.steps ?? []
      const last = list.length - 1
      return list.map((step, i) => ({ ...step, isLast: i === last }))
    })
    const errorFirstLine = derived(() => {
      const err = read(runCtx()).run.error ?? ''
      const nl = err.indexOf('\n')
      return nl >= 0 ? err.slice(0, nl) : err
    })
    const errorMultiline = derived(() => (read(runCtx()).run.error ?? '').includes('\n'))
    return { runCtx, run, observations, timeline, steps, errorFirstLine, errorMultiline }
  },
  on: {
    stopRun: async (_s, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
