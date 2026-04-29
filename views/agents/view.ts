// Agents list view — pulls live `ship.agents.summaries()` from the daemon
// and exposes the items + total to the template.
//
// Pure declarative: no local state, no callbacks beyond the navigate
// passed in by the host. The whole grid (loading boundary, empty state,
// cards) walks off the `agents` LiveSignal.

import { derived, live, view, type LiveSignal } from '@ship-set/runtime'
import type { ShipSDK, AgentsSummariesResponse, AgentSummary } from '@ship-set/sdk-client'

interface Bindings {
  agents: LiveSignal<AgentsSummariesResponse>
  items: () => AgentSummary[]
  total: () => number
  navigate: (path: string) => void
}

export default view<Bindings>((ctx) => {
  const ship = ctx.ship as ShipSDK
  const agents = live(() => ship.agents.summaries())
  const items = derived<AgentSummary[]>(() => {
    const s = agents()
    return s.kind === 'ok' ? s.value.items : []
  })
  const total = derived<number>(() => items().length)
  return {
    agents,
    items,
    total,
    navigate: ctx.navigate,
  }
})
