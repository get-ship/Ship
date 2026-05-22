import { controller, signal, derived, live, ref, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type StepDef = {
  id: string; run?: string; agent?: string; provider?: string; model?: string;
  after?: string[]; gate?: { require_changed?: { match?: string; min?: number }; forbid_changed?: { match?: string; min?: number } };
}
type PipelineDef = { total_runs: number; active_runs: number; success_rate: number; last_run_at?: string; last_run_status?: string; last_run_tone?: string; description?: string; step_count: number; }
type PipelineDoc = { steps: StepDef[]; auto_merge?: boolean; }
type RunView = { slug: string; agent?: string; job_id: string; state: string; updated_at: string; error?: string; steps?: Array<{ id: string; state: string; step_type: string; agent?: string }>; }
type FocusCtx = {
  def: PipelineDef; doc: PipelineDoc; raw: string; format: string;
  workspace_id?: string; runs: RunView[]; active_for_pipeline: RunView[];
  other_for_pipeline: RunView[]; ops_mode: boolean; providers: string[];
}

const EMPTY_DEF: PipelineDef = { total_runs: 0, active_runs: 0, success_rate: 0, step_count: 0 }
const EMPTY_CTX: FocusCtx = { def: EMPTY_DEF, doc: { steps: [] }, raw: '', format: 'jsonc', runs: [], active_for_pipeline: [], other_for_pipeline: [], ops_mode: false, providers: [] }
const read = (ctx: unknown): FocusCtx => (unwrap(ctx as any) as FocusCtx | undefined) ?? EMPTY_CTX

type Ed = HTMLElement & { value: string; getAttribute(n: string): string | null; setDiagnostics?(d: unknown[]): void }

const decorateRun = (r: RunView) => ({
  ...r,
  state_attr: r.state.replace('-', ''),
  steps: r.steps?.map((step, i, arr) => ({ ...step, isLast: i === arr.length - 1 })),
})

const decorateDef = (d: PipelineDef) => ({
  ...d,
  success_pct: Math.floor((d.success_rate ?? 0) * 100),
  success_accent: d.total_runs > 0
    ? (d.success_rate >= 0.8 ? 'success' : d.success_rate >= 0.5 ? 'warning' : 'destructive')
    : 'default',
  active_accent: d.active_runs > 0 ? 'info' : 'default',
  last_run_tone_safe: d.last_run_tone ?? 'muted',
})

const decorateStep = (step: StepDef, i: number) => ({
  ...step,
  index: i + 1,
  step_type: step.run ? 'run' : step.gate ? 'gate' : 'agent',
  after_joined: step.after ? step.after.join(',') : undefined,
})

export default controller<Scope>({
  scope: ({ ship, route }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const s = ship as any
    const name = derived(() => String(route().params.name ?? ''))
    const ctx = live(() => s.pipelines.focus({ name: name() }))
    const def = derived(() => decorateDef(read(ctx()).def))
    const doc = derived(() => {
      const d = read(ctx()).doc
      return { ...d, steps: d.steps.map(decorateStep) }
    })
    const raw = derived(() => read(ctx()).raw)
    const format = derived(() => read(ctx()).format)
    const workspaceId = derived(() => read(ctx()).workspace_id ?? null)
    const runs = derived(() => read(ctx()).runs.map(decorateRun))
    const activeForPipeline = derived(() => read(ctx()).active_for_pipeline.map(decorateRun))
    const opsMode = derived(() => read(ctx()).ops_mode)
    const providers = derived(() => read(ctx()).providers)
    const editMode = signal(false)
    const editSaving = signal(false)
    const editError = signal('')
    const editErrorLine = signal(0)
    const editErrorCol = signal(0)
    const editIssueCount = signal(0)
    const dispatchOpen = signal(false)
    const dispatchSpec = signal('')
    const dispatchProvider = signal('')
    const dispatchHeadless = signal(true)
    const deleteOpen = signal(false)
    const editorRef = ref<Ed>(null)
    return {
      ctx, name, def, doc, raw, format, workspaceId, runs, activeForPipeline, opsMode, providers,
      editMode, editSaving, editError, editErrorLine, editErrorCol, editIssueCount,
      dispatchOpen, dispatchSpec, dispatchProvider, dispatchHeadless, deleteOpen, editorRef,
    }
  },
  on: {
    startEdit: (s) => { const ds = s as any; ds.editMode = true; ds.editError = ''; ds.editIssueCount = 0 },
    cancelEdit: (s) => {
      const ds = s as any
      ds.editMode = false; ds.editError = ''; ds.editIssueCount = 0
      const ed = ds.editorRef as Ed | null
      if (ed) { ed.value = ed.getAttribute('value') ?? ''; ed.setDiagnostics?.([]) }
    },
    savePipeline: async (s) => {
      const ds = s as any
      const ed = ds.editorRef as Ed | null
      if (!ed) return
      const ws = ds.workspaceId as string | null
      const nm = ds.name as string
      if (!ws) return
      ds.editSaving = true; ds.editError = ''; ds.editErrorLine = 0; ds.editErrorCol = 0; ds.editIssueCount = 0
      ed.setDiagnostics?.([])
      try {
        const resp = await fetch(
          `/api/v1/workspaces/${encodeURIComponent(ws)}/pipelines/${encodeURIComponent(nm)}`,
          { method: 'PUT', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content: ed.value }) },
        )
        if (resp.ok) { window.location.reload(); return }
        const body = await resp.json().catch(() => ({}))
        const data = body?.data ?? {}
        const issues: any[] = Array.isArray(data.issues) ? data.issues : (data.parse ? [data.parse] : [])
        const primary = issues[0]
        if (primary) {
          ds.editError = primary.message ?? 'validation error'
          ds.editErrorLine = primary.line ?? 0; ds.editErrorCol = primary.column ?? 0
          ds.editIssueCount = issues.length
          ed.setDiagnostics?.(issues.map(i => ({ line: i.line ?? 1, column: i.column ?? 1, severity: i.kind === 'parse' ? 'error' : 'warning', message: i.message ?? 'validation error', source: i.path ?? i.kind })))
        } else { ds.editError = data.error ?? `HTTP ${resp.status}` }
      } catch (err: any) { ds.editError = 'network: ' + (err?.message ?? String(err)) }
      finally { ds.editSaving = false }
    },
    deletePipeline: async (s) => {
      const ds = s as any
      const ws = ds.workspaceId as string | null; const nm = ds.name as string
      if (!ws) return
      try {
        const resp = await fetch(
          `/api/v1/workspaces/${encodeURIComponent(ws)}/pipelines/${encodeURIComponent(nm)}`,
          { method: 'DELETE', credentials: 'include' },
        )
        if (resp.ok) { window.location.href = '/studio/pipelines'; return }
        const body = await resp.json().catch(() => ({}))
        window.alert('Delete failed: ' + (body?.data ?? `HTTP ${resp.status}`))
      } catch (err: any) { window.alert('Delete failed: ' + (err?.message ?? String(err))) }
      ds.deleteOpen = false
    },
    openDispatch: (s) => { (s as any).dispatchOpen = true },
    closeDispatch: (s) => { (s as any).dispatchOpen = false },
    dispatchRun: async (s) => {
      const ds = s as any; const nm = ds.name as string
      await fetch(`/api/v1/pipelines/${encodeURIComponent(nm)}/dispatch`, {
        method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec: ds.dispatchSpec, provider: ds.dispatchProvider || null, headless: ds.dispatchHeadless }),
      })
      ds.dispatchOpen = false
    },
    openDelete: (s) => { (s as any).deleteOpen = true },
    closeDelete: (s) => { (s as any).deleteOpen = false },
    stopRun: async (_s, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', { method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ job_id: jobId }) })
    },
    pulseRun: (_s, el: HTMLElement) => {
      el.classList.add('ship-heartbeat')
      const existing = (el as any).__pulseTimer
      if (existing) clearTimeout(existing)
      ;(el as any).__pulseTimer = setTimeout(() => el.classList.remove('ship-heartbeat'), 1500)
    },
  },
  mount: (s) => {
    if (/[?&]edit=1(&|$)/.test(window.location.search)) (s as any).editMode = true
    const beforeUnload = (e: BeforeUnloadEvent) => {
      if (!(s as any).editMode) return
      const ed = (s as any).editorRef as Ed | null
      if (!ed) return
      if (ed.value !== (ed.getAttribute('value') ?? '')) { e.preventDefault(); e.returnValue = '' }
    }
    window.addEventListener('beforeunload', beforeUnload)
    ;(s as any).__beforeUnload = beforeUnload
  },
  unmount: (s) => {
    const fn = (s as any).__beforeUnload
    if (fn) window.removeEventListener('beforeunload', fn)
  },
})
