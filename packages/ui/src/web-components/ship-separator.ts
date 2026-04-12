/**
 * <ship-separator>
 *
 * Horizontal or vertical divider. Defaults to horizontal 1px border line.
 *
 * Attributes:
 *   - orientation: horizontal | vertical (default horizontal)
 *   - variant: solid | dashed (default solid)
 */

import { LitElement, css, html } from 'lit'

export class ShipSeparator extends LitElement {
  static styles = css`
    :host { display: block; background: var(--border); }
    :host(:not([orientation='vertical'])) { height: 1px; width: 100%; margin: 8px 0; }
    :host([orientation='vertical']) { width: 1px; min-height: 100%; display: inline-block; margin: 0 8px; }
    :host([variant='dashed']) {
      background: transparent;
      border-top: 1px dashed var(--border);
      height: 0;
    }
    :host([variant='dashed'][orientation='vertical']) {
      border-top: 0;
      border-left: 1px dashed var(--border);
      width: 0;
    }
  `

  static properties = {
    orientation: { type: String, reflect: true },
    variant: { type: String, reflect: true },
  }

  orientation: 'horizontal' | 'vertical' = 'horizontal'
  variant: 'solid' | 'dashed' = 'solid'

  render() {
    this.setAttribute('role', 'separator')
    this.setAttribute('aria-orientation', this.orientation)
    return html``
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-separator')) {
  customElements.define('ship-separator', ShipSeparator)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-separator': ShipSeparator
  }
}
