// <sui-studio-chrome> — chrome wrapper for the /studio/* subtree.
//
// The custom element is registered as `sui-studio-chrome` on first
// import. The route's host inserts a <sui-studio-chrome> with a
// <div slot="route"> child; the runtime's resolveSlots() walks the
// chrome's template slots and fills `route` with the inner div at
// mount time. The LAYOUTS table emitted by the codegen (see
// apps/shell/routes.gen.ts) wires this tag to /studio/*.

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-chrome.sui'

export default controller<Scope>({
  scope: () => {
    // No reactive state — the chrome is structural. The mount() hook
    // below dispatches a ready event so descendants (the rail, status
    // bar) can wire up to it.
    return {}
  },
  mount(_state, host) {
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
