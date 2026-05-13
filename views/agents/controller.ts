import { controller, unwrap, live, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type AgentStatus = 'running' | 'completed' | 'errored'

interface EventRow { event_type: string; actor: string; timestamp: string }
interface EventsEnvelope { groups: Array<{ rows: EventRow[] }> }

export default controller<Scope>({
  scope: ({ ship }) => {
    // Subscribe to the live kernel-event stream filtered to `agent.*`.
    // `events.list` returns groups keyed by entity; we flatten + sort
    // chronologically so the most recent verdict per actor wins.
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
    return { agentEvents, agentStatus }
  },
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
})
