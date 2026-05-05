import { controller, signal, derived, effect } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-source.sui'

interface ViewSource { view_name: string; template_path: string; template: string; manifest_filename: string | null; manifest: string | null; scope: string }
interface Diagnostic { line: number; col: number; rule: string; message: string; severity?: string }

type CodeEditor = HTMLElement & {
  value?: string
  readonly?: boolean
  setDiagnostics?(d: Array<{ line: number; column?: number; severity: 'error' | 'warning' | 'info'; message: string; source?: string }>): void
}

function manifestLang(name: string | null): string {
  if (!name) return ''
  if (name.endsWith('.yaml') || name.endsWith('.yml')) return 'yaml'
  if (name.endsWith('.jsonc')) return 'jsonc'
  return 'json'
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
      if (idx < 0) return 'index.html.j2'
      return s.template_path.slice(idx + marker.length)
    })

    return { src, draft, saving, diagnostics, diagMsg, loading, loadError, isDirty, showDiag, editorValue, editorReadonly, manifestLangVal, templateRelPath }
  },

  mount(state, host) {
    const s = state as Record<string, unknown>

    const readRef = (key: string): HTMLElement | null => {
      const fn = (state as Record<string, unknown>)[key]
      return typeof fn === 'function' ? (fn as () => HTMLElement | null)() : null
    }

    // Sync editor value reactively (no innerHTML)
    let editorWired: CodeEditor | null = null
    const stopSync = effect(() => {
      const el = readRef('templateEditorEl') as CodeEditor | null
      const v = s['editorValue'] as string
      if (el && el.value !== v) el.value = v
      if (el && el !== editorWired) {
        editorWired = el
        el.addEventListener('ship-change', (e: Event) => {
          const val = (e as CustomEvent<{ value: string }>).detail?.value
          if (typeof val === 'string') s['draft'] = val
        })
        el.addEventListener('ship-save', (e: Event) => {
          const val = (e as CustomEvent<{ value: string }>).detail?.value
          if (typeof val === 'string') s['draft'] = val
          void save()
        })
      }
    })

    // Sync manifest editor value
    const stopManifestSync = effect(() => {
      const el = readRef('manifestEditorEl') as CodeEditor | null
      const srcObj = s['src'] as ViewSource | null
      if (el && srcObj?.manifest != null && el.value !== srcObj.manifest) {
        el.value = srcObj.manifest
      }
    })

    // Header click delegation (fork / save buttons)
    const headerEl = readRef('headerEl')
    if (headerEl) {
      headerEl.addEventListener('click', (e: MouseEvent) => {
        const btn = (e.target as Element).closest<HTMLButtonElement>('[data-action]')
        if (!btn) return
        if (btn.dataset.action === 'save') void save()
        else if (btn.dataset.action === 'fork') void fork()
      })
    }

    function stripPrefix(path: string): string {
      const m = path.match(/^\/api\/v\/(.+)$/)
      return m ? m[1]! : path.replace(/^\/+/, '')
    }

    async function save(): Promise<void> {
      const srcObj = s['src'] as ViewSource | null
      const draftVal = s['draft'] as string | null
      if (!srcObj || srcObj.scope !== 'global' || draftVal === null || draftVal === srcObj.template) return
      s['saving'] = true
      s['diagnostics'] = []
      s['diagMsg'] = null
      try {
        const viewName = srcObj.view_name.split('.')[0]!
        const relPath = s['templateRelPath'] as string
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/template`, {
          method: 'PUT',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: relPath, content: draftVal }),
        })
        if (resp.status === 422) {
          const body = await resp.json() as { findings: Diagnostic[] }
          s['diagnostics'] = body.findings ?? []
          return
        }
        if (!resp.ok) { s['diagMsg'] = `save failed: ${resp.status}`; return }
        s['src'] = { ...srcObj, template: draftVal }
        s['diagnostics'] = []
        const editor = readRef('templateEditorEl') as CodeEditor | null
        editor?.setDiagnostics?.([])
        host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: 'Saved' }, bubbles: true, composed: true }))
        const current = (window as unknown as { ship?: { current(): { path: string }; navigate(p: string, o?: unknown): void } }).ship?.current()
        if (current?.path) (window as unknown as { ship: { navigate(p: string, o?: unknown): void } }).ship.navigate(current.path, { replace: true })
      } catch (e) {
        s['diagMsg'] = `error: ${(e as Error).message}`
      } finally {
        s['saving'] = false
      }
    }

    async function fork(): Promise<void> {
      const srcObj = s['src'] as ViewSource | null
      if (!srcObj || srcObj.scope !== 'project') return
      s['saving'] = true
      s['diagMsg'] = null
      try {
        const viewName = srcObj.view_name.split('.')[0]!
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/fork`, { method: 'POST', credentials: 'same-origin' })
        if (!resp.ok) { s['diagMsg'] = `fork failed: ${resp.status}`; return }
        host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: `${viewName} forked — now editable` }, bubbles: true, composed: true }))
        lastPath = null; await load()
      } catch (e) {
        s['diagMsg'] = (e as Error).message
      } finally {
        s['saving'] = false
      }
    }

    let lastPath: string | null = null

    async function load(): Promise<void> {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const path = current?.path ?? location.pathname
      const stripped = stripPrefix(path)
      if (!stripped) { s['src'] = null; s['draft'] = null; s['diagnostics'] = []; s['loadError'] = null; s['loading'] = false; return }
      if (stripped === lastPath && s['src']) return
      lastPath = stripped
      s['draft'] = null
      s['diagnostics'] = []
      s['loading'] = true
      s['loadError'] = null
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: 'same-origin' })
        if (resp.status === 404) { s['src'] = null; return }
        if (!resp.ok) { s['loadError'] = `fetch failed: ${resp.status}`; return }
        s['src'] = await resp.json() as ViewSource
      } catch (e) {
        s['loadError'] = (e as Error).message
      } finally {
        s['loading'] = false
      }
    }

    const onNavigate = () => { lastPath = null; void load() }
    document.addEventListener('sui:navigate', onNavigate)
    void load()

    Object.defineProperty(host, 'refresh', { value: () => { lastPath = null; void load() }, configurable: true })

    return () => {
      stopSync()
      stopManifestSync()
      document.removeEventListener('sui:navigate', onNavigate)
    }
  },
})
