<scope>
  props: {
    count = '0'
  }
</scope>
<controller>
import { controller } from '@ship/sail-runtime'
import type { Scope } from './ship-approvals-queue.sail'

// The queue is presentational + event-out: it owns no list state (the host
// slots the pending checkpoints). The controller exists only to emit the
// operator's batch intent up to the host, which owns the batched mutation.
export default controller((_ctx: Scope, host?: Element) => {
    function emit(type: 'ship-approve-all' | 'ship-deny-all'): void {
      host?.dispatchEvent(
        new CustomEvent(type, { detail: {}, bubbles: true, composed: true }),
      )
    }
    function approveAll(): void { emit('ship-approve-all') }
    function denyAll(): void { emit('ship-deny-all') }

    return { approveAll, denyAll }
})
</controller>

<!--
  <ship-approvals-queue> — @ship/authority · the cross-space "inbox" for the gate.
  A header (N pending) + batch verbs (Deny all · Approve all) over a stack of
  slotted approval checkpoints. Suspends span sessions; the operator decides them
  in a batch.

  Presentational + event-out: the host slots the pending checkpoints and owns the
  batch mutation. The queue only emits the operator's batch intent.

  Prop:   count   string — "{count} pending"
  Slot:   default → the per-op approval checkpoints
  Parts:  queue · body
  Events (bubble + composed):
    ship-approve-all { detail: {} }
    ship-deny-all    { detail: {} }
-->

<div>
  <div>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
    <span>Approvals</span>
    <span>{count()} pending</span>
    <span>
      <button type="button" on:click={denyAll}>Deny all</button>
      <button type="button" on:click={approveAll}>Approve all</button>
    </span>
  </div>
  <div><slot></slot></div>
</div>
