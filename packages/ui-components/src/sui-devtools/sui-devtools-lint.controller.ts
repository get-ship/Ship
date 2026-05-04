import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-lint.sui'

interface Finding {
  file: string
  line: number
  col: number
  rule: string
  severity: 'error' | 'warn' | string
  message: string
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const toolbarEl = scope.toolbarEl() as HTMLElement
    const bodyEl = scope.bodyEl() as HTMLElement

    let findings: Finding[] = []
    let viewName: string | null = null
    let loading = false

    function topLevelName(): string | null {
      const p = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current().path ?? location.pathname
      const m = p.match(/^\/api\/v\/([^/]+)/)
      if (m) return m[1]!
      const seg = p.replace(/^\/+/, '').split('/')[0]
      return seg || null
    }

    function render(): void {
      const errs = findings.filter(f => f.severity === 'error').length
      const warns = findings.filter(f => f.severity === 'warn').length
      const spacer = '<span style="flex:1"></span>'
      const errBadge = errs > 0 ? `<span class="count err">${errs} error</span>` : ''
      const warnBadge = warns > 0 ? `<span class="count warn">${warns} warn</span>` : ''
      const loadingSpan = loading ? '<span>Linting…</span>' : ''
      const namePart = viewName ? `<span style="color:var(--foreground);font-weight:var(--font-weight-medium,500)">${esc(viewName)}</span>` : '<span>No view</span>'
      toolbarEl.innerHTML = `${namePart}${spacer}${errBadge}${warnBadge}${loadingSpan}`

      if (!findings.length) {
        bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="check-circle"></sui-icon><span slot="title">${viewName ? 'Clean' : 'No view'}</span>${viewName ? 'No lint errors in current view.' : 'Navigate to a view to lint it.'}</sui-empty-state>`
        return
      }

      const sorted = [...findings].sort((a, b) => {
        const sv = (s: string) => s === 'error' ? 0 : s === 'warn' ? 1 : 2
        const d = sv(a.severity) - sv(b.severity)
        if (d !== 0) return d
        if (a.file !== b.file) return a.file.localeCompare(b.file)
        return a.line !== b.line ? a.line - b.line : a.col - b.col
      })

      bodyEl.innerHTML = sorted.map(f =>
        `<div class="row"><span class="sev ${f.severity === 'error' ? 'err' : 'warn'}">${esc(f.severity)}</span><span class="loc">${esc(f.file)}:${f.line}:${f.col}</span><span><span class="rule">[${esc(f.rule)}]</span>${esc(f.message)}</span></div>`
      ).join('')
    }

    async function refresh(): Promise<void> {
      const name = topLevelName()
      viewName = name
      if (!name) { findings = []; render(); return }
      loading = true
      render()
      try {
        const resp = await fetch('/api/v1/lint/view', {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name }),
        })
        if (resp.status === 404) { findings = []; return }
        if (!resp.ok) { bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">lint failed: ${resp.status}</div>`; return }
        const body = await resp.json() as { ok: boolean; data?: { findings: Finding[] }; error?: string }
        findings = body.data?.findings ?? []
      } catch (e) {
        bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">${esc((e as Error).message)}</div>`
      } finally {
        loading = false
        render()
      }
    }

    const onNavigate = () => void refresh()
    const onDockActivate = (ev: Event) => {
      if ((ev as CustomEvent<{ tab: string }>).detail?.tab === 'lint') void refresh()
    }

    document.addEventListener('sui:navigate', onNavigate)
    host.closest('sui-panel-dock')?.addEventListener('sui:dock:activate', onDockActivate)

    /** Public refresh method. */
    Object.defineProperty(host, 'refresh', { value: () => void refresh(), configurable: true })

    void refresh()

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
      host.closest('sui-panel-dock')?.removeEventListener('sui:dock:activate', onDockActivate)
    }
  },
})
