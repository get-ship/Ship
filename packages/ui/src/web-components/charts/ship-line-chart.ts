/**
 * <ship-line-chart>
 *
 * Multi-series line chart.
 *
 * Attributes:
 *   - series: JSON string `[{name, data: number[], tone?}]`
 *   - labels: comma-separated x-axis labels (used only for accessibility + titles)
 *   - width / height: viewport (default 300x200)
 *   - points: boolean — render dots at data points
 *   - grid: boolean — render subtle y-grid (4 divisions)
 *
 * Slots:
 *   - (default) — legend content
 */

import { LitElement, css, html, svg } from 'lit'
import { linePath, parseJson, parseLabels, toneColor } from './util'

type Series = { name: string; data: number[]; tone?: string }

export class ShipLineChart extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 200px;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .wrap { display: flex; flex-direction: column; gap: 6px; width: 100%; height: 100%; }
    svg { display: block; width: 100%; flex: 1; overflow: visible; }
    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .legend:empty { display: none; }
    .grid-line { stroke: var(--border); stroke-width: 1; stroke-dasharray: 2 3; }
  `

  static properties = {
    series: { type: String, reflect: true },
    labels: { type: String },
    width: { type: Number },
    height: { type: Number },
    points: { type: Boolean, reflect: true },
    grid: { type: Boolean, reflect: true },
    label: { type: String },
  }

  series = ''
  labels = ''
  width = 300
  height = 200
  points = false
  grid = false
  label = 'line chart'

  render() {
    const allSeries = parseJson<Series[]>(this.series, [])
    const w = this.width
    const h = this.height
    const _labels = parseLabels(this.labels)
    const flat = allSeries.flatMap((s) => s.data)
    const min = flat.length ? Math.min(...flat) : 0
    const max = flat.length ? Math.max(...flat) : 1
    const range = max - min || 1
    const innerH = h - 4
    const gridLines = this.grid
      ? [0.25, 0.5, 0.75].map((f) => 2 + innerH * f)
      : []
    return html`
      <div class="wrap">
        <svg viewBox="0 0 ${w} ${h}" preserveAspectRatio="none" role="img" aria-label=${this.label}>
          ${gridLines.map(
            (y) => svg`<line class="grid-line" x1="0" x2=${w} y1=${y.toFixed(2)} y2=${y.toFixed(2)}></line>`,
          )}
          ${allSeries.map((s) => {
            const color = toneColor(s.tone)
            if (s.data.length < 2) return null
            // Normalize each series against the global min/max.
            const normalized = s.data.map((v) => ((v - min) / range) * 100)
            const { d, points } = linePath(normalized, w, h, 2)
            return svg`
              <path d=${d} fill="none" stroke=${color} stroke-width="1.75"
                stroke-linecap="round" stroke-linejoin="round"></path>
              ${
                this.points
                  ? points.map(
                      ([x, y]) =>
                        svg`<circle cx=${x.toFixed(2)} cy=${y.toFixed(2)} r="2.5" fill=${color}></circle>`,
                    )
                  : null
              }
            `
          })}
        </svg>
        <div class="legend"><slot></slot></div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-line-chart')) {
  customElements.define('ship-line-chart', ShipLineChart)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-line-chart': ShipLineChart
  }
}
