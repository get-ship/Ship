// /studio/agents — the hub's agents list.
//
// The pack's <ship-agent-list> owns the data story
// (ship.agents.summaries() via live() inside the controller) and
// the search/filter state. The route controller is empty — the
// page is a pure composition: chrome is the root layout, the page
// body is the pack composite, the controller just returns the empty
// scope.

import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<ViewScope<Scope>>({
  scope: () => ({}),
})
