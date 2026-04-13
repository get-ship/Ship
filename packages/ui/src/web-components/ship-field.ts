/**
 * <ship-field>
 *
 * Vertical stack for a label + control + optional hint/error. Matches the
 * specs view `.meta-section` layout: uppercase micro-label above, tight
 * stacked control(s) below.
 *
 * Slots:
 *   - label — label element (e.g. <ship-label>)
 *   - hint — muted helper text under the control
 *   - error — destructive-toned error text (replaces hint when invalid)
 *   - (default) the control
 *
 * Attributes:
 *   - invalid: boolean — reveals the error slot
 */

import { LitElement, css, html } from 'lit'
export class ShipField extends LitElement {
  static styles = css`
    :host { display: flex; flex-direction: column; gap: 6px; }
    .hint, .error { font-size: 11px; line-height: 1.4; }
    .hint { color: var(--muted-foreground); }
    .error { color: var(--destructive); }
    :host(:not([invalid])) .error { display: none; }
    :host([invalid]) .hint { display: none; }
  `

  static properties = {
    invalid: { type: Boolean, reflect: true },
  }

  invalid = false
  render() {
    return html`
      <slot name="label"></slot>
      <slot></slot>
      <div class="hint"><slot name="hint"></slot></div>
      <div class="error"><slot name="error"></slot></div>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-field')) {
  customElements.define('ship-field', ShipField)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-field': ShipField
  }
}
