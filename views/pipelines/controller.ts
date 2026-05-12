import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type PipelineDef = {
  name: string; description?: string; step_count: number; total_runs: number;
  active_runs: number; success_rate: number; last_run_at?: string;
  last_run_status?: string; last_run_tone?: string;
}
type RunView = {
  slug: string; agent?: string; job_id: string; state: string; updated_at: string;
  branch?: string; provider?: string; model?: string; blocker?: string;
  steps?: Array<{ id: string; state: string; step_type: string; agent?: string }>;
}
type SummariesCtx = {
  stats_subtitle: string; defs: PipelineDef[]; active_runs: RunView[];
  providers: string[]; workspace_id?: string;
}

const EMPTY: SummariesCtx = { stats_subtitle: '', defs: [], active_runs: [], providers: [] }
const read = (ctx: unknown): SummariesCtx => (unwrap(ctx as any) as SummariesCtx | undefined) ?? EMPTY

export default controller<Scope>({
  scope: ({ ctx, route }) => {
    const q = derived(() => String((route().query as any)?.q ?? ''))
    const agent = derived(() => String((route().query as any)?.agent ?? ''))
    const pipeline = derived(() => String((route().query as any)?.pipeline ?? ''))
    const status = derived(() => String((route().query as any)?.status ?? ''))
    const tab = derived(() => String((route().query as any)?.tab ?? ''))
    const defs = derived((s) => read(s.ctx).defs)
    const activeRuns = derived((s) => read(s.ctx).active_runs)
    const providers = derived((s) => read(s.ctx).providers)
    const workspaceId = derived((s) => read(s.ctx).workspace_id ?? null)
    const statsSubtitle = derived((s) => read(s.ctx).stats_subtitle)
    const dispatchOpen = signal(false)
    const dispatchSpec = signal('')
    const dispatchPipeline = signal('')
    const dispatchProvider = signal('')
    const dispatchHeadless = signal(true)
    const newPipeOpen = signal(false)
    const newPipeName = signal('')
    const newPipeError = signal('')
    const newPipeCreating = signal(false)
    return {
      q, agent, pipeline, status, tab,
      defs, activeRuns, providers, workspaceId, statsSubtitle,
      dispatchOpen, dispatchSpec, dispatchPipeline, dispatchProvider, dispatchHeadless,
      newPipeOpen, newPipeName, newPipeError, newPipeCreating,
    }
  },
  on: {
    openDispatch: (s) => {
      const ds = s as any
      if (!ds.dispatchPipeline && (ds.defs as PipelineDef[]).length > 0) {
        ds.dispatchPipeline = (ds.defs as PipelineDef[])[0].name
      }
      ds.dispatchOpen = true
    },
    closeDispatch: (s) => { (s as any).dispatchOpen = false },
    dispatchRun: async (s) => {
      const ds = s as any
      const name = String(ds.dispatchPipeline)
      if (!name) return
      await fetch(`/api/v1/pipelines/${encodeURIComponent(name)}/dispatch`, {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec: ds.dispatchSpec, provider: ds.dispatchProvider || null, headless: ds.dispatchHeadless }),
      })
      ds.dispatchOpen = false
    },
    openNewPipe: (s) => { const ds = s as any; ds.newPipeOpen = true; ds.newPipeName = ''; ds.newPipeError = '' },
    closeNewPipe: (s) => { (s as any).newPipeOpen = false },
    createPipeline: async (s) => {
      const ds = s as any
      const ws = ds.workspaceId as string | null
      if (!ws) return
      const name = String(ds.newPipeName).trim()
      if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) {
        ds.newPipeError = 'Name must be lowercase kebab-case (a–z, 0–9, -).'; return
      }
      ds.newPipeCreating = true; ds.newPipeError = ''
      const content = `{\n  "name": "${name}",\n  "description": "",\n  "steps": [\n    { "id": "one", "run": "echo hello" }\n  ]\n}\n`
      try {
        const resp = await fetch(
          `/api/v1/workspaces/${encodeURIComponent(ws)}/pipelines/${encodeURIComponent(name)}`,
          { method: 'PUT', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) },
        )
        if (resp.ok) { window.location.href = `/pipelines/focus/${encodeURIComponent(name)}?edit=1`; return }
        const parsed = await resp.json().catch(() => ({}))
        ds.newPipeError = parsed?.data?.error ?? `HTTP ${resp.status}`
      } catch (err: any) { ds.newPipeError = 'network: ' + (err?.message ?? String(err)) }
      finally { ds.newPipeCreating = false }
    },
    stopRun: async (_s, jobId: string) => {
      await fetch('/api/v1/jobs/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
