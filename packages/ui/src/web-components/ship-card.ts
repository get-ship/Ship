/**
 * <ship-card>
 *
 * Generic container matching pipelines `.def-card` / `.run-card` base
 * surface (bg: var(--card), 1px border, var(--radius)). Layout is handled
 * via slots; callers can add their own accent treatments via `tone`.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - variant: default | outline (default: default)
 *     - accent-bar: boolean — renders a 2-stop gradient top-bar on hover
 *       (matches .def-card::before behavior)
 *   - interactive: boolean — hover lift + cursor:pointer (matches .def-card)
 *   - status-rail: boolean — left colored rail (3px), tone-driven (matches
 *     .run-card.is-{status})
 *   - fill: boolean — tone-tinted background wash (uses --tone-bg). Pairs
 *     with `status-rail` or `tone` for an alert-style card.
 *
 * Slots:
 *   - header
 *   - (default)
 *   - footer
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from './util'
import type { Tone } from './util'

export class ShipCard extends LitElement {
  static styles = [
    toneVars,
    css`
      :host {
        display: block;
        background: var(--card);
        color: var(--card-foreground);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 16px 20px;
        position: relative;
        transition:
          border-color var(--duration-medium) var(--ease-out),
          transform var(--duration-fast) var(--ease-out),
          box-shadow var(--duration-fast) var(--ease-out);
      }
      :host([interactive]) { cursor: pointer; }
      :host([interactive]:hover) {
        border-color: color-mix(in oklch, var(--border) 180%, transparent);
        transform: translateY(-1px);
        box-shadow: var(--shadow-md);
      }
      :host([accent-bar])::before {
        content: '';
        position: absolute; top: 0; left: 0; right: 0; height: 2px;
        background: linear-gradient(90deg, var(--primary), var(--accent));
        opacity: 0;
        transition: opacity var(--duration-medium) var(--ease-out);
        border-radius: var(--radius) var(--radius) 0 0;
      }
      :host([accent-bar]:hover)::before { opacity: 1; }

      /* status-rail — 3px left border tinted by tone */
      :host([status-rail]) {
        border-left: 3px solid var(--tone-fg);
        padding-left: 17px;
      }

      /* fill — tone-tinted background wash. Pairs well with status-rail. */
      :host([fill]) { background: var(--tone-bg); }

      :host([variant='outline']) { background: transparent; }

      .header { margin-bottom: 10px; }
      .footer { margin-top: 10px; }
      ::slotted([slot='header']) { display: block; }
    `,
  ]

  static properties = {
    tone: { type: String, reflect: true },
    variant: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
    accentBar: { type: Boolean, reflect: true, attribute: 'accent-bar' },
    statusRail: { type: Boolean, reflect: true, attribute: 'status-rail' },
    fill: { type: Boolean, reflect: true },
  }

  tone?: Tone
  variant: 'default' | 'outline'
  interactive: boolean
  accentBar: boolean
  statusRail: boolean
  fill: boolean

  constructor() {
    super()
    this.variant = 'default'
    this.interactive = false
    this.accentBar = false
    this.statusRail = false
    this.fill = false
  }

  connectedCallback() {
    super.connectedCallback()
    if (this.interactive) {
      if (!this.hasAttribute('role')) this.setAttribute('role', 'button')
      if (!this.hasAttribute('tabindex')) this.setAttribute('tabindex', '0')
      this.addEventListener('keydown', this.onKey)
    }
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.onKey)
  }
  private onKey = (e: KeyboardEvent) => {
    if (!this.interactive) return
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.click()
    }
  }

  render() {
    return html`
      <div class="header"><slot name="header"></slot></div>
      <slot></slot>
      <div class="footer"><slot name="footer"></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-card')) {
  customElements.define('ship-card', ShipCard)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-card': ShipCard
  }
}
