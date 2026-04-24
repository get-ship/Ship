/**
 * <ship-sparkline>
 *
 * Tiny inline trend line rendered in pure SVG.
 *
 * Attributes:
 *   - data: comma-separated numbers (e.g. "1,3,2,5,4")
 *   - tone: default | info | success | warning | destructive | accent | muted | primary
 *   - width: number (default 80)
 *   - height: number (default 20)
 *   - fill: boolean — fills area under line at 18% alpha
 *
 * Slots: none.
 */

import { LitElement, css, html, svg } from 'lit'
import { linePath, parseNums, toneColor } from './util'

export class ShipSparkline extends LitElement {
  static styles = css`
    :host {
      display: inline-block;
      width: var(--ship-sparkline-w, auto);
      height: var(--ship-sparkline-h, 40px);
      line-height: 0;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; overflow: visible; }
  `

  static properties = {
    data: { type: String, reflect: true },
    tone: { type: String, reflect: true },
    width: { type: Number },
    height: { type: Number },
    fill: { type: Boolean, reflect: true },
    label: { type: String },
  }

  data = ''
  tone = 'primary'
  width = 80
  height = 20
  fill = false
  label = 'sparkline'

  render() {
    const nums = parseNums(this.data)
    const w = this.width
    const h = this.height
    if (nums.length < 2) {
      return html`<svg viewBox="0 0 ${w} ${h}" width=${w} height=${h} aria-hidden="true"></svg>`
    }
    const color = toneColor(this.tone)
    const { d, points } = linePath(nums, w, h, 2)
    const last = points[points.length - 1]
    const first = points[0]
    const areaD = this.fill
      ? `${d} L${last[0].toFixed(2)},${h} L${first[0].toFixed(2)},${h} Z`
      : ''
    return html`
      <svg
        viewBox="0 0 ${w} ${h}"
        width=${w}
        height=${h}
        role="img"
        aria-label=${this.label}
      >
        ${this.fill
          ? svg`<path d=${areaD} fill=${color} fill-opacity="0.18" stroke="none"></path>`
          : null}
        <path
          d=${d}
          fill="none"
          stroke=${color}
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        ></path>
      </svg>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-sparkline')) {
  customElements.define('ship-sparkline', ShipSparkline)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-sparkline': ShipSparkline
  }
}
