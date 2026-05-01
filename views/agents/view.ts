// Agents list view — controller for `view.sui`.
//
// Server data comes from the template's `<data name="agents" live />`
// declaration. The controller derives `items` and `total` from it and
// owns the navigation handler used by the "New agent" buttons.

import { controller, unwrap } from '@ship/ui-runtime'
import type { LiveSignal } from '@ship/ui-runtime'
import type { AgentsSummariesResponse } from '@ship/ui-sdk-client'

/** Template scope — what `view.sui` declares + framework injects. */
type Scope = { agents: LiveSignal<AgentsSummariesResponse> }

export default controller<Scope>({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
})
