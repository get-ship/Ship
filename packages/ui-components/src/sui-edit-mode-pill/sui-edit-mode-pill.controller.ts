// Edit-mode pill controller — just a click handler that bubbles
// `sui:edit-mode:exit`. The shell wires this to flip the per-view
// edit-mode signal off (see apps/shell/src/chrome/edit-mode.ts).

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-edit-mode-pill.sui'

export default controller<Scope>({
  scope: (_state, host) => {
    const hostEl = host as HTMLElement
    function onExit(): void {
      hostEl.dispatchEvent(new CustomEvent('sui:edit-mode:exit', {
        bubbles: true,
        composed: true,
      }))
    }
    return { onExit }
  },
})
