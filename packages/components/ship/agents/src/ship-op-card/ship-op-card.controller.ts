import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-op-card.sui'

// Presentation is reflected-attribute driven; the only behavior is the
// disclosure chevron. The controller toggles the `open` attribute for an
// immediate visual and emits `ship-toggle {open}` so production can wire
// the disclosure to the op stream.
export default controller({
  scope: ({ open }: ViewScope<Scope>, host) => {
    function toggle(): void {
      const next = !open()
      host?.toggleAttribute('open', next)
      host?.dispatchEvent(
        new CustomEvent('ship-toggle', {
          detail: { open: next },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { toggle }
  },
})
