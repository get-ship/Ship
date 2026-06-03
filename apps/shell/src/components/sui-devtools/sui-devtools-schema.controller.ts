import { controller, signal, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-devtools-schema.sui'
import { isShellLocalView } from '../../chrome/shell-surfaces'
import { ops as opsClient } from '../../ops'

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

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const refs = signal<DisplayRef[]>([])
    const errorMsg = signal<string | null>(null)
    const knownCount = derived(() => refs().filter(r => r.known).length)
    return { refs, errorMsg, knownCount }
  },

  mount(state, host) {
    let ops = new Map<string, OpDescriptor>()
    let lastPath: string | null = null

    async function loadOps(): Promise<void> {
      if (ops.size > 0) return
      try {
        // DiscoveryService.Schema (Connect) replaces GET /api/v1/ship/schema.
        const body = JSON.parse((await opsClient.discovery.schema({})).json) as { ops: OpDescriptor[] }
        ops = new Map(body.ops.map(op => [op.name, op]))
      } catch { /* best-effort */ }
    }

    function stripPrefix(path: string): string {
      const m = path.match(/^\/api\/v\/(.+)$/)
      // view-source is keyed by the base view name; subroutes like
      // /skills/cmdr share the parent view's source. Take just the
      // first segment to avoid spurious 404s on every detail navigation.
      const raw = m ? m[1]! : path.replace(/^\/+/, '')
      const firstSlash = raw.indexOf('/')
      return firstSlash > 0 ? raw.slice(0, firstSlash) : raw
    }

    async function refresh(): Promise<void> {
      const current = (window as unknown as { ship?: { current(): { path: string } } }).ship?.current()
      const path = current?.path ?? location.pathname
      const stripped = stripPrefix(path)
      if (!stripped) { state.refs = []; state.errorMsg = null; return }
      if (stripped === lastPath) return
      lastPath = stripped
      state.errorMsg = null
      // Shell-local surfaces (studio) carry no daemon view-source; skip the
      // fetch that would otherwise 404 on every visit.
      if (isShellLocalView(stripped)) { state.refs = []; return }
      await loadOps()
      try {
        const src = JSON.parse((await opsClient.discovery.source({ path: stripped })).json) as {
          template: string
          manifest: string | null
        }
        state.refs = collectRefs(`${src.template}\n${src.manifest ?? ''}`, ops)
      } catch (e) {
        // not_found → view has no daemon source; otherwise surface the error.
        const msg = (e as Error).message ?? ''
        if (/not.?found/i.test(msg)) { state.refs = [] }
        else state.errorMsg = msg
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
