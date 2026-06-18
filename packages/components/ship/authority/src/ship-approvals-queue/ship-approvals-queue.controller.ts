import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-approvals-queue.sui'

// The queue is presentational + event-out: it owns no list state (the host
// slots the pending checkpoints). The controller exists only to emit the
// operator's batch intent up to the host, which owns the batched mutation.
export default controller({
  scope: (_scope: ViewScope<Scope>, host) => {
    function emit(type: 'ship-approve-all' | 'ship-deny-all'): void {
      host?.dispatchEvent(
        new CustomEvent(type, { detail: {}, bubbles: true, composed: true }),
      )
    }
    function approveAll(): void { emit('ship-approve-all') }
    function denyAll(): void { emit('ship-deny-all') }

    return { approveAll, denyAll }
  },
})
