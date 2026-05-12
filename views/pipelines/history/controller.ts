import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type HistoryRow = {
  slug: string; pipeline?: string; state: string; tone: string;
  agent?: string; duration: string; updated_at: string;
}
type HistoryCtx = {
  q: string; pipeline_filter: string; rows: HistoryRow[];
  total: number; offset: number; limit: number;
  has_prev: boolean; has_next: boolean; pipelines: string[];
}

const EMPTY: HistoryCtx = {
  q: '', pipeline_filter: '', rows: [], total: 0,
  offset: 0, limit: 50, has_prev: false, has_next: false, pipelines: [],
}
const read = (ctx: unknown): HistoryCtx => (unwrap(ctx as any) as HistoryCtx | undefined) ?? EMPTY

export default controller<Scope>({
  scope: ({ ctx, route }) => {
    const q = derived(() => String((route().query as any)?.q ?? ''))
    const pipelineFilter = derived(() => String((route().query as any)?.pipeline ?? ''))
    const offset = derived(() => parseInt(String((route().query as any)?.offset ?? '0'), 10))
    const limit = derived(() => parseInt(String((route().query as any)?.limit ?? '50'), 10))
    const rows = derived((s) => read(s.ctx).rows)
    const total = derived((s) => read(s.ctx).total)
    const hasPrev = derived((s) => read(s.ctx).has_prev)
    const hasNext = derived((s) => read(s.ctx).has_next)
    const pipelines = derived((s) => read(s.ctx).pipelines)
    const qInput = signal(String((route().query as any)?.q ?? ''))
    const pipelineInput = signal(String((route().query as any)?.pipeline ?? ''))
    return { q, pipelineFilter, offset, limit, rows, total, hasPrev, hasNext, pipelines, qInput, pipelineInput }
  },
  on: {
    applyFilter: ({ navigate, qInput, pipelineInput }: any) => {
      const params = new URLSearchParams()
      if (qInput) params.set('q', String(qInput))
      if (pipelineInput) params.set('pipeline', String(pipelineInput))
      navigate('/pipelines/history' + (params.toString() ? '?' + params.toString() : ''))
    },
    clearFilter: ({ navigate }: any) => navigate('/pipelines/history'),
    prevPage: ({ navigate, offset, limit, q, pipelineFilter }: any) => {
      const params = new URLSearchParams()
      params.set('offset', String(Math.max(0, (offset as number) - (limit as number))))
      params.set('limit', String(limit))
      if (q) params.set('q', String(q))
      if (pipelineFilter) params.set('pipeline', String(pipelineFilter))
      navigate('/pipelines/history?' + params.toString())
    },
    nextPage: ({ navigate, offset, limit, q, pipelineFilter }: any) => {
      const params = new URLSearchParams()
      params.set('offset', String((offset as number) + (limit as number)))
      params.set('limit', String(limit))
      if (q) params.set('q', String(q))
      if (pipelineFilter) params.set('pipeline', String(pipelineFilter))
      navigate('/pipelines/history?' + params.toString())
    },
  },
})
