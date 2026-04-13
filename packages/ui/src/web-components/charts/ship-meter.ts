/**
 * <ship-meter>
 *
 * Segmented horizontal bar (e.g. nginx worker status / capacity breakdown).
 *
 * Attributes:
 *   - segments: JSON `[{tone, count, label?}]`
 *   - total: optional total denominator. If omitted, sum of segments.
 *
 * Slots:
 *   - (default) — legend
 */

import { LitElement, css, html } from 'lit'
import { parseJson, toneColor } from './util'

type Segment = { tone: string; count: number; label?: string }

export class ShipMeter extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .wrap { display: flex; flex-direction: column; gap: 8px; }
    .bar {
      display: flex;
      gap: 2px;
      height: 10px;
      width: 100%;
      background: var(--muted);
      border-radius: var(--radius-full);
      overflow: hidden;
      padding: 0;
    }
    .seg:first-child {
      border-top-left-radius: var(--radius-full);
      border-bottom-left-radius: var(--radius-full);
    }
    .seg:last-child {
      border-top-right-radius: var(--radius-full);
      border-bottom-right-radius: var(--radius-full);
    }
    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .legend:empty { display: none; }
  `

  static properties = {
    segments: { type: String, reflect: true },
    total: { type: Number },
  }

  segments = ''
  total: number | null = null

  render() {
    const segs = parseJson<Segment[]>(this.segments, [])
    const sum = segs.reduce((a, s) => a + Math.max(0, s.count), 0)
    const denom = this.total ?? sum
    const safeDenom = denom <= 0 ? 1 : denom
    return html`
      <div class="wrap">
        <div class="bar" role="img" aria-label="meter">
          ${segs.map((s) => {
            const pct = (Math.max(0, s.count) / safeDenom) * 100
            if (pct <= 0) return null
            const style = `width: ${pct.toFixed(3)}%; background: ${toneColor(s.tone)};`
            return html`<div class="seg" style=${style} title="${s.label ?? s.tone}: ${s.count}"></div>`
          })}
        </div>
        <div class="legend"><slot></slot></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-meter')) {
  customElements.define('ship-meter', ShipMeter)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-meter': ShipMeter
  }
}
