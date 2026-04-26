/**
 * <ship-stepper>
 *
 * Linear progress indicator for multi-step flows (wizards, onboarding,
 * checkout). Renders `total` dots, with the active dot widened into a
 * pill and previously-completed dots tinted. For pipeline per-step
 * outcome (done/failed/pending/running) use `<ship-step-segments>`;
 * this primitive is for linear position only.
 *
 * Attributes:
 *   - total: number — how many steps in the flow (default: 4)
 *   - current: number — the 1-based index of the active step (default: 1)
 *   - tone: default | info | success | warning | destructive | accent | muted
 *     Active dot color. Upstream dots render as a faded tint of the
 *     same tone. Default tone uses --primary.
 *   - size: sm | md | lg (default: md)
 *   - label: string — accessible name for the progressbar role
 *
 * Parts:
 *   - dot — every rendered dot (pick via [data-state])
 *
 * No slots. Host role is `progressbar` with aria-valuemin=1,
 * aria-valuemax=total, aria-valuenow=current.
 */

import { LitElement, css, html } from 'lit'
import { reducedMotion, toneVars } from './util'
import type { Size, Tone } from './util'

export class ShipStepper extends LitElement {
  static styles = [
    reducedMotion,
    toneVars,
    css`
      :host {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: var(--space-2);
        --dot-w: 7px;
        --dot-h: 7px;
        --dot-active-w: 22px;
      }
      :host([size='sm']) { --dot-w: 6px; --dot-h: 6px; --dot-active-w: 18px; }
      :host([size='lg']) { --dot-w: 9px; --dot-h: 9px; --dot-active-w: 28px; }

      /* Inactive dot color — a faded tint of the muted-foreground so it
       * reads as "not yet" regardless of tone. */
      .dot {
        width: var(--dot-w);
        height: var(--dot-h);
        border-radius: var(--radius-full);
        background: color-mix(in oklch, var(--muted-foreground) 35%, transparent);
        transition: all var(--duration-medium) var(--ease-out);
        flex-shrink: 0;
      }

      /* Current step — widened pill in the brand primary color by
       * default. A 'tone' attribute opts into the shared tone ramp. */
      .dot[data-state='active'] {
        width: var(--dot-active-w);
        background: var(--primary);
      }
      :host([tone]) .dot[data-state='active'] {
        background: var(--tone-fg);
      }

      /* Completed steps — same color but faded so the current step still
       * reads as "you are here". */
      .dot[data-state='done'] {
        background: color-mix(in oklch, var(--primary) 55%, transparent);
      }
      :host([tone]) .dot[data-state='done'] {
        background: color-mix(in oklch, var(--tone-fg) 55%, transparent);
      }
    `,
  ]

  static properties = {
    total: { type: Number, reflect: true },
    current: { type: Number, reflect: true },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
  }

  total = 4
  current = 1
  tone?: Tone
  size: Size = 'md'
  label = ''

  connectedCallback() {
    super.connectedCallback()
    this.setAttribute('role', 'progressbar')
    this.syncAria()
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('total') || changed.has('current') || changed.has('label')) {
      this.syncAria()
    }
  }

  private syncAria() {
    const t = Math.max(1, Math.floor(this.total || 0))
    const c = Math.max(1, Math.min(t, Math.floor(this.current || 1)))
    this.setAttribute('aria-valuemin', '1')
    this.setAttribute('aria-valuemax', String(t))
    this.setAttribute('aria-valuenow', String(c))
    if (this.label) this.setAttribute('aria-label', this.label)
    else if (!this.hasAttribute('aria-label')) {
      this.setAttribute('aria-label', `Step ${c} of ${t}`)
    }
  }

  render() {
    const t = Math.max(1, Math.floor(this.total || 0))
    const c = Math.max(1, Math.min(t, Math.floor(this.current || 1)))
    const dots = []
    for (let i = 1; i <= t; i++) {
      const state = i === c ? 'active' : i < c ? 'done' : 'pending'
      dots.push(html`<span class="dot" part="dot" data-state=${state} aria-hidden="true"></span>`)
    }
    return html`${dots}`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-stepper')) {
  customElements.define('ship-stepper', ShipStepper)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-stepper': ShipStepper
  }
}
