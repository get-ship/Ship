/**
 * <ship-heatmap>
 *
 * Time × category intensity grid.
 *
 * Attributes:
 *   - data: JSON `[{x, y, value}]` where value is 0..1
 *   - rows: comma-separated y labels (top → bottom)
 *   - cols: comma-separated x labels (left → right)
 *   - tone: fixed vocabulary (+ primary) — base hue for intensity mix
 *
 * Cells render as:
 *   fill: color-mix(in oklch, var(--<tone>) calc(value * 100%), transparent)
 *
 * Slots: none.
 */

import { LitElement, css, html, svg } from 'lit'
import { parseJson, parseLabels } from './util'

type Cell = { x: string; y: string; value: number }

const TONE_CSS_VAR: Record<string, string> = {
  default: '--foreground',
  primary: '--primary',
  info: '--info',
  success: '--success',
  warning: '--warning',
  destructive: '--destructive',
  accent: '--accent',
  muted: '--muted-foreground',
}

export class ShipHeatmap extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
    }
    .grid {
      display: grid;
      gap: 3px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .cell {
      aspect-ratio: 1 / 1;
      border-radius: var(--radius-sm);
      background: var(--muted);
    }
    .col-label, .row-label {
      font-size: 10px;
      color: var(--muted-foreground);
      font-weight: var(--font-weight-medium);
      display: flex;
      align-items: center;
    }
    .col-label { justify-content: center; }
    .row-label { justify-content: flex-end; padding-right: 6px; }
  `

  static properties = {
    data: { type: String, reflect: true },
    rows: { type: String },
    cols: { type: String },
    tone: { type: String, reflect: true },
    label: { type: String },
  }

  data = ''
  rows = ''
  cols = ''
  tone = 'primary'
  label = 'heatmap'

  render() {
    const rows = parseLabels(this.rows)
    const cols = parseLabels(this.cols)
    const cells = parseJson<Cell[]>(this.data, [])
    const toneVar = TONE_CSS_VAR[this.tone] ?? TONE_CSS_VAR.primary
    const lookup = new Map<string, number>()
    for (const c of cells) lookup.set(`${c.y}\u0000${c.x}`, c.value)
    const gridStyle = `grid-template-columns: auto repeat(${cols.length}, 1fr);`
    return html`
      <div class="grid" style=${gridStyle} role="table" aria-label=${this.label}>
        <div></div>
        ${cols.map((c) => html`<div class="col-label">${c}</div>`)}
        ${rows.map(
          (r) => html`
            <div class="row-label">${r}</div>
            ${cols.map((c) => {
              const v = Math.max(0, Math.min(1, lookup.get(`${r}\u0000${c}`) ?? 0))
              const pct = (v * 100).toFixed(1)
              const style = v > 0
                ? `background: color-mix(in oklch, var(${toneVar}) ${pct}%, transparent);`
                : ''
              return html`<div class="cell" style=${style} title="${r} / ${c}: ${v}"></div>`
            })}
          `,
        )}
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-heatmap')) {
  customElements.define('ship-heatmap', ShipHeatmap)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-heatmap': ShipHeatmap
  }
}
