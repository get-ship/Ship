// /studio/components/[name] — the hub's single-component viewer.
//
// The `[name]` segment is read off the route signals (set up by
// `defineRoutes` from the `routes/` walk) and returned on `Scope` so
// the page.sui template can pass it to the <ship-component-viewer> as
// the `tag` prop. The composite owns the data story (BACKEND-GAP;
// the empty-state explains the gap + flags the operator-visible
// `.artifacts/hub-buildout-flags.md` entry); the route controller
// just routes the path param to the prop.

import { controller, derived, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<ViewScope<Scope>>({
  scope: ({ route }: ViewScope<Scope>) => {
    const name = derived(() => route.params.name?.() ?? '')
    return { name }
  },
})
