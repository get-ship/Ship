/**
 * <ship-checkbox>
 *
 * Wraps a native `<input type="checkbox">`. Label content goes in the
 * default slot.
 *
 * Attributes:
 *   - checked: boolean
 *   - name, value
 *   - disabled, required
 *   - indeterminate: boolean
 *
 * Parts:
 *   - input — native element
 *   - label — text wrapper
 *
 * Events:
 *   - ship-change — CustomEvent { detail: { checked } }
 */

import { LitElement, css, html } from 'lit'
export class ShipCheckbox extends LitElement {
  static formAssociated = true
  private internals: ElementInternals
  private defaultChecked = false
  constructor() {
    super()
    this.internals = this.attachInternals()
  }
  static styles = css`
    :host { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font: inherit; font-size: 13px; color: var(--foreground); }
    :host([disabled]) { cursor: not-allowed; opacity: 0.6; }
    label { display: inline-flex; align-items: center; gap: 8px; cursor: inherit; }
    input { margin: 0; accent-color: var(--primary); width: 14px; height: 14px; }
    input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
  `

  static properties = {
    checked: { type: Boolean, reflect: true },
    indeterminate: { type: Boolean },
    name: { type: String },
    value: { type: String },
    disabled: { type: Boolean, reflect: true },
    required: { type: Boolean, reflect: true },
    invalid: { type: Boolean, reflect: true },
  }

  checked = false
  indeterminate = false
  name = ''
  value = 'on'
  disabled = false
  required = false
  invalid = false
  get input(): HTMLInputElement { return this.renderRoot.querySelector('input')! as HTMLInputElement }

  connectedCallback() {
    super.connectedCallback()
    this.defaultChecked = this.checked
  }

  private setFormValue(v: string | null) {
    if (typeof this.internals.setFormValue === 'function') this.internals.setFormValue(v)
  }

  updated() {
    if (this.input) this.input.indeterminate = this.indeterminate
    this.setFormValue(this.checked ? this.value : null)
  }

  formResetCallback() {
    this.checked = this.defaultChecked
    this.indeterminate = false
  }

  formStateRestoreCallback(state: string | null) {
    this.checked = state === this.value
  }

  private onChange(e: Event) {
    const c = (e.target as HTMLInputElement).checked
    this.checked = c
    this.indeterminate = false
    this.setFormValue(c ? this.value : null)
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { checked: c }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <label part="label">
        <input
          part="input"
          type="checkbox"
          .checked=${this.checked}
          name=${this.name}
          value=${this.value}
          ?disabled=${this.disabled}
          ?required=${this.required}
          aria-checked=${this.indeterminate ? 'mixed' : (this.checked ? 'true' : 'false')}
          aria-invalid=${this.invalid ? 'true' : 'false'}
          @change=${this.onChange}
        />
        <slot></slot>
      </label>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-checkbox')) {
  customElements.define('ship-checkbox', ShipCheckbox)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-checkbox': ShipCheckbox
  }
}
