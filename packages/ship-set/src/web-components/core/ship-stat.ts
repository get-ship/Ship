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
 *   - trend — optional delta adornment (e.g. <ship-badge tone="success">+12%</ship-badge>)
 *             rendered inline to the right of the value.
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
      transition:
        border-color var(--duration-medium) var(--ease-out),
        transform var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }
    :host([interactive]) { cursor: pointer; }
    :host([interactive]:hover) {
      border-color: color-mix(in oklch, var(--border) 180%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-sm);
    }
    .label {
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .value-row {
      display: flex;
      align-items: baseline;
      gap: 8px;
      flex-wrap: wrap;
    }
    .value {
      font-family: var(--heading-font);
      font-size: var(--text-3xl);
      font-weight: var(--font-weight-semibold);
      font-variant-numeric: tabular-nums;
      letter-spacing: -0.02em;
      color: var(--foreground);
    }
    .trend {
      display: inline-flex;
      align-items: center;
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
    ::slotted([slot='trend']) { font-size: var(--text-sm); }
  `

  static properties = {
    accent: { type: String, reflect: true },
    interactive: { type: Boolean, reflect: true },
  }

  accent: StatAccent = 'default'
  interactive = false

  connectedCallback() {
    super.connectedCallback()
    if (this.interactive) {
      if (!this.hasAttribute('role')) this.setAttribute('role', 'button')
      if (!this.hasAttribute('tabindex')) this.setAttribute('tabindex', '0')
      this.addEventListener('keydown', this.onKey)
    }
  }
  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.onKey)
  }
  private onKey = (e: KeyboardEvent) => {
    if (!this.interactive) return
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this.click() }
  }

  render() {
    return html`
      <span class="label"><slot name="label"></slot></span>
      <span class="value-row">
        <span class="value"><slot></slot></span>
        <span class="trend"><slot name="trend"></slot></span>
      </span>
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
