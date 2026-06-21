// /studio/skills — the hub's skill-list index.
//
// The route is a thin composition of the <ship-skill-list> composite
// in `@ship/skills`. The composite owns the data story
// (`ship.skills.list()` via Connect SDK) and the empty / loading /
// error states; the route's controller is empty (the page.sui is a
// single-tag composition).
//
// `controller()` still requires a scope — the empty scope is the
// signal that the page is a pure composition, no view-host state.

import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<ViewScope<Scope>>({
  scope: () => ({}),
})
