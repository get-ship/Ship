import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-agent-slot.sui'

// Presentation is reflected-attribute driven; the only behavior is Save, which
// graduates the injected content into a durable artifact. The controller emits
// ship-save with the slot label so production can route it to the Save RPC.
export default controller({
  scope: ({ label }: ViewScope<Scope>, host) => {
    function save(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-save', {
          detail: { label: label() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return { save }
  },
})
