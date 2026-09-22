<controller>
// <ship-studio-chrome> — @ship/studio · chrome wrapper for the /studio/*
// subtree. The chrome is structural (a route slot + a bottom slot); no
// business logic lives here. The `ready` / `dispose` events are dispatched
// so any descendant that wants to wire up to chrome mount/unmount can.
//
// The route's host (`mountRouter` → `buildMountTarget`, see
// `packages/deck/route/src/mount-router.ts:170-182`) inserts a fresh
// `<ship-studio-chrome>` with a `<div slot="route">` child on every
// navigation; the runtime's `resolveSlots()` walks the chrome's template
// slots and fills `route` with the inner div at mount time. The LAYOUTS
// table the host codegen emits wires this tag to /studio/*.

import { controller } from '@ship/sail-runtime'
import type { Scope } from './ship-studio-chrome.sail'

export default controller<Scope>({
  scope: () => {
    // No reactive state — the chrome is structural.
    return {}
  },
  mount(_state, host) {
    // The mount-router's `buildMountTarget` chain creates this chrome via
    // `document.createElement('ship-studio-chrome')` and appends it to the
    // outer chrome (a host's root layout, e.g. `<ship-app-chrome>`) as a
    // light-DOM child. Without a `slot` attribute, the outer chrome's named
    // `<slot name="route">` does not catch it — the studio chrome would be
    // unslotted light-DOM content, invisible inside the host's display (the
    // symptom: every studio route renders chrome only, with a 0x0 content
    // area). Marking the chrome with `slot="route"` here re-runs the outer
    // chrome's slot distribution and projects this element into the route
    // slot. Setting `slot` in `connectedCallback` (which the SUI runtime
    // invokes before `mount`) triggers a slotchange redistribution.
    host.setAttribute('slot', 'route')
    host.dispatchEvent(
      new CustomEvent('ship-studio-chrome:ready', {
        bubbles: true,
        composed: true,
      }),
    )
    return () => {
      host.dispatchEvent(
        new CustomEvent('ship-studio-chrome:dispose', {
          bubbles: true,
          composed: true,
        }),
      )
    }
  },
})
</controller>

<!--
  <ship-studio-chrome> — @ship/studio · chrome wrapper for the /studio/*
  subtree. A host's studio layout (`routes/studio/_layout.sui`) names this
  tag as its wrapper component, so the LAYOUTS table binds
  `{ pathPrefix: '/studio', tag: 'ship-studio-chrome' }`.

  Slots:
    route       the route content (filled by the host at navigation time)
    bottom      status bar / footer (a host that mounts its own bottom
                chrome layer — e.g. <ship-app-chrome> — leaves this empty)

  Layout: flex column; `flex: 1; min-height: 0` so it fills the height its
  parent grants via flex, without growing beyond it. The route slot's .main
  takes the remaining height; the footer (flex-shrink: 0) sits below it.
-->

<!-- style block removed: use sx / external CSS -->

<main>
  <slot name="route"></slot>
</main>
<footer>
  <slot name="bottom"></slot>
</footer>
