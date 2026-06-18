// /studio/components/[name] — placeholder controller for the hub's
// component viewer route. The [name] segment is read off the route
// signals (set up by `defineRoutes` from the `routes/` walk) and
// returned on `Scope` so the page.sui template can render it. The
// full viewer (catalog-driven chrome, source panel, prop knobs) is
// still upstream in `apps/shell/routes/studio/components/[name]/` —
// this hub scaffold is the minimum that builds.

import { controller, derived, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ route }: ViewScope<Scope>) => {
    const name = derived(() => route.params.name?.() ?? '')
    return { name }
  },
})
