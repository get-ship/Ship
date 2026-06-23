import { controller } from '@ship/sui-runtime'
import type { Scope } from './ship-view-chrome.sui'

// <ship-view-chrome> controller — @ship/studio.
//
// The chrome is fully self-rendered: the template draws the topbar/statusbar
// with named slots, and the props (title, branch, ahead, behind, tone, busy)
// bind directly to the inline elements. There is no host-owned container to
// forward to and no `chrome.*` API — a view just places <ship-view-chrome>
// in its template and fills its slots.
//
// This sidecar is a lifecycle anchor for future work (e.g. async branch
// resolution, telemetry). The mount hook is a no-op: the template's
// `{title()}` / `{branch()}` / etc. bindings already subscribe to the prop
// signals, and the template draws the chrome inline on first paint.
export default controller<Scope>({
  mount(_state, _host) {
    return () => {
      // No subscriptions to stop, no moved nodes to withdraw.
    }
  },
})
