import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { AgentDetail } from '@ship/sdk-client'
import type { Scope } from './ship-agent-card.sui'

// Use a Pick over the proto-generated AgentDetail so new proto fields are
// immediately visible without a local edit, while keeping the component
// narrowed to only the five fields it actually renders.
type AgentDetailFields = Pick<AgentDetail, 'id' | 'name' | 'description' | 'source' | 'tags'>

const EMPTY: AgentDetailFields = { id: '', name: '', description: '', source: '', tags: [] }

export default controller({
  scope: ({ id }: ViewScope<Scope>) => {
    const agent = live(() => ship.agents.detail({ id: id() }))
    const data = derived<AgentDetailFields>(() => ({ ...EMPTY, ...(unwrapOr(agent, EMPTY) as Partial<AgentDetailFields>) }))
    const name = derived(() => data().name || data().id)
    const description = derived(() => data().description)
    const source = derived(() => data().source)
    const tags = derived<string[]>(() => (Array.isArray(data().tags) ? data().tags : []))
    return { agent, data, name, description, source, tags }
  },
})
