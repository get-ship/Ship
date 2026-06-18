import { controller, signal, derived, live, unwrap } from '@ship/sui-runtime'
import { ship, ops } from '../../../ops'
import type { LiveSignal, LiveSource, ViewScope } from '@ship/sui-runtime'
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
// `/_deck/data?expr=ship.pipelines.summaries()` SSE channel via `liveExpr` (the
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
      lastRunLabel: d.last_run_at ? formatRelative(d.last_run_at) : '',
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
      updatedLabel: formatRelative(r.updated_at),
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
      // PipelinesService.Put (Connect) → writes to the active project. Replaces
      // the PUT /api/v1/workspaces/{ws}/pipelines/{name} route (no daemon
      // handler — the old fetch 404'd; this fixes it).
      try {
        await ops.pipelines.put({ name, docJson: content, scope: 'project' })
        window.location.href = `/studio/pipelines/focus/${encodeURIComponent(name)}?edit=1`
      } catch (err) { s.newPipeError = (err as Error)?.message ?? String(err) }
      finally { s.newPipeCreating = false }
    },
    stopRun: async (_s, jobId: string) => {
      // FLAG (API arc, not transport): this routes to PipelinesService.EndTurn
      // (turn-complete / step-advance), the Connect form of the old
      // POST /agent_sessions/end. The old fetch sent `job_id` where the handler
      // reads `slug` — likely a silent no-op. A true "stop" probably wants
      // agent.Stop(slug), not end_job. Preserving best-effort transport here;
      // the stop-vs-complete semantics are for the API-surface arc.
      try { await ops.pipelines.endTurn({ slug: jobId }) } catch { /* best-effort */ }
    },
  },
})
