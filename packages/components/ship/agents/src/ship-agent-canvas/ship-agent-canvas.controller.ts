import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-agent-canvas.sui'

// The canvas chrome is reflected-attribute driven; the control bar emits the
// three session verbs (undo · stop · save) with the agent + session in the
// detail so production can route them to the Stop / Save RPCs.
export default controller({
  scope: ({ agent, session }: ViewScope<Scope>, host) => {
    const emit = (name: string): void => {
      host?.dispatchEvent(
        new CustomEvent(name, {
          detail: { agent: agent(), session: session() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return {
      undo: () => emit('ship-undo'),
      stop: () => emit('ship-stop'),
      save: () => emit('ship-save'),
    }
  },
})
