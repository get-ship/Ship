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
import { customElement, property } from 'lit/decorators.js'

@customElement('ship-separator')
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

  @property({ reflect: true }) orientation: 'horizontal' | 'vertical' = 'horizontal'
  @property({ reflect: true }) variant: 'solid' | 'dashed' = 'solid'

  render() {
    this.setAttribute('role', 'separator')
    this.setAttribute('aria-orientation', this.orientation)
    return html``
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-separator': ShipSeparator
  }
}
