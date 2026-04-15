/**
 * <ship-gauge>
 *
 * Single value against a range rendered as a 270deg arc (gap at bottom).
 *
 * Attributes:
 *   - value: number
 *   - min: number (default 0)
 *   - max: number (default 100)
 *   - tone: explicit tone. If omitted and thresholds set, auto-tones:
 *           value < low-threshold → destructive
 *           value < high-threshold → warning
 *           else → success
 *   - low-threshold / high-threshold: optional numeric thresholds (auto-tone)
 *   - size: pixel dimension (default 140)
 *   - thickness: stroke width (default 12)
 *
 * Slots:
 *   - (default) — value label in the center
 */

import { LitElement, css, html, svg } from 'lit'
import { arcPath, toneColor } from './util'

export class ShipGauge extends LitElement {
  static styles = css`
    :host {
      display: inline-block;
      width: var(--ship-gauge-size, 140px);
      height: var(--ship-gauge-size, 140px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; }
    .label {
      position: absolute; inset: 0;
      display: flex; align-items: center; justify-content: center;
      font-family: var(--font-display);
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-xl);
      color: var(--foreground);
    }
    .track { stroke: var(--muted); }
  `

  static properties = {
    value: { type: Number, reflect: true },
    min: { type: Number },
    max: { type: Number },
    tone: { type: String, reflect: true },
    lowThreshold: { type: Number, attribute: 'low-threshold' },
    highThreshold: { type: Number, attribute: 'high-threshold' },
    size: { type: Number },
    thickness: { type: Number },
    label: { type: String },
  }

  value = 0
  min = 0
  max = 100
  label = 'gauge'
  tone: string | null = null
  lowThreshold: number | null = null
  highThreshold: number | null = null
  size = 140
  thickness = 12

  updated() {
    this.style.setProperty('--ship-gauge-size', `${this.size}px`)
  }

  private resolveTone(): string {
    if (this.tone) return this.tone
    if (this.lowThreshold != null && this.value < this.lowThreshold) return 'destructive'
    if (this.highThreshold != null && this.value < this.highThreshold) return 'warning'
    if (this.lowThreshold != null || this.highThreshold != null) return 'success'
    return 'primary'
  }

  render() {
    const s = this.size
    const t = this.thickness
    const r = (s - t) / 2 - 2
    const c = s / 2
    // 270 arc: start at 135deg, end at 45deg (going clockwise past bottom open).
    const start = (135 * Math.PI) / 180
    const end = (405 * Math.PI) / 180 // 45 + 360
    const fullSweep = end - start
    const ratio = Math.max(
      0,
      Math.min(1, (this.value - this.min) / (this.max - this.min || 1)),
    )
    const valueEnd = start + fullSweep * ratio
    const trackD = arcPath(c, c, r, start, end)
    const valueD = ratio > 0 ? arcPath(c, c, r, start, valueEnd) : ''
    const color = toneColor(this.resolveTone())
    return html`
      <svg viewBox="0 0 ${s} ${s}" role="img" aria-label=${this.label}>
        <path class="track" d=${trackD} fill="none" stroke-width=${t} stroke-linecap="round"></path>
        ${valueD
          ? svg`<path d=${valueD} fill="none" stroke=${color} stroke-width=${t} stroke-linecap="round"></path>`
          : null}
      </svg>
      <div class="label"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-gauge')) {
  customElements.define('ship-gauge', ShipGauge)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-gauge': ShipGauge
  }
}
