import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { LiveSignal, MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// `on:` handlers receive the writable state proxy; annotate explicitly
// since the OnSection union blocks contextual inference.
type S = MutableState<Scope>

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

export default controller<Scope>({
  scope: ({ route }) => {
    const summaries = live(() => liveExpr<SummariesCtx>('ship.pipelines.summaries()'))
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
    return {
      summaries,
      q, agent, pipeline, status, tab,
      dispatchOpen, dispatchSpec, dispatchPipeline, dispatchProvider, dispatchHeadless,
      newPipeOpen, newPipeName, newPipeError, newPipeCreating,
    }
  },
  derived: {
    defs: ({ summaries }) => read(summaries).defs.map((d) => ({
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
    })),
    activeRuns: ({ summaries }) => read(summaries).active_runs.map((r) => ({
      ...r,
      state_attr: r.state.replace('-', ''),
      steps: r.steps?.map((step, i, arr) => ({ ...step, isLast: i === arr.length - 1 })),
    })),
    providers: ({ summaries }) => read(summaries).providers,
    workspaceId: ({ summaries }) => read(summaries).workspace_id ?? null,
    statsSubtitle: ({ summaries }) => read(summaries).stats_subtitle,
  },
  on: {
    openDispatch: (s: S) => {
      const ds = s as unknown as { dispatchPipeline: string; defs: PipelineDef[]; dispatchOpen: boolean }
      if (!ds.dispatchPipeline && ds.defs.length > 0) ds.dispatchPipeline = ds.defs[0].name
      ds.dispatchOpen = true
    },
    closeDispatch: (s: S) => { (s as unknown as { dispatchOpen: boolean }).dispatchOpen = false },
    dispatchRun: async (s: S) => {
      const ds = s as unknown as { dispatchPipeline: string; dispatchSpec: string; dispatchProvider: string; dispatchHeadless: boolean; dispatchOpen: boolean }
      const name = String(ds.dispatchPipeline)
      if (!name) return
      await fetch(`/api/v1/pipelines/${encodeURIComponent(name)}/dispatch`, {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec: ds.dispatchSpec, provider: ds.dispatchProvider || null, headless: ds.dispatchHeadless }),
      })
      ds.dispatchOpen = false
    },
    openNewPipe: (s: S) => { const ds = s as unknown as { newPipeOpen: boolean; newPipeName: string; newPipeError: string }; ds.newPipeOpen = true; ds.newPipeName = ''; ds.newPipeError = '' },
    closeNewPipe: (s: S) => { (s as unknown as { newPipeOpen: boolean }).newPipeOpen = false },
    createPipeline: async (s: S) => {
      const ds = s as unknown as { workspaceId: string | null; newPipeName: string; newPipeError: string; newPipeCreating: boolean }
      const ws = ds.workspaceId
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
        if (resp.ok) { window.location.href = `/studio/pipelines/focus/${encodeURIComponent(name)}?edit=1`; return }
        const parsed = await resp.json().catch(() => ({}))
        ds.newPipeError = (parsed as { data?: { error?: string } })?.data?.error ?? `HTTP ${resp.status}`
      } catch (err) { ds.newPipeError = 'network: ' + ((err as Error)?.message ?? String(err)) }
      finally { ds.newPipeCreating = false }
    },
    stopRun: async (_s: S, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', {
        method: 'POST', credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ job_id: jobId }),
      })
    },
  },
})
