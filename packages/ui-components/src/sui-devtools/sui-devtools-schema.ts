import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-schema.sui'

interface ArgDescriptor { name: string; kind: string; required?: boolean }
interface OpDescriptor { name: string; description: string; args: ArgDescriptor[]; returns: string; permission: string; sensitive?: boolean }
interface Reference { op: string; soft: boolean; known: boolean }

const SHIP_OP_RE = /ship\.(try\.)?([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)/gi

function collectRefs(src: string, ops: Map<string, OpDescriptor>): Reference[] {
  const out = new Map<string, Reference>()
  let m: RegExpExecArray | null
  SHIP_OP_RE.lastIndex = 0
  while ((m = SHIP_OP_RE.exec(src)) !== null) {
    const soft = !!m[1]
    const name = `${m[2]}.${m[3]}`
    const key = `${soft ? 'try:' : ''}${name}`
    if (!out.has(key)) out.set(key, { op: name, soft, known: ops.has(name) })
  }
  return [...out.values()].sort((a, b) => a.op.localeCompare(b.op))
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const toolbarEl = scope.toolbarEl() as HTMLElement
    const bodyEl = scope.bodyEl() as HTMLElement

    let ops = new Map<string, OpDescriptor>()
    let lastPath: string | null = null

    async function loadOps(): Promise<void> {
      if (ops.size > 0) return
      try {
        const resp = await fetch('/api/v1/ship/schema', { credentials: 'same-origin' })
        if (!resp.ok) return
        const body = await resp.json() as { ok: boolean; data: { ops: OpDescriptor[] } }
        ops = new Map(body.data.ops.map(op => [op.name, op]))
      } catch { /* best-effort */ }
    }

    function stripPrefix(path: string): string {
      const m = path.match(/^\/api\/v\/(.+)$/)
      return m ? m[1]! : path.replace(/^\/+/, '')
    }

    async function refresh(): Promise<void> {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const path = current?.path ?? location.pathname
      const stripped = stripPrefix(path)
      if (!stripped) { toolbarEl.innerHTML = '<span>No view</span>'; bodyEl.innerHTML = ''; return }
      if (stripped === lastPath) return
      lastPath = stripped
      await loadOps()
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: 'same-origin' })
        if (resp.status === 404) { bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="network"></sui-icon><span slot="title">No schema references</span>No ship.* calls detected in this view.</sui-empty-state>`; return }
        if (!resp.ok) { bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">fetch failed: ${resp.status}</div>`; return }
        const src = await resp.json() as { template: string; manifest: string | null }
        const refs = collectRefs(`${src.template}\n${src.manifest ?? ''}`, ops)
        if (refs.length === 0) {
          toolbarEl.innerHTML = ''
          bodyEl.innerHTML = `<sui-empty-state><sui-icon slot="icon" name="network"></sui-icon><span slot="title">No schema references</span>No <code>ship.*</code> calls detected in this view.</sui-empty-state>`
          return
        }
        const known = refs.filter(r => r.known).length
        toolbarEl.innerHTML = `<span>${refs.length} ops referenced</span><span style="flex:1"></span><span>${known} known · ${refs.length - known} unknown</span>`
        bodyEl.innerHTML = refs.map(r => {
          const desc = ops.get(r.op)
          const argsHtml = desc
            ? (desc.args.length > 0
              ? `<div>args: ${desc.args.map(a => `<code>${esc(a.name)}${a.required ? '' : '?'}:${esc(a.kind)}</code>`).join(' ')}</div>`
              : '<div>args: <em>none</em></div>') + `<div>returns: <code>${esc(desc.returns)}</code></div>${desc.sensitive ? '<div><strong>sensitive</strong></div>' : ''}`
            : `<div>Not registered in <code>/api/ship/schema</code>.</div>`
          return `<details><summary><span class="op ${r.soft ? 'soft' : ''} ${r.known ? '' : 'unknown'}">${esc(r.op)}</span><span class="desc">${esc(desc?.description ?? '(not in catalog)')}</span><span class="perm">${esc(desc?.permission ?? '')}</span></summary><div class="detail">${argsHtml}</div></details>`
        }).join('')
      } catch (e) {
        bodyEl.innerHTML = `<div style="padding:20px;color:var(--muted-foreground)">${esc((e as Error).message)}</div>`
      }
    }

    const onNavigate = () => { lastPath = null; void refresh() }
    const onDockActivate = (ev: Event) => {
      if ((ev as CustomEvent<{ tab: string }>).detail?.tab === 'schema') { lastPath = null; void refresh() }
    }

    document.addEventListener('sui:navigate', onNavigate)
    host.closest('sui-panel-dock')?.addEventListener('sui:dock:activate', onDockActivate)

    Object.defineProperty(host, 'refresh', { value: () => { lastPath = null; void refresh() }, configurable: true })

    void refresh()

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
      host.closest('sui-panel-dock')?.removeEventListener('sui:dock:activate', onDockActivate)
    }
  },
})
