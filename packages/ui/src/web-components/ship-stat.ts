/**
 * <ship-stat>
 *
 * KPI tile. Matches pipelines `.stat` + `.stat-label` + `.stat-value` with
 * accent coloring on the numeric value. Tabular-nums built in.
 *
 * Attributes:
 *   - accent: default | primary | info | success | destructive | warning |
 *     accent (colors the value)
 *   - interactive: boolean — hover lift
 *
 * Slots:
 *   - label — micro-kicker label (uppercase)
 *   - (default) — the big numeric value
 *   - hint — optional small text below the value
 */

import { LitElement, css, html } from 'lit'
type StatAccent = 'default' | 'primary' | 'info' | 'success' | 'destructive' | 'warning' | 'accent'

export class ShipStat extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      gap: 4px;
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 16px 18px;
      transition: border-color 160ms, transform 160ms;
    }
    :host([interactive]:hover) {
      border-color: color-mix(in oklch, var(--border) 180%, transparent);
      transform: translateY(-1px);
    }
    .label {
      font-size: 10.5px;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .value {
      font-family: var(--font-display);
      font-size: 28px;
      font-weight: 600;
      font-variant-numeric: tabular-nums;
      letter-spacing: -0.02em;
      color: var(--foreground);
    }
    :host([accent='primary']) .value { color: var(--primary); }
    :host([accent='info']) .value { color: var(--info); }
    :host([accent='success']) .value { color: var(--success); }
    :host([accent='destructive']) .value { color: var(--destructive); }
    :host([accent='warning']) .value { color: var(--warning); }
    :host([accent='accent']) .value { color: var(--accent); }

    .hint {
      font-size: 11px;
      color: var(--muted-foreground);
    }
  `

  static properties = {
    accent: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
  }

  accent: StatAccent = 'default'
  interactive = false
  render() {
    return html`
      <span class="label"><slot name="label"></slot></span>
      <span class="value"><slot></slot></span>
      <span class="hint"><slot name="hint"></slot></span>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-stat')) {
  customElements.define('ship-stat', ShipStat)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-stat': ShipStat
  }
}
