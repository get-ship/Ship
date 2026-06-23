// Hub version of `apps/shell/routes/studio/agents/controller.ts`.
//
// Client seam: the shell injected `ship` + the `agents` LiveSignal through
// the view mount ctx; the hub pulls `ship` from the AMBIENT
// `ctx.clients.ship` injected by the host's `entry.gen.ts` (deck-runtime
// maps `clients.<name>` into view scope). The relative `src/lib/client`
// import is gone — the client lives in the view mount context, not a
// per-view module path.
//
// The list reads `ship.agents.summaries()` — agents.summaries has a
// registered live mux resolver (apps/shipd/src/ship_runtime/live_resolvers.rs),
// so `ship` (the live SSE client) streams `{ items }`, and the page's
// `{#async agents}` consumes that LiveSignal. Run-status (events.list)
// has no resolver yet — it degrades (see the `agentEvents` note below).
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
interface EventRow { event_type: string; actor: string; created_at: string }
interface EventsEnvelope { events: EventRow[] }
interface KindFilter { id: 'all' | 'project' | 'user'; label: string; count: number }
interface AgentsList { items: AgentSummary[] }

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const q = signal('')
    const kind = signal<KindFilter['id']>('all')
    // AMBIENT client (host-owned): the entry.gen.ts bootstrap constructs
    // `ship` from `src/lib/client.ts` and injects it as `clients.ship`.
    // `ship` is the LIVE SSE client — only ops with a registered mux
    // resolver (`agents.summaries`, `collections.list`, `git.overview`,
    // `account.state`) subscribe over SSE; other ops would 404.
    const ship = ctx.clients.ship
    // `ops` = the unary Connect client (host-owned, injected as `clients.ops`).
    // Run-status reads it; the live `ship` reader is only for ops with a mux
    // resolver (agents.summaries).
    const ops = ctx.clients.ops
    // The agent list is LIVE: agents.summaries has a registered mux resolver
    // (apps/shipd/src/ship_runtime/live_resolvers.rs), so `ship` (the SSE
    // client) streams `{ items }`. The page's `{#async agents}` consumes this
    // LiveSignal for its loading/error/ok states; items / filteredItems /
    const agents = live(() => ship.agents.summaries()) as LiveSignal<AgentsList>
    // Run-status feed: unary EventsService.Poll (no live resolver, so the dead
    // `ship.events.list` 404'd on subscribe). `Poll` returns a JsonResponse
    // { json } wrapping `{ events: EventEnvelope[] }`; `namespace: 'agent'`
    // server-side filters to event_type `agent.*`. The JSON (snake_case
    // EventEnvelope fields) is decoded in agentStatus below.
    const agentEvents = live(() => ops.events.poll({ namespace: 'agent', limit: 50n }))
    const agentStatus = derived<Record<string, AgentStatus>>(() => {
      const st = agentEvents()
      if (st.kind !== 'ok') return {}
      let rows: EventRow[] = []
      try {
        const env = JSON.parse((st.value as { json?: string }).json ?? '{}') as EventsEnvelope
        rows = Array.isArray(env?.events) ? env.events : []
      } catch {
        return {}
      }
      rows.sort((a, b) => (a.created_at ?? '').localeCompare(b.created_at ?? ''))
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
      ] as KindFilter[]
    })
    return { q, kind, agents, agentEvents, agentStatus, items, filteredItems, activeId, kindFilters }
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
