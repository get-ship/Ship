// `routes/home/controller.ts` — the home view's scope actions. Each
// hint pill dispatches the same event the global hotkey would fire
// (the host listens on `document` and toggles the command palette or
// navigates). All navigation goes through the public runtime API
// (`@ship/deck/route`'s `navigate()`) — no `chrome.navigate`, no
// `window.ship`.

import { controller, navigate, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => {
    return {
      onPaletteHint: () => {
        document.dispatchEvent(new CustomEvent('sui-open-command-palette', {
          bubbles: true, composed: true,
        }))
      },
      onStudioHint: () => navigate('/studio'),
      onHelpHint: () => {
        document.dispatchEvent(new CustomEvent('sui-open-help', {
          bubbles: true, composed: true,
        }))
      },
    }
  },
})
