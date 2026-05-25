import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import { ship } from '../../ops'
import type { LiveSignal, LiveSource, ViewScope } from '@ship/sui-runtime'
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
// `summaries` is a live signal the controller owns: subscribe directly to the
// `/_sui/data?expr=ship.pipelines.summaries()` SSE channel via `liveExpr` (the
// pattern views/events + the history view use). The shell SDK object has no
// `pipelines` surface, so the call form would throw — `liveExpr` wires the
// channel by name instead. `{#async summaries}` + derived args read the signal.
const read = (summaries: unknown): SummariesCtx => {
  const v = unwrap(summaries as LiveSignal<Partial<SummariesCtx>>)
  // Merge over EMPTY: the payload omits empty collections (active_runs,
  // providers) — fall back so `{#for}` / `.length` never hit undefined.
  return v && Array.isArray(v.defs) ? { ...EMPTY, ...v } : EMPTY
}

export default controller({
  scope: ({ route }: ViewScope<Scope>) => {
    const summaries = live(() => ship.pipelines.summaries() as LiveSource<SummariesCtx>)
    const q = derived(() => String(route.query.q() ?? ''))
    const agent = derived(() => String(route.query.agent() ?? ''))
    const pipeline = derived(() => String(route.query.pipeline() ?? ''))
    const status = derived(() => String(route.query.status() ?? ''))
    const tab = derived(() => String(route.query.tab() ?? ''))
    const dispatchOpen = signal(false)
    const dispatchSpec = signal('')
    const dispatchPipeline = signal('')
    const dispatchProvider = signal('')
    const dispatchHeadless = signal(true)
    const newPipeOpen = signal(false)
    const newPipeName = signal('')
    const newPipeError = signal('')
    const newPipeCreating = signal(false)
    const defs = derived(() => read(summaries).defs.map((d) => ({
      ...d,
      success_pct: Math.floor(d.success_rate * 100),
      // Map run tone → the sui-pipeline-card state taxonomy (ok/running/awaiting/err/none).
      card_state: d.active_runs > 0
        ? 'running'
        : d.last_run_tone === 'success'
          ? 'ok'
          : d.last_run_tone === 'destructive'
            ? 'err'
            : d.last_run_tone === 'warning'
              ? 'awaiting'
              : 'none',
    })))
    const activeRuns = derived(() => read(summaries).active_runs.map((r) => ({
      ...r,
      state_attr: r.state.replace('-', ''),
      steps: r.steps?.map((step, i, arr) => ({ ...step, isLast: i === arr.length - 1 })),
    })))
    const providers = derived(() => read(summaries).providers)
    const workspaceId = derived(() => read(summaries).workspace_id ?? null)
    const statsSubtitle = derived(() => read(summaries).stats_subtitle)
    return {
      summaries,
      q, agent, pipeline, status, tab,
      dispatchOpen, dispatchSpec, dispatchPipeline, dispatchProvider, dispatchHeadless,
      newPipeOpen, newPipeName, newPipeError, newPipeCreating,
      defs, activeRuns, providers, workspaceId, statsSubtitle,
    }
  },
  on: {
    openDispatch: (s) => {
      if (!s.dispatchPipeline && s.defs.length > 0) s.dispatchPipeline = s.defs[0].name
      s.dispatchOpen = true
    },
    closeDispatch: (s) => { s.dispatchOpen = false },
    dispatchRun: async (s) => {
      const name = String(s.dispatchPipeline)
      if (!name) return
      await ship.pipelines.dispatch({
        pipeline: name,
        spec: String(s.dispatchSpec),
        harness: String(s.dispatchProvider || ''),
        auto: Boolean(s.dispatchHeadless),
      })
      s.dispatchOpen = false
    },
    openNewPipe: (s) => { s.newPipeOpen = true; s.newPipeName = ''; s.newPipeError = '' },
    closeNewPipe: (s) => { s.newPipeOpen = false },
    createPipeline: async (s) => {
      const ws = s.workspaceId
      if (!ws) return
      const name = String(s.newPipeName).trim()
      if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) {
        s.newPipeError = 'Name must be lowercase kebab-case (a–z, 0–9, -).'; return
      }
      s.newPipeCreating = true; s.newPipeError = ''
      const content = `{\n  "name": "${name}",\n  "description": "",\n  "steps": [\n    { "id": "one", "run": "echo hello" }\n  ]\n}\n`
      try {
        const resp = await fetch(
          `/api/v1/workspaces/${encodeURIComponent(ws)}/pipelines/${encodeURIComponent(name)}`,
          { method: 'PUT', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) },
        )
        if (resp.ok) { window.location.href = `/studio/pipelines/focus/${encodeURIComponent(name)}?edit=1`; return }
        const parsed = await resp.json().catch(() => ({}))
        s.newPipeError = (parsed as { data?: { error?: string } })?.data?.error ?? `HTTP ${resp.status}`
      } catch (err) { s.newPipeError = 'network: ' + ((err as Error)?.message ?? String(err)) }
      finally { s.newPipeCreating = false }
    },
    stopRun: async (_s, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
