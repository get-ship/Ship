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
import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/deck/router'
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

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ops = ctx.clients.ops
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

    return {
      historyData, q, pipelineFilter, offset, limit,
      rows, total, hasPrev, hasNext, pipelines, qInput, pipelineInput,
    }
  },
  on: {
    applyFilter: (s) => {
      const params = new URLSearchParams()
      if (s.qInput) params.set('q', String(s.qInput))
      if (s.pipelineInput) params.set('pipeline', String(s.pipelineInput))
      s.navigate('/studio/pipelines/history' + (params.toString() ? '?' + params.toString() : ''))
    },
    clearFilter: (s) => s.navigate('/studio/pipelines/history'),
    prevPage: (s) => {
      const params = new URLSearchParams()
      params.set('offset', String(Math.max(0, s.offset - s.limit)))
      params.set('limit', String(s.limit))
      if (s.q) params.set('q', String(s.q))
      if (s.pipelineFilter) params.set('pipeline', String(s.pipelineFilter))
      s.navigate('/studio/pipelines/history?' + params.toString())
    },
    nextPage: (s) => {
      const params = new URLSearchParams()
      params.set('offset', String(s.offset + s.limit))
      params.set('limit', String(s.limit))
      if (s.q) params.set('q', String(s.q))
      if (s.pipelineFilter) params.set('pipeline', String(s.pipelineFilter))
      s.navigate('/studio/pipelines/history?' + params.toString())
    },
  },
})
