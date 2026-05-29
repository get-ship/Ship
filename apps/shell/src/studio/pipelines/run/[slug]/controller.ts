import { controller, derived, live } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship, ops } from '../../../../ops'
import type { LiveSource } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

function formatRelative(iso: string, now = Date.now()): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return iso
  const diff = now - t
  const abs = Math.abs(diff)
  const suffix = diff >= 0 ? 'ago' : 'from now'
  if (abs < MINUTE) return 'just now'
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`
  const days = Math.floor(abs / DAY)
  if (days < 30) return `${days}d ${suffix}`
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`
  return `${Math.floor(days / 365)}y ${suffix}`
}

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
    const runCtx = live(() => ship.pipelines.run({ slug: slug() }) as LiveSource<RunCtx>)
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
    return { runCtx, run, observations, timeline, steps, errorFirstLine, errorMultiline, formatRelative }
  },
  on: {
    stopRun: async (_s, jobId: string) => {
      // FLAG (API arc): EndTurn (turn-complete); old call sent job_id where
      // end_job reads slug (likely no-op). True stop wants agent.Stop.
      try { await ops.pipelines.endTurn({ slug: jobId }) } catch { /* best-effort */ }
    },
  },
})
