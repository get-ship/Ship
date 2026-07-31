<controller>
import { controller } from '@ship/sail-runtime'
import type { Scope } from './ship-view-chrome.sail'

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
</controller>

<!--
  <ship-view-chrome> — @ship/studio · a view's declarative contribution to
  the app's topbar + statusbar.

  Views never render their own breadcrumb or status bar — they emit a single
  <ship-view-chrome> and the chrome paints the bar on their behalf. Keeps
  first-party and third-party views symmetric.

  The chrome is a complete widget, not a forwarder: it renders its own
  topbar/statusbar pair inline, with named shadow slots for the subregions
  a view wants to fill. Attribute changes update the inline fields directly.

  Props (all reflect):
    title  (string, '')        breadcrumb leaf
    branch (string, '')        status branch name
    ahead  (number, 0)         ↑N commits
    behind (number, 0)         ↓N commits
    tone   (union, 'default')  status dot tint
    busy   (bool, false)       pulse dot
  Slots (shadow DOM, named):
    topbar-start  — left of the breadcrumb leaf
    actions       — right of the breadcrumb leaf (right-aligned cluster)
    status-start  — left of the status bar
    status-end    — right of the status bar
-->

<!-- <prop name="title" type="string" default="" reflect /> -->
<!-- <prop name="branch" type="string" default="" reflect /> -->
<!-- <prop name="ahead" type="number" default="0" reflect /> -->
<!-- <prop name="behind" type="number" default="0" reflect /> -->
<!-- <prop name="tone" values="default|success|warning|destructive|accent|muted" default="default" reflect /> -->
<!-- <prop name="busy" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<header data-test="view-chrome-topbar">
  <span>
    <slot name="topbar-start"></slot>
  </span>
  <span data-test="view-chrome-title">{title()}</span>
  <span></span>
  <span>
    <slot name="actions"></slot>
  </span>
</header>

<div data-test="view-chrome-statusbar">
  <span>
    <slot name="status-start"></slot>
  </span>
  <span></span>
  <if cond={branch()}>
    <span data-test="view-chrome-branch">{branch()}</span>
  </if>
  <if cond={ahead() > 0 || behind() > 0}>
    <span data-test="view-chrome-ab">
      <if cond={ahead() > 0}><span>↑{ahead()}</span></if>
      <if cond={behind() > 0}><span>↓{behind()}</span></if>
    </span>
  </if>
  <if cond={tone() !== 'default' || busy()}>
    <span
      data-test="view-chrome-dot"
      data-tone={tone() !== 'default' ? tone() : null}
      data-busy={busy()}
      aria-hidden="true"
    ></span>
  </if>
  <span>
    <slot name="status-end"></slot>
  </span>
</div>
