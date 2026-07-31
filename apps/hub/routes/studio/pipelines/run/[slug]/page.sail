<controller>
// Pipeline run detail (`/studio/pipelines/run/:slug`). Hub port of
// apps/shell/routes/studio/pipelines/run/[slug]/controller.ts.
//
// The shell read the deprecated live op `ship.pipelines.run` (no mux resolver →
// 404). This reads the TYPED unary `pipelines.run` → PipelineRunDetailResponse
// { run: PipelineRunRow, steps: PipelineStepRunRow[] }, wrapped in `live(() => …)`
// so the template's `` block drives loading/error/ok.
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
import { controller, derived, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

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

export default controller((ctx: ViewScope<Scope>) => {

    const ops = ctx.ops
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

    
    const cancelRun = async (s, slug: string) => {
      try { await s.clients.ops.pipelines.cancel({ slug, reason: '' }) } catch { /* best-effort */ }
    }
    return { runCtx, slug, run, steps, cancelRun }
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">

  <while-loading of="runCtx">
  <spinner size="md" centered></spinner>

  </while-loading><while-error as="err" of="runCtx">
  <error-state title="Couldn't load run" data-test="error" role="alert">
    {err.message}
    <button slot="action" tone="primary" variant="solid" on:click={() => window.location.reload()}>Retry</button>
  </error-state>

  </while-error><if cond={runCtx().kind === 'ok'}>
<if cond={run()}>
<div>
  <div>
    <div>
      <div>
        <div>
          <span>{run().slug}</span>
          <badge tone={run().tone}>{run().statusLabel}</badge>
          <if cond={run().outcome}><chip size="sm" tone="muted">{run().outcome}</chip></if>
        </div>
        <div>
          <if cond={run().startedLabel}><span>started <time datetime={run().startedAt}>{run().startedLabel}</time></span></if>
          <if cond={run().endedLabel}><span>ended <time datetime={run().endedAt}>{run().endedLabel}</time></span></if>
          <if cond={run().duration}><span>{run().duration}</span></if>
        </div>
      </div>
      <if cond={!run().terminal}>
      <div>
        <button variant="outline" size="sm" tone="destructive" on:click={() => cancelRun(run().slug)}>
          <icon slot="leading" name="stop"></icon>Cancel
        </button>
      </div>
      </if>
    </div>

    <if cond={steps().length > 0}>
    <card>
      <h3 slot="header">Pipeline</h3>
      <sui-pipeline-dag>
        <for as="step" of={steps()} key={step.stepId}>
          <sui-pipeline-step type={step.typeAttr} state={step.stateAttr}>{step.stepId}</sui-pipeline-step>
          <if cond={!step.isLast}><span aria-hidden="true">→</span></if>
        </for>
      </sui-pipeline-dag>
    </card>

    <card>
      <h3 slot="header">Steps</h3>
      <for as="step" of={steps()} key={step.stepId}>
      <div>
        <span>{step.stepId}</span>
        <div>
          <chip size="sm" tone="muted">{step.stepType}</chip>
          <span>{step.statusLabel}</span>
          <if cond={step.attempt > 1}><span>attempt {step.attempt}</span></if>
          <if cond={step.startedLabel}><span>started {step.startedLabel}</span></if>
          <if cond={step.completedLabel}><span>done {step.completedLabel}</span></if>
        </div>
      </div>
      </for>
    </card>
    </if>
  </div>
</div>
</if>
</if>
  </main>
</div>
