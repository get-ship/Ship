import { controller, signal, derived, live, chrome } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { route as routeFactory } from '@ship/sui-runtime/route'
import { ship } from '../../../../ops'
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

type HistoryRow = {
  slug: string; pipeline?: string; state: string; tone: string;
  agent?: string; duration: string; updated_at: string;
}
type HistoryRowView = HistoryRow & { updatedLabel: string }
type HistoryCtx = {
  q: string; pipeline_filter: string; rows: HistoryRow[];
  total: number; offset: number; limit: number;
  has_prev: boolean; has_next: boolean; pipelines: string[];
}

const EMPTY: HistoryCtx = {
  q: '', pipeline_filter: '', rows: [], total: 0,
  offset: 0, limit: 50, has_prev: false, has_next: false, pipelines: [],
}

// `envelope` is the read LiveSignal value (`ctx()`): pull the `ok` value,
// falling back to EMPTY while loading / on error.
const read = (envelope: unknown): HistoryCtx => {
  const env = envelope as { kind?: string; value?: HistoryCtx } | undefined
  return env?.kind === 'ok' && env.value ? env.value : EMPTY
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const route = routeFactory()
    const q = derived(() => String(route.query.q() ?? ''))
    const pipelineFilter = derived(() => String(route.query.pipeline() ?? ''))
    const offset = derived(() => parseInt(String(route.query.offset() ?? '0'), 10))
    const limit = derived(() => parseInt(String(route.query.limit() ?? '50'), 10))
    const historyData = live(() => ship.pipelines.history({
      q: q(), pipeline: pipelineFilter(), offset: offset(), limit: limit(),
    }) as LiveSource<HistoryCtx>)
    const rows = derived<HistoryRowView[]>(() =>
      read(historyData()).rows.map((row) => ({ ...row, updatedLabel: formatRelative(row.updated_at) })),
    )
    const total = derived(() => read(historyData()).total)
    const hasPrev = derived(() => read(historyData()).has_prev)
    const hasNext = derived(() => read(historyData()).has_next)
    const pipelines = derived(() => read(historyData()).pipelines)
    const qInput = signal(String(route.query.q() ?? ''))
    const pipelineInput = signal(String(route.query.pipeline() ?? ''))
    // The view-host doesn't inject `navigate` into ctx; route through the
    // runtime chrome bridge (matches views/home).
    const navigate = (path: string): void => chrome.navigate(path)
    return { historyData, q, pipelineFilter, offset, limit, rows, total, hasPrev, hasNext, pipelines, qInput, pipelineInput, navigate }
  },
  on: {
    applyFilter: ({ navigate, qInput, pipelineInput }: any) => {
      const params = new URLSearchParams()
      if (qInput) params.set('q', String(qInput))
      if (pipelineInput) params.set('pipeline', String(pipelineInput))
      navigate('/studio/pipelines/history' + (params.toString() ? '?' + params.toString() : ''))
    },
    clearFilter: ({ navigate }: any) => navigate('/studio/pipelines/history'),
    prevPage: ({ navigate, offset, limit, q, pipelineFilter }: any) => {
      const params = new URLSearchParams()
      params.set('offset', String(Math.max(0, (offset as number) - (limit as number))))
      params.set('limit', String(limit))
      if (q) params.set('q', String(q))
      if (pipelineFilter) params.set('pipeline', String(pipelineFilter))
      navigate('/studio/pipelines/history?' + params.toString())
    },
    nextPage: ({ navigate, offset, limit, q, pipelineFilter }: any) => {
      const params = new URLSearchParams()
      params.set('offset', String((offset as number) + (limit as number)))
      params.set('limit', String(limit))
      if (q) params.set('q', String(q))
      if (pipelineFilter) params.set('pipeline', String(pipelineFilter))
      navigate('/studio/pipelines/history?' + params.toString())
    },
  },
})
