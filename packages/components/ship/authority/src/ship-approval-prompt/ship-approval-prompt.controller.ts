import { controller } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './ship-approval-prompt.sui'

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
// fall out of CSS alone. The buttons auto-hide because :host(:not([state='pending']))
// .actions { display: none; }.
export default controller({
  scope: ({ open, req }: ViewScope<Scope>, host) => {
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
  },
})
