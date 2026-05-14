import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
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
const read = (envelope: unknown): SummariesCtx => {
  const env = envelope as { kind?: string; value?: SummariesCtx } | undefined
  return env?.kind === 'ok' && env.value ? env.value : EMPTY
}

export default controller<Scope>({
  scope: ({ ship, route }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const q = derived(() => String(route.query.q() ?? ''))
    const agent = derived(() => String(route.query.agent() ?? ''))
    const pipeline = derived(() => String(route.query.pipeline() ?? ''))
    const status = derived(() => String(route.query.status() ?? ''))
    const tab = derived(() => String(route.query.tab() ?? ''))
    const ctx = live(() => s.pipelines.summaries({
      q: q(), agent: agent(), pipeline: pipeline(), status: status(), tab: tab(),
    }))
    const defs = derived(() => read(ctx()).defs.map((d) => ({
      ...d, success_pct: Math.floor(d.success_rate * 100),
    })))
    const activeRuns = derived(() => read(ctx()).active_runs.map((r) => ({
      ...r,
      state_attr: r.state.replace('-', ''),
      steps: r.steps?.map((step, i, arr) => ({ ...step, isLast: i === arr.length - 1 })),
    })))
    const providers = derived(() => read(ctx()).providers)
    const workspaceId = derived(() => read(ctx()).workspace_id ?? null)
    const statsSubtitle = derived(() => read(ctx()).stats_subtitle)
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
      ctx, q, agent, pipeline, status, tab,
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
