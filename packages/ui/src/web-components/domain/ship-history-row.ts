/**
 * <ship-history-row>
 *
 * Row primitive for the pipelines history table. Renders as a native
 * <tr> by wrapping its host in a table-row display; cells are slotted
 * by column name so the rendering order stays under the view's control.
 *
 * Attributes:
 *   - state — pipeline run final state (drives badge tone at the caller
 *     through composed ship-badge slot)
 *
 * Slots:
 *   - slug, pipeline, status, duration, completed — one cell each
 *
 * Note: this is a fully visual primitive; the <tr> goes directly inside
 * <tbody>. Browsers will hoist text nodes out of a custom element inside
 * a table, so we render the actual <tr>/<td> structure ourselves.
 */

import { LitElement, css, html } from 'lit'
export class ShipHistoryRow extends LitElement {
  static styles = css`
    :host {
      display: table-row;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
      font-size: 12.5px;
      font-variant-numeric: tabular-nums;
      vertical-align: middle;
    }
    :host(:last-child) { border-bottom: 0; }
    :host(:hover) { background: var(--muted); }
    .cell { display: table-cell; padding: 12px 16px; }
  `

  static properties = {
    state: { type: String, reflect: true },
  }

  state = ''
  render() {
    return html`
      <span class="cell"><slot name="slug"></slot></span>
      <span class="cell"><slot name="pipeline"></slot></span>
      <span class="cell"><slot name="status"></slot></span>
      <span class="cell"><slot name="duration"></slot></span>
      <span class="cell"><slot name="completed"></slot></span>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-history-row')) {
  customElements.define('ship-history-row', ShipHistoryRow)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-history-row': ShipHistoryRow
  }
}
