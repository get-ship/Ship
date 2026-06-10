import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-agent-controls.sui'

// Each verb dispatches its canonical control event with the agent + state in
// the detail so production can route it to the matching control RPC. Which
// verbs are *visible* is a pure attribute-selector concern (see the .sui);
// the controller only emits.
export default controller({
  scope: ({ agent, state }: ViewScope<Scope>, host) => {
    const emit = (name: string): void => {
      host?.dispatchEvent(
        new CustomEvent(name, {
          detail: { agent: agent(), state: state() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return {
      start: () => emit('ship-start'),
      steer: () => emit('ship-steer'),
      approve: () => emit('ship-approve'),
      deny: () => emit('ship-deny'),
      rerun: () => emit('ship-rerun'),
      stop: () => emit('ship-stop'),
    }
  },
})
