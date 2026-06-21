// <ship-app-chrome> — controller.
//
// The wrapper has no state of its own: the nav bar lives on
// <ship-nav-top> (and the route content is projected by the runtime
// into the <slot name="route">). The empty `scope` is the only reason
// this file exists — the compiler generates the custom element from
// a `.sui` + sibling `.controller.ts` pair, and template-only chrome
// without a controller would not register.

import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-app-chrome.sui'

export default controller<ViewScope<Scope>>({
  scope: () => ({}),
})
