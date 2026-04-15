/**
 * <ship-progress>
 *
 * Linear progress indicator. Determinate (0..max) or indeterminate when
 * value is omitted.
 *
 * Attributes:
 *   - value: number | null
 *   - max: number (default 100)
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md (default md)
 */

import { LitElement, css, html, nothing } from 'lit'
import { reducedMotion, toneVars } from './util'
import type { Tone } from './util'

export class ShipProgress extends LitElement {
  static styles = [
    reducedMotion,
    toneVars,
    css`
      :host { display: block; }
      .track {
        background: var(--muted);
        border-radius: 999px;
        overflow: hidden;
        height: 6px;
      }
      :host([size='sm']) .track { height: 3px; }
      .bar {
        height: 100%;
        background: var(--tone-fg);
        border-radius: inherit;
        transition: width 180ms ease;
      }
      :host(:not([value])) .bar {
        width: 35%;
        animation: indet 1.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
      }
      @keyframes indet {
        0% { transform: translateX(-100%); }
        60% { transform: translateX(200%); }
        100% { transform: translateX(200%); }
      }
    `,
  ]

  static properties = {
    value: { type: Number, reflect: true },
    max: { type: Number },
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    label: { type: String },
  }

  value: number | null
  max: number
  tone: Tone
  size: 'sm' | 'md'
  label: string

  constructor() {
    super()
    this.value = null
    this.max = 100
    this.tone = 'info'
    this.size = 'md'
    this.label = ''
  }

  render() {
    const pct = this.value == null ? null : Math.max(0, Math.min(100, (this.value / this.max) * 100))
    return html`
      <div class="track" role="progressbar"
        aria-valuemin="0"
        aria-valuemax=${this.max}
        aria-valuenow=${this.value != null ? this.value : nothing}
        aria-busy=${this.value == null ? 'true' : 'false'}
        aria-label=${this.label || nothing}
      >
        <div class="bar" style=${pct != null ? `width: ${pct}%` : ''}></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-progress')) {
  customElements.define('ship-progress', ShipProgress)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-progress': ShipProgress
  }
}
