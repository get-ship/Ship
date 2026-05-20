import { chrome, controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-source.sui'
import { isShellLocalView } from '../../chrome/shell-surfaces'

interface ViewFile {
  path: string
  name: string
  kind: string
  size: number
  content?: string
  subroute?: string
}

interface FilesResponse {
  view_name: string
  view_dir: string
  files: Array<{ path: string; name: string; kind: string; size: number; content?: string }>
}

interface SuiDiagnostic {
  line: number
  col?: number
  severity?: 'error' | 'warning' | 'info'
  message: string
  source?: string
}
type CodeEditor = HTMLElement & {
  value?: string
  setDiagnostics?: (diags: SuiDiagnostic[]) => void
}

const LANG_BY_KIND: Record<string, string> = {
  sui: 'html',
  ts: 'javascript',
  tsx: 'javascript',
  css: 'css',
  jsonc: 'json',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  md: 'markdown',
}

function ensureCodeEditorLoaded(): void {
  if (customElements.get('sui-code-editor')) return
  if (document.querySelector('script[data-sui-code-editor]')) return
  const s = document.createElement('script')
  s.type = 'module'
  s.src = '/_sui/sui-components/sui-code-editor.js'
  s.dataset['suiCodeEditor'] = '1'
  document.head.appendChild(s)
}

function viewNameFromRoute(path: string): string {
  const seg = path.split('/').filter(Boolean)[0]
  return seg ?? ''
}

function subrouteOf(rel: string): string | undefined {
  const idx = rel.indexOf('/')
  if (idx < 0) return undefined
  const first = rel.slice(0, idx)
  if (first.startsWith('[') && first.endsWith(']')) return first
  return undefined
}

export default controller<Scope>({
  scope: () => {
    const viewName = signal<string>('')
    const files = signal<ViewFile[]>([])
    const selectedPath = signal<string>('')
    const drafts = signal<Record<string, string>>({})
    const saving = signal(false)
    const saved = signal(false)
    const errorMsg = signal<string | null>(null)
    const loading = signal(false)
    const loadError = signal<string | null>(null)
    const editorEl = ref<CodeEditor>(null)
    // Compile feedback from the per-view SSE bridge (chrome opens the
    // EventSource on edit-mode-enter; events arrive on document).
    // 'idle' | 'compiling' | 'compiled' | 'errored'
    const compileStatus = signal<'idle' | 'compiling' | 'compiled' | 'errored'>('idle')
    const compileError = signal<string | null>(null)
    // Snapshot history — list pulled via __viewMcp.listSnapshots when
    // the user opens the History popover. Empty until first open.
    interface Snap { snapshot_id: string; timestamp: string; label?: string; agent_id: string }
    const snapshots = signal<Snap[]>([])
    const snapshotsLoading = signal(false)
    const snapshotsOpen = signal(false)
    const snapshotsError = signal<string | null>(null)

    const selectedFile = derived<ViewFile | null>(() => {
      const p = selectedPath()
      return files().find((f) => f.path === p) ?? null
    })

    const editorLang = derived<string>(() => {
      const k = selectedFile()?.kind ?? ''
      return LANG_BY_KIND[k] ?? ''
    })

    const editorValue = derived<string>(() => {
      const p = selectedPath()
      const d = drafts()[p]
      if (typeof d === 'string') return d
      return selectedFile()?.content ?? ''
    })

    const isDirty = derived<boolean>(() => {
      const p = selectedPath()
      const d = drafts()[p]
      if (typeof d !== 'string') return false
      return d !== (selectedFile()?.content ?? '')
    })

    function onPickFile(e: MouseEvent): void {
      const row = (e.target as Element).closest<HTMLElement>('[data-path]')
      if (!row?.dataset['path']) return
      selectedPath.set(row.dataset['path'])
      saved.set(false)
      errorMsg.set(null)
    }

    function onChange(e: Event): void {
      const val = (e as CustomEvent<{ value: string }>).detail?.value
      const p = selectedPath()
      if (typeof val !== 'string' || !p) return
      const next = { ...drafts() }
      next[p] = val
      drafts.set(next)
      saved.set(false)
    }

    async function onSave(): Promise<void> {
      const p = selectedPath()
      const file = files().find((f) => f.path === p)
      const draft = drafts()[p]
      const name = viewName()
      if (!file || typeof draft !== 'string' || draft === file.content || !name) return
      saving.set(true)
      errorMsg.set(null)
      compileStatus.set('compiling')
      compileError.set(null)
      try {
        const resp = await fetch(
          `/api/v1/views/${encodeURIComponent(name)}/template`,
          {
            method: 'PUT',
            credentials: 'same-origin',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ file: p, content: draft }),
          },
        )
        if (!resp.ok) {
          const text = await resp.text().catch(() => '')
          errorMsg.set(text || `save failed (${resp.status})`)
          return
        }
        files.set(files().map((f) => f.path === p ? { ...f, content: draft } : f))
        const nextDrafts = { ...drafts() }
        delete nextDrafts[p]
        drafts.set(nextDrafts)
        saved.set(true)
        chrome.notify({ tone: 'success', title: `Saved ${p}` })
        window.setTimeout(() => saved.set(false), 1500)
        // Auto-refresh the snapshots list if the panel is open — the
        // daemon creates a snapshot on every successful save, so users
        // expect to see it appear without closing + reopening the panel.
        if (snapshotsOpen()) {
          const mcp = (window as unknown as { __viewMcp?: typeof import('../../../../apps/shell/src/chrome/view-mcp').viewMcp }).__viewMcp
          if (mcp) {
            mcp.listSnapshots(name, 50).then((r) => {
              snapshots.set(r.snapshots ?? [])
            }).catch(() => { /* best-effort; manual reopen still works */ })
          }
        }
      } catch (e2) {
        errorMsg.set((e2 as Error).message)
      } finally {
        saving.set(false)
      }
    }

    async function onOpenSnapshots(): Promise<void> {
      const name = viewName()
      if (!name) return
      snapshotsOpen.set(!snapshotsOpen())
      if (!snapshotsOpen()) return
      snapshotsLoading.set(true)
      snapshotsError.set(null)
      const mcp = (window as unknown as { __viewMcp?: typeof import('../../../../apps/shell/src/chrome/view-mcp').viewMcp }).__viewMcp
      if (!mcp) {
        snapshotsError.set('MCP not available — enter edit mode first.')
        snapshotsLoading.set(false)
        return
      }
      try {
        const r = await mcp.listSnapshots(name, 50)
        snapshots.set(r.snapshots ?? [])
      } catch (e) {
        snapshotsError.set((e as Error).message)
      } finally {
        snapshotsLoading.set(false)
      }
    }

    async function onRestoreSnapshot(snapshot_id: string): Promise<void> {
      const name = viewName()
      const mcp = (window as unknown as { __viewMcp?: typeof import('../../../../apps/shell/src/chrome/view-mcp').viewMcp }).__viewMcp
      if (!name || !mcp) return
      try {
        await mcp.restoreSnapshot(name, snapshot_id)
        snapshotsOpen.set(false)
        // Refresh files to pull restored content into the editor.
        const enriched = files().map((f) => ({ ...f }))
        files.set(enriched)
        // Force a reload of files (lastLoadedView reset in refresh).
        ;(window as unknown as { dispatchEvent: typeof document.dispatchEvent }).dispatchEvent(
          new CustomEvent('sui:navigate', { detail: { to: location.pathname } }),
        )
      } catch (e) {
        snapshotsError.set((e as Error).message)
      }
    }

    return {
      viewName, files, selectedPath, drafts, saving, saved, errorMsg,
      loading, loadError, editorEl,
      selectedFile, editorLang, editorValue, isDirty,
      onPickFile, onChange, onSave,
      compileStatus, compileError,
      snapshots, snapshotsLoading, snapshotsOpen, snapshotsError,
      onOpenSnapshots, onRestoreSnapshot,
    }
  },

  mount(state, host) {
    // sui-runtime mount-state proxy: writes are plain assignments
    // (`s.<sig> = next`). Reads of the *value* still need `()` on the
    // signal — for that we keep a parallel `readState` alias.
    const s = state as unknown as {
      viewName: string
      files: ViewFile[]
      selectedPath: string
      drafts: Record<string, string>
      loading: boolean
      loadError: string | null
      editorEl: () => CodeEditor | null
      editorValue: () => string
    }
    const readState = state as unknown as {
      viewName: ReturnType<typeof signal<string>>
      files: ReturnType<typeof signal<ViewFile[]>>
      selectedPath: ReturnType<typeof signal<string>>
      drafts: ReturnType<typeof signal<Record<string, string>>>
      loading: ReturnType<typeof signal<boolean>>
      loadError: ReturnType<typeof signal<string | null>>
      editorEl: () => CodeEditor | null
      editorValue: () => string
    }
    ensureCodeEditorLoaded()

    let editorWired: CodeEditor | null = null
    let lastLoadedView: string | null = null

    async function loadFiles(name: string): Promise<void> {
      if (!name || lastLoadedView === name) return
      lastLoadedView = name
      // Shell-local surfaces (e.g. studio) have no daemon source — their
      // files are bundled into the shell. Skip the fetch so it doesn't 404.
      if (isShellLocalView(name)) {
        s.files = []
        s.selectedPath = ''
        s.loadError = 'shell-local surface — source is bundled into the shell'
        return
      }
      s.loading = true
      s.loadError = null
      s.files = []
      s.selectedPath = ''
      s.drafts = {}
      try {
        const resp = await fetch(
          `/api/v1/views/${encodeURIComponent(name)}/files`,
          { credentials: 'same-origin' },
        )
        if (!resp.ok) {
          s.loadError = resp.status === 404 ? 'no source for this view' : `fetch failed (${resp.status})`
          return
        }
        const body = await resp.json() as FilesResponse
        const enriched: ViewFile[] = (body.files ?? []).map((f) => {
          const sub = subrouteOf(f.path)
          return sub ? { ...f, subroute: sub } : f
        })
        s.files = enriched
        const def = enriched.find((f) => f.path === 'page.sui') ?? enriched[0]
        if (def) s.selectedPath = def.path
      } catch (e) {
        s.loadError = (e as Error).message
      } finally {
        s.loading = false
      }
    }

    function tryAssignEditorValue(): void {
      const el = readState.editorEl()
      if (!el) return
      const v = readState.editorValue()
      if (el.value !== v) el.value = v
      if (el !== editorWired) {
        editorWired = el
        // sui-change is hooked via on:sui-change in the template;
        // also bind sui-save here so Cmd/Ctrl+S triggers the save flow.
        el.addEventListener('sui-save', () => { void (state as unknown as { onSave: () => Promise<void> }).onSave() })
      }
    }

    // Wait for the editor's custom element to upgrade before we trust
    // that setting `.value` actually drives the inner CodeMirror.
    // Before this, the prop setter exists but the editor's mount effect
    // hasn't run, so writes get lost.
    let editorUpgraded = false
    void customElements.whenDefined('sui-code-editor').then(() => {
      editorUpgraded = true
      tryAssignEditorValue()
    })

    const stopSync = effect(() => {
      // Track every reactive read this effect cares about.
      readState.editorEl()
      readState.editorValue()
      readState.selectedPath()
      readState.files()
      if (editorUpgraded) tryAssignEditorValue()
      // The template can re-render the <sui-code-editor> after the
      // effect snapshot. Poll a few frames so the value lands once the
      // host element appears in the DOM.
      let attempts = 0
      const tick = (): void => {
        attempts++
        if (!editorUpgraded) {
          if (attempts < 60) requestAnimationFrame(tick)
          return
        }
        const el = readState.editorEl()
        const want = readState.editorValue()
        if (el && el.value === want && want.length > 0) return
        tryAssignEditorValue()
        if (attempts < 60) requestAnimationFrame(tick)
      }
      requestAnimationFrame(tick)
    })

    function syncViewFromRoute(): void {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const pathname = current?.path ?? location.pathname
      const next = viewNameFromRoute(pathname)
      if (!next) return
      if (next !== readState.viewName()) {
        s.viewName = next
        void loadFiles(next)
      }
    }
    syncViewFromRoute()
    const onNav = () => syncViewFromRoute()
    document.addEventListener('sui:navigate', onNav)
    window.addEventListener('popstate', onNav)

    // ── Compile feedback from per-view SSE bridge ────────────────
    const matchesView = (detail: unknown): boolean => {
      const d = detail as { view_name?: string; view?: string } | undefined
      const name = (state as unknown as { viewName: () => string }).viewName()
      const v = d?.view_name ?? d?.view
      return !v || v === name
    }
    const compileStatusSig = (state as unknown as { compileStatus: { set(v: string): void; (): string } }).compileStatus
    const compileErrorSig = (state as unknown as { compileError: { set(v: string | null): void; (): string | null } }).compileError
    // Push compile diagnostics into the CodeMirror gutter on the active
    // editor instance. The editor exposes setDiagnostics(diags[]) per
    // its sui-code-editor.sui prop/method surface; we call it on every
    // errored event (with the new diag) and clear on every compiled.
    function pushDiagnosticsToEditor(diags: SuiDiagnostic[]): void {
      const el = readState.editorEl()
      if (!el?.setDiagnostics) return
      el.setDiagnostics(diags)
    }
    const onCompiled = (e: Event): void => {
      const d = (e as CustomEvent).detail
      if (!matchesView(d)) return
      compileStatusSig.set('compiled')
      compileErrorSig.set(null)
      pushDiagnosticsToEditor([])
      // Auto-revert to idle so the saved chip doesn't linger forever.
      window.setTimeout(() => {
        if (compileStatusSig() === 'compiled') compileStatusSig.set('idle')
      }, 2500)
    }
    const onErrored = (e: Event): void => {
      const d = (e as CustomEvent).detail as { message?: string; line?: number; col?: number; file?: string; severity?: 'error' | 'warning' | 'info' } | undefined
      if (!matchesView(d)) return
      const loc = d?.line ? `:${d.line}${d?.col ? `:${d.col}` : ''}` : ''
      compileStatusSig.set('errored')
      compileErrorSig.set(`${d?.file ?? 'view'}${loc} — ${d?.message ?? 'compile failed'}`)
      pushDiagnosticsToEditor([{
        line: d?.line ?? 1,
        col: d?.col,
        severity: d?.severity ?? 'error',
        message: d?.message ?? 'compile failed',
        source: 'sui-compiler',
      }])
    }
    document.addEventListener('sui:view:compiled', onCompiled)
    document.addEventListener('sui:view:errored', onErrored)

    Object.defineProperty(host, 'refresh', {
      configurable: true,
      value: () => { lastLoadedView = null; void loadFiles(readState.viewName()) },
    })

    return () => {
      stopSync()
      document.removeEventListener('sui:navigate', onNav)
      window.removeEventListener('popstate', onNav)
      document.removeEventListener('sui:view:compiled', onCompiled)
      document.removeEventListener('sui:view:errored', onErrored)
    }
  },
})
