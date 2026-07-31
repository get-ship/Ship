<controller>
// Hub version of `apps/shell/routes/studio/agents/controller.ts`.
//
// Client seam: the shell injected `ship` + the `agents` LiveSignal through
// the view mount ctx; the hub pulls `ship` from the AMBIENT
// `ctx.ship` injected by the host's `entry.gen.ts` (deck-runtime
// maps `clients.<name>` into view scope). The relative `src/lib/client`
// import is gone — the client lives in the view mount context, not a
// per-view module path.
//
// The list reads `ship.agents.summaries()` — agents.summaries has a
// registered live mux resolver (apps/shipd/src/ship_runtime/live_resolvers.rs),
// so `ship` (the live SSE client) streams `{ items }`, and the page's
// `` consumes that LiveSignal. Run-status (events.list)
// has no resolver yet — it degrades (see the `agentEvents` note below).
import { controller, signal, unwrap, live, derived } from '@ship/sail-runtime'
import type { LiveSignal, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
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
    const ship = ctx.ship
    // `ops` = the unary Connect client (host-owned, injected as `clients.ops`).
    // Run-status reads it; the live `ship` reader is only for ops with a mux
    // resolver (agents.summaries).
    const ops = ctx.ops
    // The agent list is LIVE: agents.summaries has a registered mux resolver
    // (apps/shipd/src/ship_runtime/live_resolvers.rs), so `ship` (the SSE
    // client) streams `{ items }`. The page's `` consumes this
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
</controller>

<data name="agents" source="ship.agents.summaries()" live />

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
<div>
  <div>

    <header>
      <div>
        <h1>Agents</h1>
        <p>
          <strong>{items().length}</strong> agent{items().length === 1 ? '' : 's'}
          <span aria-hidden="true"></span>
          <strong>{kindFilters()[1].count}</strong> project
          <span aria-hidden="true"></span>
          <strong>{kindFilters()[2].count}</strong> library
          <span aria-hidden="true"></span>
          <code>.ship/agents/</code>
          <if cond={activeId()}>
            <span aria-hidden="true"></span>
            <span data-test="active-summary">active: {activeId()}</span>
          </if>
        </p>
        <div aria-hidden="true"></div>
      </div>
      <button tone="primary" variant="solid" on:click={goCreate} data-test="cta-new">
        <icon slot="leading" name="plus"></icon>
        New agent
      </button>
    </header>

    <div>
      <div>
        <icon name="search" size="sm" tone="muted"></icon>
        <input type="text" placeholder="Search by name, description, or tag…"
          aria-label="Search agents"
          spellcheck="false"
          data-test="search"
          value={q()}
          on:input={onSearchInput} />
        <if cond={q()}>
          <button type="button" aria-label="Clear search"
            on:click={clearSearch}>
            <icon name="x" size="xs"></icon>
          </button>
        </if>
      </div>
      <toggle-group variant="segmented" type="single" value={kind()} size="sm" aria-label="Filter by source"
        on:sail-change={(e) => kind.set(e.detail.value)}>
        <for as="f" of={kindFilters()} key={f.id}>
          <toggle value={f.id} data-test="filter" data-filter={f.id}>
            {f.label}
          </toggle>
        </for>
      </toggle-group>
    </div>

    
      <while-loading of="agents">
      <div data-test="loading" role="status" aria-live="polite" aria-busy="true" aria-label="Loading agents">
        <for as="n" of={[0,1,2,3,4,5]} key={n}>
          <div aria-hidden="true">
            <spinner shape="block" size="md"></spinner>
            <div>
              <spinner shape="line" size="md" style="width:70%"></spinner>
              <spinner shape="line" size="sm" style="width:95%"></spinner>
              <spinner shape="line" size="sm" style="width:55%"></spinner>
            </div>
          </div>
        </for>
      </div>
    
      </while-loading><while-error as="err" of="agents">
      <error-state title="Couldn't load agents" data-test="error" role="alert" aria-live="assertive">
        {err.message}
        <button slot="action" tone="primary" variant="solid" on:click={() => window.location.reload()}>Retry</button>
      </error-state>
    
      </while-error><if cond={agents().kind === 'ok'}>
      <if cond={items().length === 0}>
        <empty-state icon="sparkles" variant="default" size="sm" data-test="empty" role="status">
          <span slot="title">No agents yet</span>
          Drop a JSONC file under
          <code>.ship/agents/</code>
          or create one from the Studio.
          <button slot="action" tone="primary" variant="solid" on:click={goCreate} data-test="cta-create-first">
            <icon slot="leading" name="plus"></icon>
            Create your first agent
          </button>
        </empty-state>
      <else-if cond={filteredItems().length === 0}>
        <empty-state icon="search-x" variant="default" size="sm" data-test="empty-filtered" role="status">
          <span slot="title">No matching agents</span>
          {q() ? 'Nothing matches "' + q() + '"' : 'No agents in this scope.'}
          <button slot="action" variant="outline" size="sm" on:click={resetFilters}>
            Clear filters
          </button>
        </empty-state>
      </else-if><else>
        <div data-view-enter data-test="grid">
          <for as="a" of={filteredItems()} key={a.id}>
            <a href={'/studio/agents/' + a.id}
                style="background:var(--card);border-color:var(--border-subtle);box-shadow:var(--shadow-card-rest);display:flex;flex-direction:column;min-height:100%;cursor:pointer;text-decoration:none;color:inherit"
                data-test="agent"
                data-id={a.id}
                data-active={a.active}>
              <div aria-hidden="true" data-test="hue-tile">
                <span>{a.letter}</span>
              </div>
              <div>
                <div>
                  <div>{a.name}</div>
                  <p>{a.description || 'No description'}</p>
                </div>
                <div>
                  <span>{a.skills_count} skill{a.skills_count === 1 ? '' : 's'}</span>
                  <span>&middot;</span>
                  <span>{a.mcp_count} MCP</span>
                  <span>&middot;</span>
                  <span>{a.rules_count} rule{a.rules_count === 1 ? '' : 's'}</span>
                </div>
                <div>
                  <badge tone="muted" variant="outline" size="sm">{a.source}</badge>
                  <if cond={a.active}>
                    <chip tone="success" variant="soft" size="sm" data-test="active-chip">
                      <status-dot slot="leading" tone="success" size="sm" pulse label="Active"></status-dot>
                      active
                    </chip>
                  </if>
                  <if cond={agentStatus()[a.id]}>
                    <chip
                        tone={agentStatus()[a.id] === 'errored' ? 'destructive' : agentStatus()[a.id] === 'completed' ? 'success' : 'primary'}
                        variant="soft"
                        size="sm"
                        data-test="live-status"
                        data-status={agentStatus()[a.id]}>
                      <status-dot
                          slot="leading"
                          tone={agentStatus()[a.id] === 'errored' ? 'destructive' : agentStatus()[a.id] === 'completed' ? 'success' : 'primary'}
                          size="sm"
                          pulse={agentStatus()[a.id] === 'running'}
                          label={agentStatus()[a.id]}></status-dot>
                      {agentStatus()[a.id]}
                    </chip>
                  </if>
                  <for as="t" of={(a.tags || []).slice(0, 2)} key={t}>
                    <chip tone="muted" variant="soft" size="sm">{t}</chip>
                  </for>
                  <if cond={(a.tags || []).length > 2}>
                    <chip tone="muted" variant="outline" size="sm">+{(a.tags || []).length - 2}</chip>
                  </if>
                </div>
              </div>
            </a>
          </for>
        </div>
      </else></if>
    
    </if>

  </div>
</div>
  </main>
</div>
