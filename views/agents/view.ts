// Agents list view — controller for `view.html.set`.
//
// Server data comes from the template's `<data name="agents" live />`
// declaration. The controller derives `items` and `total` from it and
// owns the navigation handler used by the "New agent" buttons.

import { controller, unwrap } from '@ship-set/runtime'
import type { LiveSignal } from '@ship-set/runtime'
import type { AgentsSummariesResponse } from '@ship-set/sdk-client'

/** Template scope — what `view.html.set` declares + framework injects. */
type Scope = { agents: LiveSignal<AgentsSummariesResponse> }

export default controller<Scope>({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ items }) => (items as () => unknown[])().length,
  },
  on: {
    newAgent: ({ navigate }: { navigate: (path: string) => void }) =>
      navigate('/agents/create'),
  },
})
