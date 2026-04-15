/**
 * <ship-table>
 *
 * Generic data table. Consumers compose `<thead>` and `<tbody>` via slots;
 * the component supplies consistent styling and two display modes.
 *
 * Attributes:
 *   - dense: boolean — tighter cell padding
 *   - hoverable: boolean (default: true) — row hover highlight
 *
 * Slots:
 *   - head — slot for thead content (rows with th)
 *   - (default) — slot for tbody rows
 */

import { LitElement, css, html, nothing } from 'lit'

export class ShipTable extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      font-size: var(--text-sm);
      color: var(--foreground);
    }
    .wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--radius-md); }
    table { width: 100%; border-collapse: collapse; }
    ::slotted(tr) { border-bottom: 1px solid var(--border); }
    ::slotted(tr:last-child) { border-bottom: none; }
    ::slotted(th), ::slotted(td) {
      text-align: left;
      padding: 8px 12px;
      vertical-align: middle;
    }
    ::slotted(th) {
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-xs);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--muted-foreground);
      background: color-mix(in oklch, var(--muted) 60%, var(--background));
    }
    :host([dense]) ::slotted(th),
    :host([dense]) ::slotted(td) { padding: 4px 8px; }
    :host([hoverable]) ::slotted(tr:hover) { background: var(--muted); }
    thead { border-bottom: 1px solid var(--border); }
  `

  static properties = {
    dense: { type: Boolean, reflect: true },
    hoverable: { type: Boolean, reflect: true },
    label: { type: String },
  }

  dense = false
  hoverable = true
  label = ''

  render() {
    return html`
      <div class="wrap" part="wrap">
        <table part="table" aria-label=${this.label || nothing}>
          <thead part="thead"><slot name="head"></slot></thead>
          <tbody part="tbody"><slot></slot></tbody>
        </table>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-table')) {
  customElements.define('ship-table', ShipTable)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-table': ShipTable
  }
}
