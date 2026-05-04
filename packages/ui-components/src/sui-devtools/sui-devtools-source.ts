import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-source.sui'

interface ViewSource { view_name: string; template_path: string; template: string; manifest_filename: string | null; manifest: string | null; scope: string }
interface Diagnostic { line: number; col: number; rule: string; message: string; severity?: string }

type CodeEditor = HTMLElement & {
  value?: string
  readonly?: boolean
  setDiagnostics?(d: Array<{ line: number; column?: number; severity: 'error' | 'warning' | 'info'; message: string; source?: string }>): void
  setCompletions?(s: Array<{ trigger: string; items: Array<{ label: string; detail?: string; info?: string; insert?: string }> }>): void
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function manifestLang(name: string | null): string {
  if (!name) return ''
  if (name.endsWith('.yaml') || name.endsWith('.yml')) return 'yaml'
  if (name.endsWith('.jsonc')) return 'jsonc'
  return 'json'
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const headerEl = scope.headerEl() as HTMLElement
    const splitEl = scope.splitEl() as HTMLElement
    const diagEl = scope.diagEl() as HTMLElement

    let src: ViewSource | null = null
    let draft: string | null = null
    let saving = false
    let diagnostics: Diagnostic[] = []
    let lastPath: string | null = null
    let templateEditor: CodeEditor | null = null
    let manifestEditor: CodeEditor | null = null

    function renderHeader(): void {
      if (!src) { headerEl.innerHTML = ''; return }
      const dirty = draft !== null && draft !== src.template
      const dirtyDot = dirty ? '<span class="dirty-dot" title="Unsaved changes"></span>' : ''
      const forkBtn = src.scope === 'project' ? `<button class="edit-action primary" data-action="fork" ${saving ? 'disabled' : ''}>Fork to edit</button>` : ''
      const saveBtn = src.scope === 'global' ? `<button class="edit-action primary" data-action="save" ${saving || !dirty ? 'disabled' : ''} title="Save (⌘/Ctrl+S)">${saving ? 'Saving…' : 'Save'}</button>` : ''
      headerEl.innerHTML = `<span style="color:var(--foreground);font-weight:var(--font-weight-medium,500)">${esc(src.view_name)}</span><span style="padding:1px 6px;border:1px solid var(--border);border-radius:999px;text-transform:uppercase;letter-spacing:.04em;font-size:10px">${esc(src.scope)}</span>${dirtyDot}<span style="flex:1"></span><span>${esc(src.template_path)}</span><span style="display:inline-flex;align-items:center;gap:6px">${forkBtn}${saveBtn}</span>`
    }

    function renderDiag(): void {
      const show = diagnostics.length > 0
      diagEl.style.display = show ? '' : 'none'
      if (show) diagEl.innerHTML = diagnostics.map(d => `<div>${d.line}:${d.col} [${esc(d.rule)}] ${esc(d.message)}</div>`).join('')
    }

    function buildSplit(): void {
      if (!src) {
        splitEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="file-code"></sui-icon><span slot="title">No source available</span>This path is not served by a file-routed view.</sui-empty-state>`
        templateEditor = null; manifestEditor = null
        return
      }
      // Build template pane
      const tPane = document.createElement('div')
      tPane.className = 'pane'
      const tHead = document.createElement('div')
      tHead.className = 'pane-head'
      tHead.textContent = 'index.html.j2'
      const tEditorWrap = document.createElement('div')
      tEditorWrap.className = 'pane-editor'
      const tEditor = document.createElement('ship-code-editor') as CodeEditor
      tEditor.setAttribute('language', 'html')
      if (src.scope !== 'global') tEditor.setAttribute('readonly', '')
      tEditor.value = draft ?? src.template
      tEditorWrap.appendChild(tEditor)
      tPane.appendChild(tHead)
      tPane.appendChild(tEditorWrap)
      templateEditor = tEditor

      splitEl.innerHTML = ''
      splitEl.appendChild(tPane)

      if (src.manifest) {
        const mHead = document.createElement('div')
        mHead.className = 'pane-head manifest'
        mHead.textContent = src.manifest_filename ?? ''
        const mPane = document.createElement('div')
        mPane.className = 'pane'
        const mEditorWrap = document.createElement('div')
        mEditorWrap.className = 'pane-editor'
        const mEditor = document.createElement('ship-code-editor') as CodeEditor
        mEditor.setAttribute('language', manifestLang(src.manifest_filename))
        mEditor.setAttribute('readonly', '')
        mEditor.value = src.manifest
        mEditorWrap.appendChild(mEditor)
        mPane.appendChild(mEditorWrap)
        manifestEditor = mEditor
        splitEl.appendChild(mHead)
        splitEl.appendChild(mPane)
      } else {
        manifestEditor = null
      }

      // Wire editor events
      tEditor.addEventListener('ship-change', (e: Event) => {
        const val = (e as CustomEvent<{ value: string }>).detail?.value
        if (typeof val === 'string') { draft = val; renderHeader() }
      })
      tEditor.addEventListener('ship-save', (e: Event) => {
        const val = (e as CustomEvent<{ value: string }>).detail?.value
        if (typeof val === 'string') draft = val
        void save()
      })
    }

    function stripPrefix(path: string): string {
      const m = path.match(/^\/api\/v\/(.+)$/)
      return m ? m[1]! : path.replace(/^\/+/, '')
    }

    function templateRelPath(): string {
      if (!src) return 'index.html.j2'
      const marker = `/views/${src.view_name.split('.')[0]}/`
      const idx = src.template_path.indexOf(marker)
      if (idx < 0) return 'index.html.j2'
      return src.template_path.slice(idx + marker.length)
    }

    async function save(): Promise<void> {
      if (!src || src.scope !== 'global' || draft === null || draft === src.template) return
      saving = true; renderHeader()
      diagnostics = []
      try {
        const viewName = src.view_name.split('.')[0]!
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/template`, {
          method: 'PUT',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ file: templateRelPath(), content: draft }),
        })
        if (resp.status === 422) {
          const body = await resp.json() as { findings: Diagnostic[] }
          diagnostics = body.findings ?? []
          renderDiag()
          return
        }
        if (!resp.ok) { diagEl.style.display = ''; diagEl.textContent = `save failed: ${resp.status}`; return }
        src = { ...src, template: draft! }
        diagnostics = []
        renderDiag()
        templateEditor?.setDiagnostics?.([])
        host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: 'Saved' }, bubbles: true, composed: true }))
        const current = (window as unknown as { ship?: { current(): { path: string }; navigate(p: string, o?: unknown): void } }).ship?.current()
        if (current?.path) (window as unknown as { ship: { navigate(p: string, o?: unknown): void } }).ship.navigate(current.path, { replace: true })
      } catch (e) {
        diagEl.style.display = ''; diagEl.textContent = `error: ${(e as Error).message}`
      } finally {
        saving = false; renderHeader()
      }
    }

    async function fork(): Promise<void> {
      if (!src || src.scope !== 'project') return
      saving = true; renderHeader()
      try {
        const viewName = src.view_name.split('.')[0]!
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/fork`, { method: 'POST', credentials: 'same-origin' })
        if (!resp.ok) { diagEl.style.display = ''; diagEl.textContent = `fork failed: ${resp.status}`; return }
        host.dispatchEvent(new CustomEvent('sui:notify', { detail: { tone: 'success', title: `${viewName} forked — now editable` }, bubbles: true, composed: true }))
        lastPath = null; await load()
      } catch (e) {
        diagEl.style.display = ''; diagEl.textContent = (e as Error).message
      } finally {
        saving = false; renderHeader()
      }
    }

    headerEl.addEventListener('click', (e: MouseEvent) => {
      const btn = (e.target as Element).closest<HTMLButtonElement>('[data-action]')
      if (!btn) return
      if (btn.dataset.action === 'save') void save()
      else if (btn.dataset.action === 'fork') void fork()
    })

    async function load(): Promise<void> {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const path = current?.path ?? location.pathname
      const stripped = stripPrefix(path)
      if (!stripped) { src = null; draft = null; diagnostics = []; buildSplit(); renderHeader(); return }
      if (stripped === lastPath && src) return
      lastPath = stripped; draft = null; diagnostics = []
      splitEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="file-code"></sui-icon><span slot="title">Loading source…</span>Fetching the view template from the daemon.</sui-empty-state>`
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: 'same-origin' })
        if (resp.status === 404) { src = null; buildSplit(); renderHeader(); return }
        if (!resp.ok) { splitEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="alert-triangle"></sui-icon><span slot="title">Source error</span>fetch failed: ${resp.status}</sui-empty-state>`; return }
        src = await resp.json() as ViewSource
        buildSplit(); renderHeader()
      } catch (e) {
        splitEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="alert-triangle"></sui-icon><span slot="title">Source error</span>${esc((e as Error).message)}</sui-empty-state>`
      }
    }

    const onNavigate = () => { lastPath = null; void load() }
    document.addEventListener('sui:navigate', onNavigate)
    void load()

    Object.defineProperty(host, 'refresh', { value: () => { lastPath = null; void load() }, configurable: true })

    return () => document.removeEventListener('sui:navigate', onNavigate)
  },
})
