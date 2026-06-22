import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-view-chrome.sui'

// <sui-view-chrome> hub controller.
//
// The hub's chrome is fully self-rendered: the template draws the
// topbar/statusbar with named slots, and the props (title, branch,
// ahead, behind, tone, busy) bind directly to the inline elements.
// There is no shell-owned container to forward to, and no
// `chrome.*` API on the hub — views just place <sui-view-chrome>
// in their template and fill its slots.
//
// The shell port forwards to `#view-topbar-actions` /
// `#view-status-start` / `#view-status-end` and writes
// `#breadcrumbs-title` / `#status-branch` / `#status-ab` /
// `#status-dot` via document.querySelector. The hub version has
// none of that — the equivalent fields ARE the inline elements
// in the template, bound by the compiler.
//
// This sidecar is kept for parity with the shell's chrome-controller
// surface and as a lifecycle anchor for future work (e.g. async
// branch resolution, telemetry). The mount hook is a no-op: the
// template's `{title()}` / `{branch()}` / etc. bindings already
// subscribe to the prop signals, and the template draws the chrome
// inline on first paint. No `chrome.*` calls; no DOM lookups.
export default controller<Scope>({
  mount(_state, _host) {
    return () => {
      // No subscriptions to stop, no moved nodes to withdraw.
    }
  },
})
