import { chrome, controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-source.sui'

interface ViewSource { view_name: string; template_path: string; template: string; manifest_filename: string | null; manifest: string | null; scope: string }
interface Diagnostic { line: number; col: number; rule: string; message: string; severity?: string; file?: string }
interface CompletionItem { label: string; detail?: string; info?: string; insert?: string }
interface CompletionSource { trigger: string; items: CompletionItem[] }
interface OpDescriptor { name: string; description?: string; returns?: string; permission?: string; args?: Array<{ name: string; kind: string; required?: boolean }> }
interface MacroDescriptor { name: string; signature?: string; summary?: string; description?: string }

type EditorDiag = { line: number; column?: number; severity: 'error' | 'warning' | 'info'; message: string; source?: string }
type CodeEditor = HTMLElement & {
  value?: string
  setDiagnostics?(d: EditorDiag[]): void
  setCompletions?(s: CompletionSource[]): void
}

function manifestLang(name: string | null): string {
  if (!name) return ''
  if (name.endsWith('.yaml') || name.endsWith('.yml')) return 'yaml'
  if (name.endsWith('.jsonc')) return 'jsonc'
  return 'json'
}

// shell loads the sui-devtools island eagerly but not sui-code-editor —
// fetch it the first time the source tab needs it.
function ensureCodeEditorLoaded(): void {
  if (customElements.get('sui-code-editor')) return
  if (document.querySelector('script[data-sui-code-editor]')) return
  const s = document.createElement('script')
  s.type = 'module'
  s.src = '/_sui/sui-components/sui-code-editor.js'
  s.dataset['suiCodeEditor'] = '1'
  document.head.appendChild(s)
}

let cachedCompletions: CompletionSource[] | null = null
let inflightCompletions: Promise<CompletionSource[]> | null = null

async function loadCompletionSources(): Promise<CompletionSource[]> {
  if (cachedCompletions) return cachedCompletions
  if (inflightCompletions) return inflightCompletions
  inflightCompletions = (async () => {
    const [macros, schema] = await Promise.all([
      fetch('/api/assets/jinja-macros.json', { credentials: 'same-origin' })
        .then(r => r.ok ? r.json() as Promise<MacroDescriptor[]> : null).catch(() => null),
      fetch('/api/v1/ship/schema', { credentials: 'same-origin' })
        .then(r => r.ok ? r.json() as Promise<{ data?: { ops?: OpDescriptor[] } }> : null).catch(() => null),
    ])
    const sources: CompletionSource[] = []
    if (Array.isArray(macros)) {
      const items: CompletionItem[] = macros.map(m => ({
        label: m.name,
        ...(m.signature !== undefined ? { detail: m.signature } : {}),
        ...(m.summary !== undefined || m.description !== undefined ? { info: m.summary ?? m.description ?? '' } : {}),
        insert: `${m.name}()`,
      }))
      sources.push({ trigger: '{{ ', items }, { trigger: '{% ', items })
    }
    const ops = schema?.data?.ops
    if (Array.isArray(ops)) {
      const items: CompletionItem[] = ops.map(op => ({
        label: op.name,
        detail: `→ ${op.returns ?? '?'}`,
        info: `${op.description ?? ''}\n\nargs: ${op.args?.map(a => `${a.name}${a.required ? '' : '?'}: ${a.kind}`).join(', ') ?? ''}\nperm: ${op.permission ?? '?'}`,
        insert: `${op.name}()`,
      }))
      sources.push({ trigger: 'ship.', items }, { trigger: 'ship.try.', items })
    }
    cachedCompletions = sources
    return sources
  })()
  try { return await inflightCompletions } finally { inflightCompletions = null }
}

function pushEditorDiagnostics(el: CodeEditor | null, findings: Diagnostic[]): void {
  if (!el?.setDiagnostics) return
  el.setDiagnostics(findings.map(f => ({
    line: f.line,
    column: f.col,
    severity: f.severity === 'warn' || f.severity === 'warning' ? 'warning' : f.severity === 'info' ? 'info' : 'error',
    message: f.message,
    source: f.rule,
  })))
}

export default controller<Scope>({
  scope: () => {
    const src = signal<ViewSource | null>(null)
    const draft = signal<string | null>(null)
    const saving = signal(false)
    const diagnostics = signal<Diagnostic[]>([])
    const diagMsg = signal<string | null>(null)
    const loading = signal(false)
    const loadError = signal<string | null>(null)

    const isDirty = derived(() => draft() !== null && draft() !== src()?.template)
    const showDiag = derived(() => diagnostics().length > 0 || diagMsg() !== null)
    const editorValue = derived(() => draft() ?? src()?.template ?? '')
    const editorReadonly = derived(() => src()?.scope !== 'global')
    const manifestLangVal = derived(() => manifestLang(src()?.manifest_filename ?? null))
    const templateRelPath = derived(() => {
      const s = src()
      if (!s) return 'index.html.j2'
      const marker = `/views/${s.view_name.split('.')[0]}/`
      const idx = s.template_path.indexOf(marker)
      return idx < 0 ? 'index.html.j2' : s.template_path.slice(idx + marker.length)
    })

    const headerEl = ref<HTMLDivElement>(null)
    const templateEditorEl = ref<HTMLElement>(null)
    const manifestEditorEl = ref<HTMLElement>(null)

    return { src, draft, saving, diagnostics, diagMsg, loading, loadError, isDirty, showDiag, editorValue, editorReadonly, manifestLangVal, templateRelPath, headerEl, templateEditorEl, manifestEditorEl }
  },

  mount(state, host) {
    const s = state as Record<string, unknown>
    ensureCodeEditorLoaded()

    // The proxy `state` passes through reads (returns the signal itself)
    // and converts writes into `.set()` calls. So reads must invoke the
    // signal-as-getter; writes can stay as `s['key'] = value`.
    const srcSig = s['src'] as () => ViewSource | null
    const editorValueSig = s['editorValue'] as () => string
    const templateRelPathSig = s['templateRelPath'] as () => string
    const templateEditorRef = s['templateEditorEl'] as () => HTMLElement | null
    const manifestEditorRef = s['manifestEditorEl'] as () => HTMLElement | null
    const headerRef = s['headerEl'] as () => HTMLElement | null
    const draftSig = s['draft'] as () => string | null

    let editorWired: CodeEditor | null = null
    let completionsApplied = false

    // Two-phase sync. Phase 1 reacts to ref attachment + value derivation
    // changes (both are signals). Phase 2 forces a microtask-deferred
    // re-check because the {#if src()} branch and the bind:this ref
    // attach can land in the same microtask, leaving the effect snapshot
    // with el=null even though v already has the template string. Without
    // the deferred re-check the editor stays empty after first load.
    function tryAssignValue(): void {
      const el = templateEditorRef() as CodeEditor | null
      const v = editorValueSig()
      if (!el) return
      if (el.value !== v) el.value = v
      if (el !== editorWired) {
        editorWired = el
        el.addEventListener('sui-change', (e: Event) => {
          const val = (e as CustomEvent<{ value: string }>).detail?.value
          if (typeof val === 'string') s['draft'] = val
        })
        el.addEventListener('sui-save', (e: Event) => {
          const val = (e as CustomEvent<{ value: string }>).detail?.value
          if (typeof val === 'string') s['draft'] = val
          void save()
        })
        if (!completionsApplied && srcSig()?.scope === 'global') {
          completionsApplied = true
          void loadCompletionSources().then(srcs => el.setCompletions?.(srcs)).catch(() => {})
        }
      }
    }

    const stopSync = effect(() => {
      // Track the signals so subsequent updates re-fire this effect.
      templateEditorRef()
      editorValueSig()
      tryAssignValue()
      // Deferred second pass — handles the case where the template's
      // bind:this attaches in the same tick as the value change.
      queueMicrotask(tryAssignValue)
    })

    const stopManifestSync = effect(() => {
      const el = manifestEditorRef() as CodeEditor | null
      const srcObj = srcSig()
      if (!el) return
      const desired = srcObj?.manifest ?? ''
      if (el.value !== desired) el.value = desired
      // Same deferred-pass for the manifest editor.
      queueMicrotask(() => {
        const el2 = manifestEditorRef() as CodeEditor | null
        const s2 = srcSig()
        if (el2 && s2?.manifest != null && el2.value !== s2.manifest) el2.value = s2.manifest
      })
    })

    const headerEl = headerRef()
    if (headerEl) {
      headerEl.addEventListener('click', (e: MouseEvent) => {
        const btn = (e.target as Element).closest<HTMLButtonElement>('[data-action]')
        if (!btn) return
        if (btn.dataset['action'] === 'save') void save()
        else if (btn.dataset['action'] === 'fork') void fork()
      })
    }

    function stripPrefix(path: string): string {
      const m = path.match(/^\/api\/v\/(.+)$/)
      return m ? m[1]! : path.replace(/^\/+/, '')
    }

    async function save(): Promise<void> {
      const srcObj = srcSig()
      const draftVal = draftSig()
      if (!srcObj || srcObj.scope !== 'global' || draftVal === null || draftVal === srcObj.template) return
      s['saving'] = true; s['diagnostics'] = []; s['diagMsg'] = null
      try {
        const viewName = srcObj.view_name.split('.')[0]!
        const relPath = templateRelPathSig()
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/template`, {
          method: 'PUT', credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: relPath, content: draftVal }),
        })
        if (resp.status === 422) {
          const body = await resp.json() as { findings: Diagnostic[] }
          const findings = body.findings ?? []
          s['diagnostics'] = findings
          pushEditorDiagnostics(templateEditorRef() as CodeEditor | null, findings)
          s['diagMsg'] = 'Lint errors — fix before saving.'
          return
        }
        if (!resp.ok) { s['diagMsg'] = `save failed: ${resp.status}`; return }
        s['src'] = { ...srcObj, template: draftVal }
        s['diagnostics'] = []
        ;(templateEditorRef() as CodeEditor | null)?.setDiagnostics?.([])
        chrome.notify({ tone: 'success', title: 'Saved' })
        const current = (window as unknown as { ship?: { current(): { path: string }; navigate(p: string, o?: unknown): void } }).ship?.current()
        if (current?.path) (window as unknown as { ship: { navigate(p: string, o?: unknown): void } }).ship.navigate(current.path, { replace: true })
      } catch (e) {
        s['diagMsg'] = `error: ${(e as Error).message}`
      } finally { s['saving'] = false }
    }

    async function fork(): Promise<void> {
      const srcObj = srcSig()
      if (!srcObj || srcObj.scope !== 'project') return
      s['saving'] = true; s['diagMsg'] = null
      try {
        const viewName = srcObj.view_name.split('.')[0]!
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/fork`, { method: 'POST', credentials: 'same-origin' })
        if (!resp.ok) { s['diagMsg'] = `fork failed: ${resp.status}`; return }
        chrome.notify({ tone: 'success', title: `${viewName} forked — now editable` })
        lastPath = null; await load()
      } catch (e) {
        s['diagMsg'] = (e as Error).message
      } finally { s['saving'] = false }
    }

    let lastPath: string | null = null

    async function load(): Promise<void> {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const path = current?.path ?? location.pathname
      const stripped = stripPrefix(path)
      if (!stripped) { s['src'] = null; s['draft'] = null; s['diagnostics'] = []; s['loadError'] = null; s['loading'] = false; return }
      if (stripped === lastPath && srcSig()) return
      lastPath = stripped
      s['draft'] = null; s['diagnostics'] = []; s['loading'] = true; s['loadError'] = null
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: 'same-origin' })
        if (resp.status === 404) { s['src'] = null; return }
        if (!resp.ok) { s['loadError'] = `fetch failed: ${resp.status}`; return }
        s['src'] = await resp.json() as ViewSource
      } catch (e) {
        s['loadError'] = (e as Error).message
      } finally { s['loading'] = false }
    }

    const onNavigate = () => { lastPath = null; void load() }
    document.addEventListener('sui:navigate', onNavigate)
    void load()

    Object.defineProperty(host, 'refresh', { value: () => { lastPath = null; void load() }, configurable: true })

    return () => {
      stopSync(); stopManifestSync()
      document.removeEventListener('sui:navigate', onNavigate)
    }
  },
})
