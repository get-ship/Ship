/**
 * <ship-label>
 *
 * Section / field label. Typography matches specs .meta-title and the
 * pipelines .stat-label (uppercase micro-kicker).
 *
 * Attributes:
 *   - for: id of the associated control (rendered on a native <label for=...>)
 *   - size: sm | md (default: sm)
 *   - required: boolean — shows `*` after the label
 *
 * Slots:
 *   - (default) label text
 */

import { LitElement, css, html } from 'lit'
export class ShipLabel extends LitElement {
  static styles = css`
    :host { display: inline-block; }
    label {
      font-family: var(--font-body);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--muted-foreground);
    }
    :host([size='sm']) label,
    :host(:not([size])) label { font-size: 10px; }
    :host([size='md']) label { font-size: 11px; }
    .req { color: var(--destructive); margin-left: 2px; }
  `

  static properties = {
    htmlFor: { type: String, attribute: 'for' },
    size: { type: String, reflect: true },
    required: { type: Boolean, reflect: true },
  }

  htmlFor = ''
  size: 'sm' | 'md' = 'sm'
  required = false
  render() {
    return html`
      <label for=${this.htmlFor}>
        <slot></slot>${this.required ? html`<span class="req" aria-hidden="true">*</span>` : null}
      </label>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-label')) {
  customElements.define('ship-label', ShipLabel)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-label': ShipLabel
  }
}
