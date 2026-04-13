/**
 * <ship-progress-ring>
 *
 * Circular progress (0–100).
 *
 * Attributes:
 *   - value: number 0..100
 *   - size: px (default 48)
 *   - thickness: px (default 4)
 *   - tone: fixed vocabulary (+ primary)
 *
 * Slots:
 *   - (default) — centered label
 */

import { LitElement, css, html } from 'lit'
import { toneColor } from './util'

export class ShipProgressRing extends LitElement {
  static styles = css`
    :host {
      display: inline-block;
      width: var(--ship-ring-size, 48px);
      height: var(--ship-ring-size, 48px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; transform: rotate(-90deg); }
    .label {
      position: absolute; inset: 0;
      display: flex; align-items: center; justify-content: center;
      font-size: var(--text-xs);
      font-weight: var(--font-weight-semibold);
      font-family: var(--font-body);
      color: var(--foreground);
      font-variant-numeric: tabular-nums;
    }
    .track { stroke: var(--muted); }
    .bar { transition: stroke-dashoffset var(--duration-medium) var(--ease-out); }
  `

  static properties = {
    value: { type: Number, reflect: true },
    size: { type: Number },
    thickness: { type: Number },
    tone: { type: String, reflect: true },
  }

  value = 0
  size = 48
  thickness = 4
  tone = 'primary'

  updated() {
    this.style.setProperty('--ship-ring-size', `${this.size}px`)
  }

  render() {
    const s = this.size
    const t = this.thickness
    const r = (s - t) / 2
    const c = s / 2
    const circ = 2 * Math.PI * r
    const pct = Math.max(0, Math.min(100, this.value))
    const offset = circ - (pct / 100) * circ
    const color = toneColor(this.tone)
    return html`
      <svg viewBox="0 0 ${s} ${s}" role="progressbar"
        aria-valuemin="0" aria-valuemax="100" aria-valuenow=${pct}>
        <circle class="track" cx=${c} cy=${c} r=${r}
          fill="none" stroke-width=${t}></circle>
        <circle class="bar" cx=${c} cy=${c} r=${r}
          fill="none" stroke=${color} stroke-width=${t}
          stroke-linecap="round"
          stroke-dasharray=${circ.toFixed(2)}
          stroke-dashoffset=${offset.toFixed(2)}></circle>
      </svg>
      <div class="label"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-progress-ring')) {
  customElements.define('ship-progress-ring', ShipProgressRing)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-progress-ring': ShipProgressRing
  }
}
