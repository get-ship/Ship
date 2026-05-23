import { controller, signal, unwrap, live, derived } from '@ship/sui-runtime'
import type { LiveSignal, MutableState } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
// Reuse the agent-detail hue derivation so a card's tile and the detail
// page it links to land on the same colour — one source of truth, no
// parallel palette. (Reading a sibling route's helper, not editing it.)
import { pickHue } from './[id]/hue'
import { matchesKind, matchesQuery } from './filter'

type AgentStatus = 'running' | 'completed' | 'errored'

interface AgentSummary {
  id: string
  name: string
  description: string
  source: string
  active: boolean
  tags?: string[]
}
interface EventRow { event_type: string; actor: string; timestamp: string }
interface EventsEnvelope { groups: Array<{ rows: EventRow[] }> }
interface KindFilter { id: 'all' | 'project' | 'user'; label: string; count: number }
interface AgentsList { items: AgentSummary[] }

// `on:`/`mount` handlers receive the writable state proxy. TS can't
// contextually infer it (OnSection is a union of two function shapes),
// so annotate explicitly with `MutableState<Scope>` — widening it would
// break assignability to the OnSection overload.
type S = MutableState<Scope>
// `navigate` is injected onto the proxy by the runtime mount context but
// is not a template-declared scope key, so it isn't on `MutableState`.
type Nav = { navigate: (path: string) => void }

export default controller<Scope>({
  scope: ({ ship }) => {
    const q = signal('')
    const kind = signal<KindFilter['id']>('all')
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const agentEvents = live(() => (ship as any).events.list({ event_type: 'agent.', limit: 50 }))
    const agentStatus = derived<Record<string, AgentStatus>>(() => {
      const env = agentEvents()
      if (env.kind !== 'ok') return {}
      const v = env.value as EventsEnvelope
      const rows: EventRow[] = (v?.groups ?? []).flatMap((g) => g.rows ?? [])
      rows.sort((a, b) => a.timestamp.localeCompare(b.timestamp))
      const map: Record<string, AgentStatus> = {}
      for (const r of rows) {
        if (!r.actor) continue
        if (r.event_type === 'agent.started') map[r.actor] = 'running'
        else if (r.event_type === 'agent.completed') map[r.actor] = 'completed'
        else if (
          r.event_type === 'agent.error' ||
          r.event_type === 'agent.errored' ||
          r.event_type === 'agent.failed'
        ) map[r.actor] = 'errored'
      }
      return map
    })
    return { q, kind, agentEvents, agentStatus }
  },
  derived: {
    items: ({ agents }) => (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[],
    filteredItems: ({ agents, q, kind }) => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      // Derived scope-args arrive as signal getters, not unwrapped values.
      // Call them before using; cast assertions don't unwrap at runtime.
      const k = (typeof kind === 'function' ? kind() : kind) as KindFilter['id']
      const term = (typeof q === 'function' ? q() : q) as string
      // Attach the tile hue here: renderable expressions can't call a
      // helper with args, so the card reads `a.hue` as a plain field.
      return all
        .filter((a) => matchesKind(a, k) && matchesQuery(a, term ?? ''))
        .map((a) => ({ ...a, hue: pickHue(a.id) }))
    },
    activeId: ({ agents }) => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      return all.find((a) => a.active)?.id ?? ''
    },
    kindFilters: ({ agents }) => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      const project = all.filter((a) => a.source === 'PROJECT AGENTS').length
      const user = all.filter((a) => a.source === 'USER AGENTS').length
      return [
        { id: 'all', label: 'All', count: all.length },
        { id: 'project', label: 'Project', count: project },
        { id: 'user', label: 'Library', count: user },
      ] satisfies KindFilter[]
    },
  },
  on: {
    goCreate: (s: S) => (s as unknown as Nav).navigate('/studio/agents/create'),
    onSearchInput: (s: S, e: Event) => {
      const target = e.target as HTMLInputElement
      s.q = target.value ?? ''
    },
    clearSearch: (s: S) => { s.q = '' },
    // `kind` emits as `unknown` (emitter gap), so it isn't a MutableState
    // key — assign through a narrowed view.
    resetFilters: (s: S) => { s.q = ''; (s as unknown as { kind: KindFilter['id'] }).kind = 'all' },
  },
})
