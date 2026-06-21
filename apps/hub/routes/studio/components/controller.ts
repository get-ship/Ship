// /studio/components — the hub's component-catalog index.
//
// The route is a thin composition of the <ship-component-catalog>
// composite in `@ship/studio`. The composite owns the data story
// (BACKEND-GAP today) and the empty-state; the route's controller
// is empty (the page.sui is a single-tag composition).
//
// `controller()` still requires a scope — the empty scope is the
// signal that the page is a pure composition, no view-host state.

import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<ViewScope<Scope>>({
  scope: () => ({}),
})
