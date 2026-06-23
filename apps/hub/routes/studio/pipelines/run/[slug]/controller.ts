// Pipeline run detail (`/studio/pipelines/run/:slug`). Hub port of
// apps/shell/routes/studio/pipelines/run/[slug]/controller.ts.
//
// The shell read the deprecated live op `ship.pipelines.run` (no mux resolver →
// 404). This reads the TYPED unary `pipelines.run` → PipelineRunDetailResponse
// { run: PipelineRunRow, steps: PipelineStepRunRow[] }, wrapped in `live(() => …)`
// so the template's `{#async runCtx}` block drives loading/error/ok.
//
// The typed projection carries the run row (slug/status/startedAt/endedAt/
// outcome) + step rows (stepId/status/startedAt/completedAt/dependsOn/stepType/
// attempt). It DELIBERATELY EXCLUDES the GUI view-model the shell stream added:
// agent/branch/provider/model, error text, step observations, the event
// timeline, and the status_text/tone tokens. Those sections are dropped; tone is
// derived from `status`. The live log stream (a separate REST SSE endpoint, not a
// proto op) is deferred to the live-tail follow-up — see the port notes.
//
// Cancel routes to the unary `pipelines.cancel` (slug-keyed) — the correct stop
// verb (the shell routed to EndTurn, a likely no-op).
import { controller, derived, live, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
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

function formatDuration(startIso?: string, endIso?: string): string {
  if (!startIso || !endIso) return ''
  const start = Date.parse(startIso)
  const end = Date.parse(endIso)
  if (Number.isNaN(start) || Number.isNaN(end) || end < start) return ''
  const secs = Math.round((end - start) / 1000)
  if (secs < 60) return `${secs}s`
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`
  return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`
}

const RUN_STATE: Record<string, string> = {
  pending: 'running', dispatched: 'running', running: 'running',
  gate_pending: 'awaiting', blocked: 'awaiting',
  completed: 'ok', merged: 'ok', failed: 'err', cancelled: 'none',
}
const RUN_TONE: Record<string, string> = {
  pending: 'info', dispatched: 'info', running: 'info',
  gate_pending: 'warning', blocked: 'warning',
  completed: 'success', merged: 'success', failed: 'destructive', cancelled: 'muted',
}
const STEP_STATE: Record<string, string> = {
  pending: 'none', running: 'running', completed: 'ok', failed: 'err',
}
const STEP_TYPE: Record<string, string> = {
  agent: 'agent', review: 'gate', run: 'agent',
}
const TERMINAL = new Set(['completed', 'merged', 'failed', 'cancelled'])

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.clients.ops
    const slug = derived(() => String(ctx.route.params.slug?.() ?? ''))
    const runCtx = live(() => ops.pipelines.run({ slug: slug() }))

    const run = derived(() => {
      const r = unwrap(runCtx)?.run
      if (!r) return null
      return {
        slug: r.slug,
        statusLabel: r.status.replace(/_/g, ' '),
        tone: RUN_TONE[r.status] ?? 'muted',
        stateAttr: RUN_STATE[r.status] ?? 'none',
        outcome: r.outcome ?? '',
        startedAt: r.startedAt ?? '',
        startedLabel: r.startedAt ? formatRelative(r.startedAt) : '',
        endedAt: r.endedAt ?? '',
        endedLabel: r.endedAt ? formatRelative(r.endedAt) : '',
        duration: formatDuration(r.startedAt, r.endedAt),
        terminal: TERMINAL.has(r.status),
      }
    })
    const steps = derived(() => {
      const list = unwrap(runCtx)?.steps ?? []
      const last = list.length - 1
      return list.map((step, i) => ({
        stepId: step.stepId,
        stateAttr: STEP_STATE[step.status] ?? 'none',
        typeAttr: STEP_TYPE[step.stepType] ?? 'agent',
        statusLabel: step.status,
        stepType: step.stepType,
        attempt: step.attempt,
        startedLabel: step.startedAt ? formatRelative(step.startedAt) : '',
        completedLabel: step.completedAt ? formatRelative(step.completedAt) : '',
        isLast: i === last,
      }))
    })

    return { runCtx, slug, run, steps }
  },
  on: {
    cancelRun: async (s, slug: string) => {
      try { await s.clients.ops.pipelines.cancel({ slug, reason: '' }) } catch { /* best-effort */ }
    },
  },
})
