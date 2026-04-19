/**
 * <ship-donut>
 *
 * Distribution donut. Segments drawn as stroked arcs so thickness is uniform.
 *
 * Attributes:
 *   - data: JSON `[{label, value, tone?}]`
 *   - size: pixel dimension (default 120)
 *   - thickness: stroke width in px (default 16)
 *
 * Slots:
 *   - (default) — center content (typically a big number)
 */

import { LitElement, css, html, svg } from 'lit'
import { parseJson, toneColor } from './util'

type Segment = { label: string; value: number; tone?: string }

export class ShipDonut extends LitElement {
  static styles = css`
    :host {
      display: inline-block;
      width: var(--ship-donut-size, 120px);
      height: var(--ship-donut-size, 120px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; transform: rotate(-90deg); }
    .center {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      font-family: var(--font-display);
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-lg);
      color: var(--foreground);
      line-height: var(--leading-tight);
      pointer-events: none;
    }
    .track { stroke: var(--muted); }
  `

  static properties = {
    data: { type: String, reflect: true },
    size: { type: Number },
    thickness: { type: Number },
    label: { type: String },
  }

  data = ''
  size = 120
  thickness = 16
  label = 'donut chart'

  updated() {
    this.style.setProperty('--ship-donut-size', `${this.size}px`)
  }

  render() {
    const segs = parseJson<Segment[]>(this.data, [])
    const s = this.size
    const t = this.thickness
    const r = (s - t) / 2
    const c = s / 2
    const circumference = 2 * Math.PI * r
    const total = segs.reduce((acc, x) => acc + Math.max(0, x.value), 0)
    let offset = 0
    return html`
      <svg viewBox="0 0 ${s} ${s}" role="img" aria-label=${this.label}>
        <circle class="track" cx=${c} cy=${c} r=${r} fill="none" stroke-width=${t}></circle>
        ${total > 0
          ? segs.map((seg) => {
              const len = (Math.max(0, seg.value) / total) * circumference
              const dash = `${len.toFixed(2)} ${(circumference - len).toFixed(2)}`
              const node = svg`
                <circle
                  cx=${c} cy=${c} r=${r} fill="none"
                  stroke=${toneColor(seg.tone)}
                  stroke-width=${t}
                  stroke-dasharray=${dash}
                  stroke-dashoffset=${(-offset).toFixed(2)}
                  stroke-linecap="butt"
                ><title>${seg.label}: ${seg.value}</title></circle>
              `
              offset += len
              return node
            })
          : null}
      </svg>
      <div class="center"><slot></slot></div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-donut')) {
  customElements.define('ship-donut', ShipDonut)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-donut': ShipDonut
  }
}
