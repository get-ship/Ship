import { controller, signal, derived } from '@ship/ui-runtime'
import type { Scope } from './sui-devtools-schema.sui'

interface ArgDescriptor { name: string; kind: string; required?: boolean }
interface OpDescriptor { name: string; description: string; args: ArgDescriptor[]; returns: string; permission: string; sensitive?: boolean }

interface DisplayRef {
  op: string
  soft: boolean
  known: boolean
  description: string
  permission: string
  args: ArgDescriptor[] | null
  argsSummary: string
  returns: string | null
  sensitive: boolean
}

const SHIP_OP_RE = /ship\.(try\.)?([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)/gi

function collectRefs(src: string, ops: Map<string, OpDescriptor>): DisplayRef[] {
  const seen = new Map<string, DisplayRef>()
  let m: RegExpExecArray | null
  SHIP_OP_RE.lastIndex = 0
  while ((m = SHIP_OP_RE.exec(src)) !== null) {
    const soft = !!m[1]
    const name = `${m[2]}.${m[3]}`
    const key = `${soft ? 'try:' : ''}${name}`
    if (seen.has(key)) continue
    const desc = ops.get(name)
    seen.set(key, {
      op: name,
      soft,
      known: ops.has(name),
      description: desc?.description ?? '(not in catalog)',
      permission: desc?.permission ?? '',
      args: desc ? desc.args : null,
      argsSummary: desc
        ? desc.args.map(a => `${a.name}${a.required ? '' : '?'}:${a.kind}`).join(' ')
        : '',
      returns: desc?.returns ?? null,
      sensitive: desc?.sensitive ?? false,
    })
  }
  return [...seen.values()].sort((a, b) => a.op.localeCompare(b.op))
}

export default controller<Scope>({
  scope: () => {
    const refs = signal<DisplayRef[]>([])
    const errorMsg = signal<string | null>(null)
    const knownCount = derived(() => refs().filter(r => r.known).length)
    return { refs, errorMsg, knownCount }
  },

  mount(state, host) {
    const s = state as Record<string, unknown>
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
      if (!stripped) { s['refs'] = []; s['errorMsg'] = null; return }
      if (stripped === lastPath) return
      lastPath = stripped
      s['errorMsg'] = null
      await loadOps()
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: 'same-origin' })
        if (resp.status === 404) { s['refs'] = []; return }
        if (!resp.ok) { s['errorMsg'] = `fetch failed: ${resp.status}`; return }
        const src = await resp.json() as { template: string; manifest: string | null }
        s['refs'] = collectRefs(`${src.template}\n${src.manifest ?? ''}`, ops)
      } catch (e) {
        s['errorMsg'] = (e as Error).message
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
