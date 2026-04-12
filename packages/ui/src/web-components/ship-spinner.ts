/**
 * <ship-spinner>
 *
 * Rotating circle spinner. Matches the 2.2s spin used by pipelines
 * `.step-running .step-icon`, and also usable standalone inline.
 *
 * Attributes:
 *   - size: sm | md | lg (default: md = 14px)
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - label: accessible label text (default: "Loading")
 */

import { LitElement, css, html } from 'lit'
import { toneVars } from './util'
import type { Size, Tone } from './util'

export class ShipSpinner extends LitElement {
  static styles = [
    toneVars,
    css`
      :host { display: inline-flex; vertical-align: middle; }
      .spin {
        width: 14px; height: 14px;
        border: 2px solid color-mix(in oklch, var(--tone-fg) 25%, transparent);
        border-top-color: var(--tone-fg);
        border-radius: 50%;
        animation: spin 0.9s linear infinite;
      }
      :host([size='sm']) .spin { width: 10px; height: 10px; border-width: 1.5px; }
      :host([size='lg']) .spin { width: 22px; height: 22px; border-width: 2.5px; }
      @keyframes spin { to { transform: rotate(360deg); } }
    `,
  ]

  static properties = {
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
  }

  tone: Tone
  size: Size
  label: string

  constructor() {
    super()
    this.tone = 'info'
    this.size = 'md'
    this.label = 'Loading'
  }

  render() {
    return html`<span class="spin" part="spinner" role="status" aria-label=${this.label}></span>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-spinner')) {
  customElements.define('ship-spinner', ShipSpinner)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-spinner': ShipSpinner
  }
}
