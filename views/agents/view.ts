// Agents list view — controller for `view.sui`.
//
// Server data comes from the template's `<data name="agents" live />`
// declaration. The controller derives `items` and `total` from it and
// owns the navigation handler used by the "New agent" buttons.

import { controller, unwrap } from '@ship/ui-runtime'
import type { Scope } from './view.sui'

export default controller<Scope>({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
  },
  on: {
    newAgent: ({ navigate }) => navigate('/agents/create'),
  },
})
