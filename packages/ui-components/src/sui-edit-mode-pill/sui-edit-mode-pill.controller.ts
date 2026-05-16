// Edit-mode pill controller — bubble click handlers for exit /
// annotate / send. Chrome (apps/shell/src/chrome/edit-mode.ts)
// listens for each event and routes to the right action:
//   sui:edit-mode:exit         → flip per-view edit-mode signal off
//   sui:annotation-mode:toggle → flip per-view annotation-mode signal
//   sui:annotations:send       → open cmdk AI mode with structured prefill

import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-edit-mode-pill.sui'

function emit(host: HTMLElement, type: string): void {
  host.dispatchEvent(new CustomEvent(type, { bubbles: true, composed: true }))
}

export default controller<Scope>({
  scope: (_state, host) => {
    const hostEl = host as HTMLElement
    return {
      onExit: () => emit(hostEl, 'sui:edit-mode:exit'),
      onAnnotateToggle: () => emit(hostEl, 'sui:annotation-mode:toggle'),
      onSend: () => emit(hostEl, 'sui:annotations:send'),
    }
  },
})
