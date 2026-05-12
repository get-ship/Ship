import { controller, signal, derived, unwrap } from '@ship/sui-runtime'
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

export default controller<Scope>({
  scope: ({ ctx, route }) => {
    const name = derived(() => String(route().params.name ?? ''))
    const def = derived((s) => read(s.ctx).def)
    const doc = derived((s) => read(s.ctx).doc)
    const raw = derived((s) => read(s.ctx).raw)
    const format = derived((s) => read(s.ctx).format)
    const workspaceId = derived((s) => read(s.ctx).workspace_id ?? null)
    const runs = derived((s) => read(s.ctx).runs)
    const activeForPipeline = derived((s) => read(s.ctx).active_for_pipeline)
    const opsMode = derived((s) => read(s.ctx).ops_mode)
    const providers = derived((s) => read(s.ctx).providers)
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
    const editorRef = signal<Ed | null>(null)
    return {
      name, def, doc, raw, format, workspaceId, runs, activeForPipeline, opsMode, providers,
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
        if (resp.ok) { window.location.href = '/pipelines'; return }
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
      await fetch('/api/v1/jobs/end', { method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ job_id: jobId }) })
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
