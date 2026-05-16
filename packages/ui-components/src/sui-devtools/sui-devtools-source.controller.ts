import { chrome, controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-source.sui'

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

type CodeEditor = HTMLElement & { value?: string }

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
      } catch (e2) {
        errorMsg.set((e2 as Error).message)
      } finally {
        saving.set(false)
      }
    }

    return {
      viewName, files, selectedPath, drafts, saving, saved, errorMsg,
      loading, loadError, editorEl,
      selectedFile, editorLang, editorValue, isDirty,
      onPickFile, onChange, onSave,
      compileStatus, compileError,
    }
  },

  mount(state, host) {
    const s = state as unknown as {
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
      s.loading.set(true)
      s.loadError.set(null)
      s.files.set([])
      s.selectedPath.set('')
      s.drafts.set({})
      try {
        const resp = await fetch(
          `/api/v1/views/${encodeURIComponent(name)}/files`,
          { credentials: 'same-origin' },
        )
        if (!resp.ok) {
          s.loadError.set(resp.status === 404 ? 'no source for this view' : `fetch failed (${resp.status})`)
          return
        }
        const body = await resp.json() as FilesResponse
        const enriched: ViewFile[] = (body.files ?? []).map((f) => {
          const sub = subrouteOf(f.path)
          return sub ? { ...f, subroute: sub } : f
        })
        s.files.set(enriched)
        const def = enriched.find((f) => f.path === 'page.sui') ?? enriched[0]
        if (def) s.selectedPath.set(def.path)
      } catch (e) {
        s.loadError.set((e as Error).message)
      } finally {
        s.loading.set(false)
      }
    }

    function tryAssignEditorValue(): void {
      const el = s.editorEl()
      if (!el) return
      const v = s.editorValue()
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
      s.editorEl()
      s.editorValue()
      s.selectedPath()
      s.files()
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
        const el = s.editorEl()
        const want = s.editorValue()
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
      if (next !== s.viewName()) {
        s.viewName.set(next)
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
    const onCompiled = (e: Event): void => {
      const d = (e as CustomEvent).detail
      if (!matchesView(d)) return
      compileStatusSig.set('compiled')
      compileErrorSig.set(null)
      // Auto-revert to idle so the saved chip doesn't linger forever.
      window.setTimeout(() => {
        if (compileStatusSig() === 'compiled') compileStatusSig.set('idle')
      }, 2500)
    }
    const onErrored = (e: Event): void => {
      const d = (e as CustomEvent).detail as { message?: string; line?: number; col?: number; file?: string } | undefined
      if (!matchesView(d)) return
      const loc = d?.line ? `:${d.line}${d?.col ? `:${d.col}` : ''}` : ''
      compileStatusSig.set('errored')
      compileErrorSig.set(`${d?.file ?? 'view'}${loc} — ${d?.message ?? 'compile failed'}`)
    }
    document.addEventListener('sui:view:compiled', onCompiled)
    document.addEventListener('sui:view:errored', onErrored)

    Object.defineProperty(host, 'refresh', {
      configurable: true,
      value: () => { lastLoadedView = null; void loadFiles(s.viewName()) },
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
