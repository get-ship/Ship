import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-agent-card.sui'

// `ship.agents.detail()` is typed `LiveSource<unknown>` in the SDK
// surface (no exported row type yet), so narrow locally to the fields
// this card renders. Mirrors the daemon `AgentDetail` shape.
interface AgentDetail {
  id: string
  name: string
  description: string
  source: string
  tags: string[]
}

const EMPTY: AgentDetail = { id: '', name: '', description: '', source: '', tags: [] }

export default controller({
  scope: ({ id }: ViewScope<Scope>) => {
    const agent = live(() => ship.agents.detail({ id: id() }))
    const data = derived<AgentDetail>(() => ({ ...EMPTY, ...(unwrapOr(agent, EMPTY) as Partial<AgentDetail>) }))
    const name = derived(() => data().name || data().id)
    const description = derived(() => data().description)
    const source = derived(() => data().source)
    const tags = derived<string[]>(() => (Array.isArray(data().tags) ? data().tags : []))
    return { agent, data, name, description, source, tags }
  },
})
