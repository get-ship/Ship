/**
 * <ship-bar-chart>
 *
 * Vertical or horizontal bars.
 *
 * Attributes:
 *   - data: comma-separated numbers
 *   - labels: optional comma-separated labels (rendered as <title> per bar)
 *   - tone: fixed vocabulary (+ primary)
 *   - orientation: vertical | horizontal (default vertical)
 *   - width / height: numeric viewport size (default container, fallback 300x200)
 *   - max: optional numeric axis override
 *
 * Slots: none.
 */

import { LitElement, css, html, svg } from 'lit'
import { parseLabels, parseNums, toneColor } from './util'

export class ShipBarChart extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 200px;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; overflow: visible; }
  `

  static properties = {
    data: { type: String, reflect: true },
    labels: { type: String },
    tone: { type: String, reflect: true },
    orientation: { type: String, reflect: true },
    width: { type: Number },
    height: { type: Number },
    max: { type: Number },
    label: { type: String },
  }

  data = ''
  labels = ''
  tone = 'primary'
  orientation: 'vertical' | 'horizontal' = 'vertical'
  width = 300
  height = 200
  max: number | null = null
  label = 'bar chart'

  render() {
    const nums = parseNums(this.data)
    const labels = parseLabels(this.labels)
    const w = this.width
    const h = this.height
    if (nums.length === 0) {
      return html`<svg viewBox="0 0 ${w} ${h}" aria-hidden="true"></svg>`
    }
    const color = toneColor(this.tone)
    const maxVal = this.max ?? Math.max(...nums, 0)
    const safeMax = maxVal <= 0 ? 1 : maxVal
    const gap = 4
    const radius = 3
    const horizontal = this.orientation === 'horizontal'
    const n = nums.length
    const bars = nums.map((v, i) => {
      const ratio = Math.max(0, v) / safeMax
      if (horizontal) {
        const barH = (h - gap * (n - 1)) / n
        const y = i * (barH + gap)
        const barW = ratio * w
        return { x: 0, y, w: barW, h: barH }
      }
      const barW = (w - gap * (n - 1)) / n
      const x = i * (barW + gap)
      const barH = ratio * h
      const y = h - barH
      return { x, y, w: barW, h: barH }
    })
    return html`
      <svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" role="img" aria-label=${this.label}>
        ${bars.map(
          (b, i) => svg`
          <rect
            x=${b.x}
            y=${b.y}
            width=${Math.max(0, b.w)}
            height=${Math.max(0, b.h)}
            rx=${radius}
            ry=${radius}
            fill=${color}
          ><title>${labels[i] ?? String(nums[i])}: ${nums[i]}</title></rect>
        `,
        )}
      </svg>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-bar-chart')) {
  customElements.define('ship-bar-chart', ShipBarChart)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-bar-chart': ShipBarChart
  }
}
