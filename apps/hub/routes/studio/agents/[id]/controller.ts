// /studio/agents/[id] — the hub's single-agent detail viewer.
//
// The `[id]` segment is read off the route signals (set up by
// `defineRoutes` from the `routes/` walk) and returned on `Scope` so
// the page.sui template can pass it to the <ship-agent-detail> as
// the `id` prop. The composite owns the data story
// (ship.agents.detail({ id }) via live() inside the controller) and
// renders the agent's identity header, description, tags,
// capabilities, and source path; the route controller just routes
// the path param to the prop.

import { controller, derived, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<ViewScope<Scope>>({
  scope: ({ route }: ViewScope<Scope>) => {
    const id = derived(() => route.params.id?.() ?? '')
    return { id }
  },
})
