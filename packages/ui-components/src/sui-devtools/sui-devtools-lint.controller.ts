import { controller, signal, derived } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-lint.sui'

interface Finding {
  file: string
  line: number
  col: number
  rule: string
  severity: 'error' | 'warn' | string
  message: string
}

export default controller<Scope>({
  scope: () => {
    const findings = signal<Finding[]>([])
    const filter = signal<'all' | 'error' | 'warn'>('all')
    const viewName = signal<string | null>(null)
    const loading = signal(false)
    const errorMsg = signal<string | null>(null)

    const errCount = derived(() => findings().filter(f => f.severity === 'error').length)
    const warnCount = derived(() => findings().filter(f => f.severity === 'warn').length)
    const filteredFindings = derived(() => {
      const f = filter()
      const sorted = [...findings()].sort((a, b) => {
        const sv = (s: string) => s === 'error' ? 0 : s === 'warn' ? 1 : 2
        const d = sv(a.severity) - sv(b.severity)
        if (d !== 0) return d
        if (a.file !== b.file) return a.file.localeCompare(b.file)
        return a.line !== b.line ? a.line - b.line : a.col - b.col
      })
      if (f === 'all') return sorted
      return sorted.filter(x => x.severity === f)
    })

    return { findings, filter, viewName, loading, errorMsg, errCount, warnCount, filteredFindings }
  },

  mount(state, host) {
    const s = state as Record<string, unknown>

    function topLevelName(): string | null {
      const p = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current().path ?? location.pathname
      const m = p.match(/^\/api\/v\/([^/]+)/)
      if (m) return m[1]!
      const seg = p.replace(/^\/+/, '').split('/')[0]
      return seg || null
    }

    async function refresh(): Promise<void> {
      const name = topLevelName()
      s['viewName'] = name
      if (!name) { s['findings'] = []; s['errorMsg'] = null; return }
      s['loading'] = true
      s['errorMsg'] = null
      try {
        const resp = await fetch('/api/v1/lint/view', {
          method: 'POST',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name }),
        })
        if (resp.status === 404) { s['findings'] = []; return }
        if (!resp.ok) { s['errorMsg'] = `lint failed: ${resp.status}`; return }
        const body = await resp.json() as { ok: boolean; data?: { findings: Finding[] }; error?: string }
        s['findings'] = body.data?.findings ?? []
      } catch (e) {
        s['errorMsg'] = (e as Error).message
      } finally {
        s['loading'] = false
      }
    }

    const onNavigate = () => void refresh()
    const onDockActivate = (ev: Event) => {
      if ((ev as CustomEvent<{ tab: string }>).detail?.tab === 'lint') void refresh()
    }

    document.addEventListener('sui:navigate', onNavigate)
    host.closest('sui-panel-dock')?.addEventListener('sui:dock:activate', onDockActivate)

    Object.defineProperty(host, 'refresh', { value: () => void refresh(), configurable: true })

    void refresh()

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
      host.closest('sui-panel-dock')?.removeEventListener('sui:dock:activate', onDockActivate)
    }
  },
})
