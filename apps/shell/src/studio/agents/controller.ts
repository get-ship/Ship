import { controller, signal, unwrap, live, derived } from '@ship/sui-runtime'
import type { LiveSignal, ViewScope } from '@ship/sui-runtime'
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

export default controller({
  scope: ({ ship, agents }: ViewScope<Scope>) => {
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
    const items = derived(() => (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[])
    const filteredItems = derived(() => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      const k = kind()
      const term = q()
      // Attach the tile hue here: renderable expressions can't call a
      // helper with args, so the card reads `a.hue` as a plain field.
      return all
        .filter((a) => matchesKind(a, k) && matchesQuery(a, term ?? ''))
        .map((a) => ({ ...a, hue: pickHue(a.id) }))
    })
    const activeId = derived(() => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      return all.find((a) => a.active)?.id ?? ''
    })
    const kindFilters = derived(() => {
      const all = (unwrap(agents as LiveSignal<AgentsList>)?.items ?? []) as AgentSummary[]
      const project = all.filter((a) => a.source === 'PROJECT AGENTS').length
      const user = all.filter((a) => a.source === 'USER AGENTS').length
      return [
        { id: 'all', label: 'All', count: all.length },
        { id: 'project', label: 'Project', count: project },
        { id: 'user', label: 'Library', count: user },
      ] satisfies KindFilter[]
    })
    return { q, kind, agentEvents, agentStatus, items, filteredItems, activeId, kindFilters }
  },
  on: {
    goCreate: (s) => s.navigate('/studio/agents/create'),
    onSearchInput: (s, e: Event) => {
      const target = e.target as HTMLInputElement
      s.q = target.value ?? ''
    },
    clearSearch: (s) => { s.q = '' },
    resetFilters: (s) => { s.q = ''; s.kind = 'all' },
  },
})
