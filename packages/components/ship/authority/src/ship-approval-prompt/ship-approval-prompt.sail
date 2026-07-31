<controller>
import { controller } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-approval-prompt.sail'

// Presentation is reflected-attribute driven (op · args · req · grant · blast ·
// state · open). The controller exists to wire the three interactive verbs:
//
//   • Details disclosure toggle — flips the `open` attribute and emits
//     `ship-inspect { req }` so the host (a transcript / modal) can mark the
//     request as inspected.
//   • Approve / Deny buttons — emit `ship-decide { req, decision }`. The host
//     is the one that calls ship.authority.decide() — the prompt never touches
//     the SDK so it stays trivially testable and reusable.
//
// `state` is reflected: when the host flips `state` to 'approved' / 'denied' /
// 'expired' after the decision is persisted, the resolved chip + tone shift
// fall out of CSS alone. The buttons auto-hide because ship-approval-prompt(:not([state='pending']))
// .actions { display: none; }.
export default controller(({ open, req }: ViewScope<Scope>, host) => {

    function toggleDetails(): void {
      const next = !open()
      host?.toggleAttribute('open', next)
      host?.dispatchEvent(
        new CustomEvent('ship-inspect', {
          detail: { req: req() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    function decide(decision: 'approve' | 'deny'): void {
      host?.dispatchEvent(
        new CustomEvent('ship-decide', {
          detail: { req: req(), decision },
          bubbles: true,
          composed: true,
        }),
      )
    }

    function approve(): void { decide('approve') }
    function deny(): void { decide('deny') }

    return { toggleDetails, approve, deny }
})
</controller>

<!--
  <ship-approval-prompt> — @ship/authority · the Ask-stance checkpoint. ★ human-in-the-loop.

  A suspended op held at the gate; Approve(request_id) resolves it. The op NEVER runs
  before approval. APPROACHABLE BY DEFAULT: collapsed shows just the ask + the verbs;
  `open` discloses the call / standing-grant gap / request id. Inline in a transcript or
  a focused modal. Attribute-driven (production wires the toggle + verbs). Tokens only.

  Props: op · args · req · grant · blast(C0|C1|C2|C3) · state(pending|approved|denied|expired) · open.

  Events (bubble + composed, per HANDOFF F "the .controller.ts emits them"):
    ship-decide    { detail: { req, decision: 'approve' | 'deny' } } — Approve/Deny buttons
    ship-inspect   { detail: { req } }                              — Details disclosure toggle
-->
<!-- <prop name="op" type="string" default="op" /> -->
<!-- <prop name="args" type="string" default="" /> -->
<!-- <prop name="req" type="string" default="" /> -->
<!-- <prop name="grant" type="string" default="" /> -->
<!-- <prop name="blast" values="C0|C1|C2|C3" default="C2" reflect /> -->
<!-- <prop name="state" values="pending|approved|denied|expired" default="pending" reflect /> -->
<!-- <prop name="open" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div>
  <div>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
    <span>
      <span>Approve</span>
      <span>Approved</span>
      <span>Denied</span>
      <span>Expired</span>
    </span>
    <span>{op()}</span>
    <span>{blast()}</span>
    <button type="button" aria-label="Details" data-test="details" on:click={toggleDetails}>details<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"></path></svg></button>
  </div>
  <div>
    <div><span>{op()}</span>({args()})<span>req {req()}</span></div>
    <div><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2 2 7v6c0 5 4 8 10 9 6-1 10-4 10-9V7z"></path></svg>exceeds standing grant <b>{grant()}</b></div>
  </div>
  <div>
    <span data-test="resolved-approved"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"></path></svg>op dispatched</span>
    <span data-test="resolved-denied"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6L6 18M6 6l12 12"></path></svg>op never ran</span>
    <span data-test="resolved-expired"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"></circle><path d="M12 7v5l3 2"></path></svg>released unrun</span>
    <span>op held at the gate</span>
    <span>
      <button type="button" data-test="deny" on:click={deny}>Deny</button>
      <button type="button" data-test="approve" on:click={approve}>Approve</button>
    </span>
  </div>
</div>
