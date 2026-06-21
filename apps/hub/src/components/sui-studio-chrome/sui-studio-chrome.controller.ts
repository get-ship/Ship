// <sui-studio-chrome> — chrome wrapper for the /studio/* subtree.
//
// Hub port of `apps/shell/src/components/sui-studio-chrome/sui-studio-chrome.controller.ts`.
// Body is byte-identical to the shell — the chrome is structural (a route
// slot + a bottom slot), and no business logic lives here. The `ready` /
// `dispose` events are dispatched so any descendants that want to wire up
// to chrome mount/unmount can; the hub has no such listeners today, but
// keeping the dispatch keeps the shell + hub surface byte-compatible for
// any future descendants (e.g. a rail that needs to recompute on chrome
// re-mount).
//
// The custom element is registered as `sui-studio-chrome` on first import.
// The route's host (`mountRouter` → `buildMountTarget`, see
// `packages/deck/route/src/mount-router.ts:170-182`) inserts a fresh
// `<sui-studio-chrome>` with a `<div slot="route">` child on every
// navigation; the runtime's `resolveSlots()` walks the chrome's template
// slots and fills `route` with the inner div at mount time. The LAYOUTS
// table emitted by the codegen (see `apps/hub/routes.gen.ts`) wires this
// tag to /studio/*.

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-chrome.sui'

export default controller<Scope>({
  scope: () => {
    // No reactive state — the chrome is structural.
    return {}
  },
  mount(_state, host) {
    // The mount-router's `buildMountTarget` chain creates the studio
    // chrome via `document.createElement('sui-studio-chrome')` and
    // appends it to the outer chrome (the HUB's root layout's
    // `<ship-app-chrome>`) as a light-DOM child. Without a `slot` attribute,
    // the outer chrome's named `<slot name="route">` does not catch it —
    // the studio chrome would be unslotted light-DOM content, invisible
    // inside the host's display (the prior symptom: every studio route
    // rendered chrome only, with a 0x0 content area). Marking the chrome
    // with `slot="route"` here re-runs the outer chrome's slot distribution
    // and projects this element into the route slot. Verified empirically:
    // setting `slot` in `connectedCallback` (which the SUI runtime invokes
    // before `mount`) triggers a slotchange redistribution. The shell
    // doesn't need this because its mount target is a regular `<main>`,
    // not a custom element with shadow DOM + a named slot.
    host.setAttribute('slot', 'route')
    host.dispatchEvent(
      new CustomEvent('sui-studio-chrome:ready', {
        bubbles: true,
        composed: true,
      }),
    )
    return () => {
      host.dispatchEvent(
        new CustomEvent('sui-studio-chrome:dispose', {
          bubbles: true,
          composed: true,
        }),
      )
    }
  },
})
