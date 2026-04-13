/**
 * <ship-metachip>
 *
 * Compact label + value pair for metadata rows. Renders as two inline
 * segments separated by a subtle divider. Designed to sit in a row of
 * sibling metachips separated by hairline dividers — put a wrapper
 * `display:flex; gap:0` around them and each chip contributes its own
 * right border.
 *
 * Attributes:
 *   - label: string — key text
 *   - value: string — value text
 *
 * Slots:
 *   - label — overrides `label` attr (lets you embed code/icon)
 *   - (default) — value content; overrides `value` attr
 */

import { LitElement, css, html } from 'lit'

export class ShipMetachip extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: baseline;
      gap: 8px;
      padding: 4px 14px;
      border-right: 1px solid color-mix(in oklch, var(--border) 70%, transparent);
      font-size: var(--text-xs);
      line-height: var(--leading-tight);
    }
    :host(:last-of-type) { border-right: 0; padding-right: 0; }
    :host(:first-of-type) { padding-left: 0; }
    .k {
      font-size: 10px;
      font-weight: var(--font-weight-semibold);
      color: var(--muted-foreground);
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }
    .v {
      font-family: var(--font-mono);
      color: var(--foreground);
      font-size: var(--text-xs);
    }
    ::slotted(*) { color: inherit; }
  `

  static properties = {
    label: { type: String, reflect: true },
    value: { type: String, reflect: true },
  }

  label = ''
  value = ''

  render() {
    return html`
      <span class="k" part="label"><slot name="label">${this.label}</slot></span>
      <span class="v" part="value"><slot>${this.value}</slot></span>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-metachip')) {
  customElements.define('ship-metachip', ShipMetachip)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-metachip': ShipMetachip
  }
}
