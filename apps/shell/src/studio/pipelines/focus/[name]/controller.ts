import { controller, signal, derived, live, ref } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

// `on:` handlers receive the writable state proxy; annotate explicitly
// since the OnSection union blocks contextual inference.
type S = MutableState<Scope>

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
// `envelope` is the live signal's current value (`focusCtx()`): pull the `ok`
// value, falling back to EMPTY_CTX while loading / on error. Guards `.value`
// for null on the focus op (returns 404→error, never a null ok value).
const read = (envelope: unknown): FocusCtx => {
  const env = envelope as { kind?: string; value?: Partial<FocusCtx> } | undefined
  if (env?.kind !== 'ok' || !env.value) return EMPTY_CTX
  // Merge over EMPTY_CTX: the payload omits optional collections (e.g.
  // `providers`, `active_for_pipeline`) when empty — fall back so `{#for}`
  // and `.map` never hit undefined.
  return { ...EMPTY_CTX, ...env.value }
}

type Ed = HTMLElement & { value: string; getAttribute(n: string): string | null; setDiagnostics?(d: unknown[]): void }

// `sui-code-editor` ships as a lazy island bundle (`/_sui/sui-components/<tag>.js`).
// The daemon eagerly injects islands it finds by static-scanning templates, but the
// editor only renders inside the `editMode` branch — fetch it on demand so the
// element upgrades when the user opens the editor.
// The active beforeUnload listener for this view instance. Module-scoped
// because the writable state proxy only accepts declared template-state keys.
let beforeUnloadListener: ((e: BeforeUnloadEvent) => void) | null = null
let editorIslandLoaded = false
function ensureEditorIsland(): void {
  if (editorIslandLoaded || customElements.get('sui-code-editor')) { editorIslandLoaded = true; return }
  editorIslandLoaded = true
  const tag = 'sui-code-editor'
  import(/* @vite-ignore */ `/_sui/sui-components/${tag}.js`).catch(() => { editorIslandLoaded = false })
}

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
  scope: ({ route }) => {
    // `route` is a RouteSignals object; `params.name` is a Signal — read it via
    // `route.params.name?.()` (matches every other studio dynamic-route view).
    // Requires the `/studio/pipelines/focus/:name` pattern registered via
    // defineRoutes at shell startup; absent that, name() is undefined → ''.
    const name = derived(() => String(route.params.name?.() ?? ''))
    // Subscribe to the focus op by name via `liveExpr` — the shell SDK object
    // exposes no `pipelines` surface, so the call form would throw. Re-subscribes
    // when `name()` changes. `liveExpr` wraps the SSE channel as a LiveSource.
    const focusCtx = live(() => liveExpr<FocusCtx>(`ship.pipelines.focus(${JSON.stringify({ name: name() })})`))
    const def = derived(() => decorateDef(read(focusCtx()).def))
    const doc = derived(() => {
      const d = read(focusCtx()).doc
      return { ...d, steps: d.steps.map(decorateStep) }
    })
    const raw = derived(() => read(focusCtx()).raw)
    const format = derived(() => read(focusCtx()).format)
    const workspaceId = derived(() => read(focusCtx()).workspace_id ?? null)
    const runs = derived(() => read(focusCtx()).runs.map(decorateRun))
    const activeForPipeline = derived(() => read(focusCtx()).active_for_pipeline.map(decorateRun))
    const opsMode = derived(() => read(focusCtx()).ops_mode)
    const providers = derived(() => read(focusCtx()).providers)
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
      focusCtx, name, def, doc, raw, format, workspaceId, runs, activeForPipeline, opsMode, providers,
      editMode, editSaving, editError, editErrorLine, editErrorCol, editIssueCount,
      dispatchOpen, dispatchSpec, dispatchProvider, dispatchHeadless, deleteOpen, editorRef,
    }
  },
  on: {
    startEdit: (s: S) => { ensureEditorIsland(); const ds = s as any; ds.editMode = true; ds.editError = ''; ds.editIssueCount = 0 },
    cancelEdit: (s: S) => {
      const ds = s as any
      ds.editMode = false; ds.editError = ''; ds.editIssueCount = 0
      const ed = ds.editorRef as Ed | null
      if (ed) { ed.value = ed.getAttribute('value') ?? ''; ed.setDiagnostics?.([]) }
    },
    savePipeline: async (s: S) => {
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
    deletePipeline: async (s: S) => {
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
    openDispatch: (s: S) => { (s as any).dispatchOpen = true },
    closeDispatch: (s: S) => { (s as any).dispatchOpen = false },
    dispatchRun: async (s: S) => {
      const ds = s as any; const nm = ds.name as string
      await fetch(`/api/v1/pipelines/${encodeURIComponent(nm)}/dispatch`, {
        method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ spec: ds.dispatchSpec, provider: ds.dispatchProvider || null, headless: ds.dispatchHeadless }),
      })
      ds.dispatchOpen = false
    },
    openDelete: (s: S) => { (s as any).deleteOpen = true },
    closeDelete: (s: S) => { (s as any).deleteOpen = false },
    stopRun: async (_s: S, jobId: string) => {
      await fetch('/api/v1/agent_sessions/end', { method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ job_id: jobId }) })
    },
    pulseRun: (_s: S, el: HTMLElement) => {
      el.classList.add('ship-heartbeat')
      const existing = (el as any).__pulseTimer
      if (existing) clearTimeout(existing)
      ;(el as any).__pulseTimer = setTimeout(() => el.classList.remove('ship-heartbeat'), 1500)
    },
  },
  mount: (s) => {
    if (/[?&]edit=1(&|$)/.test(window.location.search)) { ensureEditorIsland(); (s as any).editMode = true }
    const beforeUnload = (e: BeforeUnloadEvent) => {
      if (!(s as any).editMode) return
      const ed = (s as any).editorRef as Ed | null
      if (!ed) return
      if (ed.value !== (ed.getAttribute('value') ?? '')) { e.preventDefault(); e.returnValue = '' }
    }
    window.addEventListener('beforeunload', beforeUnload)
    // Hold the listener at module scope: the runtime ignores writes to keys
    // that aren't declared template state, so stashing it on `s` would drop it
    // and leak the listener past unmount.
    beforeUnloadListener = beforeUnload
  },
  unmount: () => {
    if (beforeUnloadListener) {
      window.removeEventListener('beforeunload', beforeUnloadListener)
      beforeUnloadListener = null
    }
  },
})
