<controller>
// Pipelines history (`/studio/pipelines/history`). Hub port of
// apps/shell/routes/studio/pipelines/history/controller.ts.
//
// The shell read the deprecated live op `ship.pipelines.history` (no mux
// resolver → 404). This reads the TYPED unary `pipelines.history` →
// PipelineRunHistoryResponse { runs: PipelineRunRow[], total, offset, limit }
// (the count fields are uint64 → bigint over the wire). The request takes the
// same `q` / `pipeline` / `offset` / `limit` filters (offset/limit as bigint).
//
// The typed projection rows carry slug/status/startedAt/endedAt/outcome — NOT
// the GUI tokens the shell stream added (tone/agent/duration text/pipeline echo).
// Tone is derived from `status`; duration is computed from startedAt→endedAt.
// The pipeline-filter dropdown is populated from `pipelines.list` (the
// definitions index), since the history response carries no pipeline names.
import { controller, signal, derived, live, unwrap } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import { route as routeFactory } from '@ship/deck-router'
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

function formatDuration(startIso: string, endIso?: string): string {
  if (!endIso) return '—'
  const start = Date.parse(startIso)
  const end = Date.parse(endIso)
  if (Number.isNaN(start) || Number.isNaN(end) || end < start) return '—'
  const secs = Math.round((end - start) / 1000)
  if (secs < 60) return `${secs}s`
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`
  return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`
}

const RUN_TONE: Record<string, string> = {
  pending: 'info', dispatched: 'info', running: 'info',
  gate_pending: 'warning', blocked: 'warning',
  completed: 'success', merged: 'success', failed: 'destructive', cancelled: 'muted',
}

export default controller((ctx: ViewScope<Scope>) => {

    const ops = ctx.ops
    const route = routeFactory()
    const q = derived(() => String(route.query.q() ?? ''))
    const pipelineFilter = derived(() => String(route.query.pipeline() ?? ''))
    const offset = derived(() => parseInt(String(route.query.offset() ?? '0'), 10) || 0)
    const limit = derived(() => parseInt(String(route.query.limit() ?? '50'), 10) || 50)

    const historyData = live(() => ops.pipelines.history({
      q: q(), pipeline: pipelineFilter(),
      offset: BigInt(offset()), limit: BigInt(limit()),
    }))
    const defsData = live(() => ops.pipelines.list({}))

    const rows = derived(() => (unwrap(historyData)?.runs ?? []).map((r) => ({
      slug: r.slug,
      statusLabel: r.status.replace(/_/g, ' '),
      tone: RUN_TONE[r.status] ?? 'muted',
      outcome: r.outcome ?? '',
      startedAt: r.startedAt,
      startedLabel: r.startedAt ? formatRelative(r.startedAt) : '',
      duration: formatDuration(r.startedAt, r.endedAt),
    })))
    const total = derived(() => Number(unwrap(historyData)?.total ?? 0n))
    const hasPrev = derived(() => offset() > 0)
    const hasNext = derived(() => offset() + rows().length < total())
    const pipelines = derived(() => (unwrap(defsData)?.items ?? []).map((i) => i.id))

    const qInput = signal(String(route.query.q() ?? ''))
    const pipelineInput = signal(String(route.query.pipeline() ?? ''))

    
    const applyFilter = (s) => {
      const params = new URLSearchParams()
      if (s.qInput) params.set('q', String(s.qInput))
      if (s.pipelineInput) params.set('pipeline', String(s.pipelineInput))
      s.navigate('/studio/pipelines/history' + (params.toString() ? '?' + params.toString() : ''))
    }
    const clearFilter = (s) => s.navigate('/studio/pipelines/history')
    const prevPage = (s) => {
      const params = new URLSearchParams()
      params.set('offset', String(Math.max(0, s.offset - s.limit)))
      params.set('limit', String(s.limit))
      if (s.q) params.set('q', String(s.q))
      if (s.pipelineFilter) params.set('pipeline', String(s.pipelineFilter))
      s.navigate('/studio/pipelines/history?' + params.toString())
    }
    const nextPage = (s) => {
      const params = new URLSearchParams()
      params.set('offset', String(s.offset + s.limit))
      params.set('limit', String(s.limit))
      if (s.q) params.set('q', String(s.q))
      if (s.pipelineFilter) params.set('pipeline', String(s.pipelineFilter))
      s.navigate('/studio/pipelines/history?' + params.toString())
    }
    return {
      historyData, q, pipelineFilter, offset, limit,
      rows, total, hasPrev, hasNext, pipelines, qInput, pipelineInput, applyFilter, clearFilter, prevPage, nextPage }
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
<div>
  <div>
    <header>
      <div>
        <h1>History</h1>
        <p>
          {total()} run{total() !== 1 ? 's' : ''}
          {q() ? ' matching "' + q() + '"' : ''}
          {pipelineFilter() ? ' in ' + pipelineFilter() : ''}
        </p>
      </div>
    </header>

    <div>
      <field label="Search" hint="Run slug.">
        <input bind:value={qInput} placeholder="e.g. shell-smoke, rust-lane…" on:keydown={(e) => e.key === 'Enter' && applyFilter()}>
      </field>
      <field label="Pipeline" data-narrow>
        <select bind:value={pipelineInput}>
          <option value="">(all)</option>
          <for as="p" of={pipelines()} key={p}><option value={p}>{p}</option></for>
        </select>
      </field>
      <button variant="outline" size="sm" on:click={applyFilter}>
        <icon slot="leading" name="search" size="sm"></icon>Apply
      </button>
      <if cond={q() || pipelineFilter()}>
      <button variant="ghost" size="sm" on:click={clearFilter}>Clear</button>
      </if>
    </div>

    
      <while-loading of="historyData">
      <spinner size="md" centered></spinner>

      </while-loading><while-error as="err" of="historyData">
      <error-state title="Couldn't load history" data-test="error" role="alert">
        {err.message}
        <button slot="action" tone="primary" variant="solid" on:click={() => window.location.reload()}>Retry</button>
      </error-state>

      </while-error><if cond={historyData().kind === 'ok'}>
    <div>
      <if cond={rows().length > 0}>
      <table>
        <thead>
          <tr>
            <th>Slug</th><th>Status</th><th>Outcome</th>
            <th>Started</th><th>Duration</th>
          </tr>
        </thead>
        <tbody>
          <for as="row" of={rows()} key={row.slug}>
          <tr on:click={() => navigate('/studio/pipelines/run/' + row.slug)}>
            <td><span>{row.slug}</span></td>
            <td><badge tone={row.tone} size="sm">{row.statusLabel}</badge></td>
            <td><if cond={row.outcome}><code>{row.outcome}</code><else><span>—</span></else></if></td>
            <td><time datetime={row.startedAt}>{row.startedLabel}</time></td>
            <td><span>{row.duration}</span></td>
          </tr>
          </for>
        </tbody>
      </table>
      <else>
      <div>
        {q() || pipelineFilter() ? 'No runs match the current filter.' : 'No run history yet. Dispatch a pipeline to see runs here.'}
      </div>
      </else></if>
    </div>

    <if cond={total() > limit()}>
    <nav aria-label="History pagination">
      <span>Showing {offset() + 1}–{offset() + rows().length} of {total()}</span>
      <div>
        <if cond={hasPrev()}>
        <button variant="outline" size="sm" on:click={prevPage}>
          <icon slot="leading" name="chevron-left" size="sm"></icon>Previous
        </button>
        </if>
        <if cond={hasNext()}>
        <button variant="outline" size="sm" on:click={nextPage}>
          Next<icon slot="trailing" name="chevron-right" size="sm"></icon>
        </button>
        </if>
      </div>
    </nav>
    </if>
    </if>
  </div>
</div>
  </main>
</div>
